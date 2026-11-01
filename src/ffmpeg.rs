use std::path::Path;
use std::process::{Command, Stdio};

use crate::error::FfmpegError;

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

/// Returns the width in pixels of the first video stream of `src`, using ffprobe.
pub fn probe_width(src: &Path) -> Result<u32, FfmpegError> {
    let output = Command::new("ffprobe")
        .args(["-v", "error", "-select_streams", "v:0"])
        .args(["-show_entries", "stream=width", "-of", "csv=p=0"])
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
    let stdout = stdout.trim();
    stdout
        .parse()
        .map_err(|_| FfmpegError::InvalidWidth(stdout.to_string()))
}

/// Resizes `src` to `width` (height follows aspect ratio) at `fps`, writing `dst`.
pub fn resize(
    src: &Path,
    dst: &Path,
    width: u32,
    fps: u32,
    overwrite: bool,
) -> Result<(), FfmpegError> {
    // stderr is inherited so ffmpeg's progress and errors show in the terminal.
    let status = Command::new("ffmpeg")
        .args(["-hide_banner", "-loglevel", "error", "-stats", "-i"])
        .arg(src)
        .args(["-vf", &format!("scale={width}:-2")])
        .args(["-r", &fps.to_string()])
        .args(["-c:a", "copy"])
        .arg(if overwrite { "-y" } else { "-n" })
        .arg(dst)
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
