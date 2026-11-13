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
//!
//! [`Resizer::sources_with_outputs`] finds originals whose output exists. With
//! the `trash` feature, `trash_sources` moves them to the Recycle Bin.

mod error;
mod ffmpeg;
mod height;
mod inputs;
mod output;
#[cfg(feature = "trash")]
mod recycle;
mod report;
mod resizer;

#[cfg(feature = "trash")]
pub use error::TrashError;
pub use error::{BuildError, FfmpegError, FileError, InputError, RunError};
#[cfg(feature = "trash")]
pub use recycle::{TrashFailed, TrashSummary, trash_sources};
pub use report::{Converted, Failed, Job, Reporter, SkipReason, Skipped};
pub use resizer::{DEFAULT_FORMAT, DEFAULT_FPS, DEFAULT_HEIGHT, Resizer, ResizerBuilder, Summary};
