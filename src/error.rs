use std::fmt;
use std::io;
use std::path::PathBuf;
use std::process::ExitStatus;

/// Errors from running ffmpeg or ffprobe.
#[derive(Debug)]
pub enum FfmpegError {
    /// The program could not be started (e.g. not on PATH).
    Spawn {
        program: &'static str,
        source: io::Error,
    },
    /// The program ran but exited unsuccessfully.
    Failed {
        program: &'static str,
        status: ExitStatus,
        stderr: String,
    },
    /// ffprobe output could not be parsed.
    InvalidProbeOutput(String),
}

impl fmt::Display for FfmpegError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Spawn { program, source } => {
                write!(f, "failed to run {program} (is it on PATH?): {source}")
            }
            Self::Failed {
                program,
                status,
                stderr,
            } => {
                write!(f, "{program} exited with {status}")?;
                if !stderr.is_empty() {
                    write!(f, ": {stderr}")?;
                }
                Ok(())
            }
            Self::InvalidProbeOutput(output) => {
                write!(f, "could not parse ffprobe output {output:?}")
            }
        }
    }
}

impl std::error::Error for FfmpegError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Spawn { source, .. } => Some(source),
            _ => None,
        }
    }
}

/// Errors from expanding the input paths into video files.
#[derive(Debug)]
pub enum InputError {
    /// An input path does not exist.
    NotFound(PathBuf),
    /// A directory could not be read.
    ReadDir { path: PathBuf, source: io::Error },
}

impl fmt::Display for InputError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound(path) => write!(f, "input not found: {}", path.display()),
            Self::ReadDir { path, source } => {
                write!(f, "cannot read {}: {source}", path.display())
            }
        }
    }
}

impl std::error::Error for InputError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::NotFound(_) => None,
            Self::ReadDir { source, .. } => Some(source),
        }
    }
}

/// Errors that fail a single video without stopping the batch.
#[derive(Debug)]
pub enum FileError {
    /// The output path would overwrite the source file.
    SameAsSource,
    /// Probing or resizing failed.
    Ffmpeg(FfmpegError),
}

impl fmt::Display for FileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SameAsSource => write!(f, "output path equals source"),
            Self::Ffmpeg(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for FileError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::SameAsSource => None,
            Self::Ffmpeg(e) => Some(e),
        }
    }
}

impl From<FfmpegError> for FileError {
    fn from(e: FfmpegError) -> Self {
        Self::Ffmpeg(e)
    }
}

/// Errors that stop [`Resizer::run`](crate::Resizer::run) before or while setting up the batch.
#[derive(Debug)]
pub enum RunError {
    /// ffmpeg could not be executed.
    FfmpegUnavailable(FfmpegError),
    /// An input path was missing or a directory could not be read.
    Inputs(InputError),
    /// The inputs contained no video files.
    NoVideos,
    /// The output directory could not be created.
    CreateOutputDir { path: PathBuf, source: io::Error },
}

impl fmt::Display for RunError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FfmpegUnavailable(e) => write!(f, "{e}"),
            Self::Inputs(e) => write!(f, "{e}"),
            Self::NoVideos => write!(f, "no video files found"),
            Self::CreateOutputDir { path, source } => {
                write!(
                    f,
                    "cannot create output directory {}: {source}",
                    path.display()
                )
            }
        }
    }
}

impl std::error::Error for RunError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::FfmpegUnavailable(e) => Some(e),
            Self::Inputs(e) => Some(e),
            Self::NoVideos => None,
            Self::CreateOutputDir { source, .. } => Some(source),
        }
    }
}

impl From<InputError> for RunError {
    fn from(e: InputError) -> Self {
        Self::Inputs(e)
    }
}

/// A file could not be moved to the Recycle Bin by
/// [`trash_sources`](crate::trash_sources).
#[cfg(feature = "trash")]
#[derive(Debug)]
pub struct TrashError(pub(crate) trash::Error);

#[cfg(feature = "trash")]
impl fmt::Display for TrashError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(feature = "trash")]
impl std::error::Error for TrashError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}

/// Errors from [`ResizerBuilder::build`](crate::ResizerBuilder::build).
#[derive(Debug)]
pub enum BuildError {
    /// The frame rate was 0.
    ZeroFps,
    /// The output format could not be turned into a matching pattern.
    OutputPattern(regex::Error),
}

impl fmt::Display for BuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroFps => write!(f, "fps must be at least 1"),
            Self::OutputPattern(e) => write!(f, "invalid output format pattern: {e}"),
        }
    }
}

impl std::error::Error for BuildError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ZeroFps => None,
            Self::OutputPattern(e) => Some(e),
        }
    }
}
