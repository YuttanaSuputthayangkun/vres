//! Resize videos to a given height with ffmpeg, preserving aspect ratio.
//!
//! Requires `ffmpeg` and `ffprobe` on `PATH`.
//!
//! ```no_run
//! use vres::Resizer;
//!
//! let resizer = Resizer::builder().height(1080).fps(24).recursive(true).build()?;
//! let summary = resizer.run(&["videos/"], &mut ())?;
//! for converted in &summary.converted {
//!     println!("{} -> {}", converted.src.display(), converted.dst.display());
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! Pass your own [`Reporter`] instead of `&mut ()` to observe skips, progress
//! and failures.

mod error;
mod ffmpeg;
mod height;
mod inputs;
mod output;
mod report;
mod resizer;

pub use error::{BuildError, FfmpegError, FileError, InputError, RunError};
pub use report::{Converted, Failed, Job, Reporter, SkipReason, Skipped};
pub use resizer::{DEFAULT_FORMAT, DEFAULT_FPS, DEFAULT_HEIGHT, Resizer, ResizerBuilder, Summary};
