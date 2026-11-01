use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::error::FileError;

/// Why a source video was not converted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkipReason {
    /// The file name matches the output format at the target width, so it is
    /// most likely a previous output.
    MatchesOutputFormat,
    /// The source is narrower than the target width; videos are never upscaled.
    TooNarrow { width: u32, target: u32 },
    /// The output file already exists and overwriting is off.
    OutputExists(PathBuf),
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
    /// `src` will not be converted.
    fn skipped(&mut self, _src: &Path, _reason: &SkipReason) {}
    /// A conversion is starting.
    fn started(&mut self, _job: &Job<'_>) {}
    /// The current conversion has written `position` of output.
    fn progress(&mut self, _position: Duration) {}
    /// The current conversion finished successfully.
    fn finished(&mut self, _src: &Path) {}
    /// `src` failed, either before or during its conversion.
    fn failed(&mut self, _src: &Path, _error: &FileError) {}
}

impl Reporter for () {}
