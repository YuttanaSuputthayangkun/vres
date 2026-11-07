use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::error::FileError;

/// Why a source video was not converted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkipReason {
    /// The file name matches the output format at the target height, so it is
    /// most likely a previous output.
    MatchesOutputFormat,
    /// The source is shorter than the target height; videos are never upscaled.
    TooShort { height: u32, target: u32 },
    /// The output file already exists and overwriting is off.
    OutputExists,
}

/// A video that was converted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Converted {
    pub src: PathBuf,
    pub dst: PathBuf,
}

/// A video that was not converted. `dst` is where its output would have gone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Skipped {
    pub src: PathBuf,
    pub dst: PathBuf,
    pub reason: SkipReason,
}

/// A video that failed, either before or during its conversion. `dst` is
/// where its output would have gone.
#[derive(Debug)]
pub struct Failed {
    pub src: PathBuf,
    pub dst: PathBuf,
    pub error: FileError,
}

/// A conversion that is about to start.
#[derive(Debug, Clone, Copy)]
pub struct Job<'a> {
    /// Zero-based position among the videos being converted.
    pub index: usize,
    /// Number of videos being converted.
    pub total: usize,
    pub src: &'a Path,
    pub dst: &'a Path,
    /// Source duration, if ffprobe reports one.
    pub duration: Option<Duration>,
}

/// Receives events while [`Resizer::run`](crate::Resizer::run) works through
/// the videos. Every method does nothing by default; `()` ignores everything.
pub trait Reporter {
    /// A video will not be converted.
    fn skipped(&mut self, _skipped: &Skipped) {}
    /// A conversion is starting.
    fn started(&mut self, _job: &Job<'_>) {}
    /// The current conversion has written `position` of output.
    fn progress(&mut self, _position: Duration) {}
    /// The current conversion finished successfully.
    fn finished(&mut self, _converted: &Converted) {}
    /// A video failed.
    fn failed(&mut self, _failed: &Failed) {}
}

impl Reporter for () {}
