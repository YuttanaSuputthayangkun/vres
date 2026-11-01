use std::io::{BufRead, BufReader, Read};
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

use indicatif::{ProgressBar, ProgressStyle};

use crate::error::FfmpegError;

/// Facts about a source video read by ffprobe.
pub struct Probe {
    /// Width in pixels of the first video stream.
    pub width: u32,
    /// Container duration, if ffprobe reports one.
    pub duration: Option<Duration>,
}

/// Checks that `ffmpeg` can be executed.
pub fn check_available() -> Result<(), FfmpegError> {
    let status = Command::new("ffmpeg")
        .arg("-version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|source| FfmpegError::Spawn {
            program: "ffmpeg",
            source,
        })?;
    if status.success() {
        Ok(())
    } else {
        Err(FfmpegError::Failed {
            program: "ffmpeg",
            status,
            stderr: String::new(),
        })
    }
}

/// Reads the width of the first video stream and the duration of `src`, using ffprobe.
pub fn probe(src: &Path) -> Result<Probe, FfmpegError> {
    let output = Command::new("ffprobe")
        .args(["-v", "error", "-select_streams", "v:0"])
        .args(["-show_entries", "stream=width:format=duration"])
        .args(["-of", "default=noprint_wrappers=1"])
        .arg(src)
        .output()
        .map_err(|source| FfmpegError::Spawn {
            program: "ffprobe",
            source,
        })?;
    if !output.status.success() {
        return Err(FfmpegError::Failed {
            program: "ffprobe",
            status: output.status,
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        });
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut width = None;
    let mut duration = None;
    for line in stdout.lines() {
        match line.trim().split_once('=') {
            Some(("width", value)) => width = value.parse().ok(),
            // "N/A" when unknown; leave as None.
            Some(("duration", value)) => {
                duration = value
                    .parse()
                    .ok()
                    .and_then(|s| Duration::try_from_secs_f64(s).ok())
            }
            _ => {}
        }
    }
    let width = width.ok_or_else(|| FfmpegError::InvalidProbeOutput(stdout.trim().to_string()))?;
    Ok(Probe { width, duration })
}

/// Resizes `src` to `width` (height follows aspect ratio) at `fps`, writing `dst`.
/// Shows a progress bar with an ETA when the source duration is known.
pub fn resize(
    src: &Path,
    dst: &Path,
    width: u32,
    fps: u32,
    overwrite: bool,
    duration: Option<Duration>,
) -> Result<(), FfmpegError> {
    // Progress goes to stdout as key=value lines; stderr only carries errors.
    let mut child = Command::new("ffmpeg")
        .args(["-hide_banner", "-loglevel", "error", "-nostats"])
        .args(["-progress", "pipe:1", "-i"])
        .arg(src)
        .args(["-vf", &format!("scale={width}:-2")])
        .args(["-r", &fps.to_string()])
        .args(["-c:a", "copy"])
        .arg(if overwrite { "-y" } else { "-n" })
        .arg(dst)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|source| FfmpegError::Spawn {
            program: "ffmpeg",
            source,
        })?;

    // Drain stderr on another thread so a full pipe can't block ffmpeg.
    let mut stderr = child.stderr.take().expect("stderr is piped");
    let stderr_reader = thread::spawn(move || {
        let mut buf = String::new();
        let _ = stderr.read_to_string(&mut buf);
        buf
    });

    let bar = progress_bar(duration);
    let stdout = child.stdout.take().expect("stdout is piped");
    for line in BufReader::new(stdout).lines().map_while(Result::ok) {
        // out_time_us is "N/A" until the first frame is written.
        if let Some(us) = line
            .strip_prefix("out_time_us=")
            .and_then(|v| v.parse::<u64>().ok())
        {
            bar.set_position((us / 1000).min(bar.length().unwrap_or(u64::MAX)));
        }
    }

    let status = child.wait().map_err(|source| FfmpegError::Spawn {
        program: "ffmpeg",
        source,
    })?;
    let stderr = stderr_reader.join().unwrap_or_default();
    if status.success() {
        bar.finish();
        Ok(())
    } else {
        bar.abandon();
        Err(FfmpegError::Failed {
            program: "ffmpeg",
            status,
            stderr: stderr.trim().to_string(),
        })
    }
}

/// A bar measured in milliseconds of output when `duration` is known, else a spinner.
fn progress_bar(duration: Option<Duration>) -> ProgressBar {
    let bar = match duration {
        Some(d) => ProgressBar::new(d.as_millis() as u64).with_style(
            ProgressStyle::with_template(
                "  [{elapsed_precise}] [{wide_bar:.cyan/blue}] {percent:>3}%  ETA {eta_precise}",
            )
            .expect("valid template")
            .progress_chars("=> "),
        ),
        None => ProgressBar::new_spinner().with_style(
            ProgressStyle::with_template("  {spinner} [{elapsed_precise}] converting...")
                .expect("valid template"),
        ),
    };
    bar.enable_steady_tick(Duration::from_millis(200));
    bar
}
