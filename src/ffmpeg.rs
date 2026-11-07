use std::io::{BufRead, BufReader, Read};
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

use crate::error::FfmpegError;

/// Facts about a source video read by ffprobe.
pub struct Probe {
    /// Height in pixels of the first video stream.
    pub height: u32,
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

/// Reads the height of the first video stream and the duration of `src`, using ffprobe.
pub fn probe(src: &Path) -> Result<Probe, FfmpegError> {
    let output = Command::new("ffprobe")
        .args(["-v", "error", "-select_streams", "v:0"])
        .args(["-show_entries", "stream=height:format=duration"])
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
    let mut height = None;
    let mut duration = None;
    for line in stdout.lines() {
        match line.trim().split_once('=') {
            Some(("height", value)) => height = value.parse().ok(),
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
    let height =
        height.ok_or_else(|| FfmpegError::InvalidProbeOutput(stdout.trim().to_string()))?;
    Ok(Probe { height, duration })
}

/// Resizes `src` to `height` (width follows aspect ratio) at `fps`, writing `dst`.
/// Calls `on_progress` with the amount of output written so far.
pub fn resize(
    src: &Path,
    dst: &Path,
    height: u32,
    fps: u32,
    overwrite: bool,
    on_progress: &mut dyn FnMut(Duration),
) -> Result<(), FfmpegError> {
    // Progress goes to stdout as key=value lines; stderr only carries errors.
    let mut child = Command::new("ffmpeg")
        .args(["-hide_banner", "-loglevel", "error", "-nostats"])
        .args(["-progress", "pipe:1", "-i"])
        .arg(src)
        .args(["-vf", &format!("scale=-2:{height}")])
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

    let stdout = child.stdout.take().expect("stdout is piped");
    for line in BufReader::new(stdout).lines().map_while(Result::ok) {
        // out_time_us is "N/A" until the first frame is written.
        if let Some(us) = line
            .strip_prefix("out_time_us=")
            .and_then(|v| v.parse::<u64>().ok())
        {
            on_progress(Duration::from_micros(us));
        }
    }

    let status = child.wait().map_err(|source| FfmpegError::Spawn {
        program: "ffmpeg",
        source,
    })?;
    let stderr = stderr_reader.join().unwrap_or_default();
    if status.success() {
        Ok(())
    } else {
        Err(FfmpegError::Failed {
            program: "ffmpeg",
            status,
            stderr: stderr.trim().to_string(),
        })
    }
}
