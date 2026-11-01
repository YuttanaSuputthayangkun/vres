use std::fs;
use std::path::{Path, PathBuf};

use regex::Regex;

use crate::error::{BuildError, FileError, RunError};
use crate::report::{Converted, Failed, Job, Reporter, SkipReason, Skipped};
use crate::{ffmpeg, inputs, output, width};

pub const DEFAULT_WIDTH: u32 = 720;
pub const DEFAULT_FPS: u32 = 30;
pub const DEFAULT_FORMAT: &str = "{source_name}_{width}";

/// Resizes videos to a fixed width with ffmpeg, preserving aspect ratio.
///
/// Create one with [`Resizer::builder`].
#[derive(Debug, Clone)]
pub struct Resizer {
    width: u32,
    fps: u32,
    output_dir: Option<PathBuf>,
    format: String,
    recursive: bool,
    include_outputs: bool,
    overwrite: bool,
    output_pattern: Regex,
}

/// What happened to each video in a run, in processing order within each list.
#[derive(Debug, Default)]
pub struct Summary {
    pub converted: Vec<Converted>,
    pub skipped: Vec<Skipped>,
    pub failed: Vec<Failed>,
}

impl Summary {
    /// True if every video was converted or skipped.
    pub fn is_success(&self) -> bool {
        self.failed.is_empty()
    }
}

impl Resizer {
    pub fn builder() -> ResizerBuilder {
        ResizerBuilder::default()
    }

    /// The target width, after being made valid (even and at least 2).
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Resizes every video in `inputs` (files and/or directories).
    ///
    /// Problems with a single video are passed to `reporter` and recorded in the
    /// [`Summary`]; only problems that stop the whole batch return `Err`.
    pub fn run<P, R>(&self, inputs: &[P], reporter: &mut R) -> Result<Summary, RunError>
    where
        P: AsRef<Path>,
        R: Reporter + ?Sized,
    {
        ffmpeg::check_available().map_err(RunError::FfmpegUnavailable)?;

        let videos = inputs::collect_videos(inputs, self.recursive)?;
        if videos.is_empty() {
            return Err(RunError::NoVideos);
        }

        let mut summary = Summary::default();
        let mut queue = Vec::with_capacity(videos.len());
        for src in videos {
            let dst = self.output_path(&src);
            if !self.include_outputs && output::is_output(&src, &self.output_pattern) {
                let skipped = Skipped {
                    src,
                    dst,
                    reason: SkipReason::MatchesOutputFormat,
                };
                reporter.skipped(&skipped);
                summary.skipped.push(skipped);
            } else {
                queue.push((src, dst));
            }
        }

        if let Some(dir) = &self.output_dir {
            fs::create_dir_all(dir).map_err(|source| RunError::CreateOutputDir {
                path: dir.clone(),
                source,
            })?;
        }

        let total = queue.len();
        for (index, (src, dst)) in queue.into_iter().enumerate() {
            match self.process_file(&src, &dst, index, total, reporter) {
                Ok(None) => {
                    let converted = Converted { src, dst };
                    reporter.finished(&converted);
                    summary.converted.push(converted);
                }
                Ok(Some(reason)) => {
                    let skipped = Skipped { src, dst, reason };
                    reporter.skipped(&skipped);
                    summary.skipped.push(skipped);
                }
                Err(error) => {
                    let failed = Failed { src, dst, error };
                    reporter.failed(&failed);
                    summary.failed.push(failed);
                }
            }
        }
        Ok(summary)
    }

    fn output_path(&self, src: &Path) -> PathBuf {
        output::output_path(src, self.output_dir.as_deref(), &self.format, self.width)
    }

    /// Resizes one video. Returns the reason if it was skipped.
    fn process_file<R: Reporter + ?Sized>(
        &self,
        src: &Path,
        dst: &Path,
        index: usize,
        total: usize,
        reporter: &mut R,
    ) -> Result<Option<SkipReason>, FileError> {
        if dst == src {
            return Err(FileError::SameAsSource);
        }

        let probe = ffmpeg::probe(src)?;
        if probe.width < self.width {
            return Ok(Some(SkipReason::TooNarrow {
                width: probe.width,
                target: self.width,
            }));
        }

        if dst.exists() && !self.overwrite {
            return Ok(Some(SkipReason::OutputExists));
        }

        reporter.started(&Job {
            index,
            total,
            src,
            dst,
            duration: probe.duration,
        });
        ffmpeg::resize(
            src,
            dst,
            self.width,
            self.fps,
            self.overwrite,
            &mut |position| reporter.progress(position),
        )?;
        Ok(None)
    }
}

/// Configures a [`Resizer`]. Unset options use the `DEFAULT_*` constants,
/// write next to each source, scan only the top level of directories, skip
/// inputs that look like previous outputs, and never overwrite.
#[derive(Debug, Clone)]
pub struct ResizerBuilder {
    width: u32,
    fps: u32,
    output_dir: Option<PathBuf>,
    format: String,
    recursive: bool,
    include_outputs: bool,
    overwrite: bool,
}

impl Default for ResizerBuilder {
    fn default() -> Self {
        Self {
            width: DEFAULT_WIDTH,
            fps: DEFAULT_FPS,
            output_dir: None,
            format: DEFAULT_FORMAT.to_string(),
            recursive: false,
            include_outputs: false,
            overwrite: false,
        }
    }
}

impl ResizerBuilder {
    /// Target width in pixels. Odd values are bumped to the next even number;
    /// check [`Resizer::width`] for the value used.
    pub fn width(mut self, width: u32) -> Self {
        self.width = width;
        self
    }

    /// Output frame rate. Must be at least 1.
    pub fn fps(mut self, fps: u32) -> Self {
        self.fps = fps;
        self
    }

    /// Directory for output files, created if missing.
    pub fn output_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.output_dir = Some(dir.into());
        self
    }

    /// Output file name format; supports `{source_name}` and `{width}`.
    /// The source extension is kept.
    pub fn format(mut self, format: impl Into<String>) -> Self {
        self.format = format.into();
        self
    }

    /// Scan directories recursively.
    pub fn recursive(mut self, recursive: bool) -> Self {
        self.recursive = recursive;
        self
    }

    /// Also process inputs whose name matches the output format at the
    /// target width (e.g. `clip_720.mp4`).
    pub fn include_outputs(mut self, include_outputs: bool) -> Self {
        self.include_outputs = include_outputs;
        self
    }

    /// Overwrite existing output files instead of skipping them.
    pub fn overwrite(mut self, overwrite: bool) -> Self {
        self.overwrite = overwrite;
        self
    }

    pub fn build(self) -> Result<Resizer, BuildError> {
        if self.fps == 0 {
            return Err(BuildError::ZeroFps);
        }
        let width = width::normalize_width(self.width);
        let output_pattern =
            output::output_pattern(&self.format, width).map_err(BuildError::OutputPattern)?;
        Ok(Resizer {
            width,
            fps: self.fps,
            output_dir: self.output_dir,
            format: self.format,
            recursive: self.recursive,
            include_outputs: self.include_outputs,
            overwrite: self.overwrite,
            output_pattern,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_with_defaults() {
        let resizer = Resizer::builder().build().unwrap();
        assert_eq!(resizer.width(), DEFAULT_WIDTH);
        assert_eq!(resizer.fps, DEFAULT_FPS);
        assert_eq!(resizer.format, DEFAULT_FORMAT);
    }

    #[test]
    fn normalizes_width() {
        let resizer = Resizer::builder().width(853).build().unwrap();
        assert_eq!(resizer.width(), 854);
    }

    #[test]
    fn rejects_zero_fps() {
        assert!(matches!(
            Resizer::builder().fps(0).build(),
            Err(BuildError::ZeroFps)
        ));
    }
}
