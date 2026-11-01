//! Resize videos to a given width with ffmpeg, preserving aspect ratio.
//!
//! Requires `ffmpeg` and `ffprobe` on `PATH`.
//!
//! ```no_run
//! use vres::Resizer;
//!
//! let resizer = Resizer::builder().width(1280).fps(24).recursive(true).build()?;
//! let summary = resizer.run(&["videos/"], &mut ())?;
//! println!("{} converted", summary.done);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! Pass your own [`Reporter`] instead of `&mut ()` to observe skips, progress
//! and failures.

mod error;
mod ffmpeg;
mod inputs;
mod output;
mod report;
mod resizer;
mod width;

pub use error::{BuildError, FfmpegError, FileError, InputError, RunError};
pub use report::{Job, Reporter, SkipReason};
pub use resizer::{DEFAULT_FORMAT, DEFAULT_FPS, DEFAULT_WIDTH, Resizer, ResizerBuilder, Summary};
