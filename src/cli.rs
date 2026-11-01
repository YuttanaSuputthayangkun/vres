use std::path::PathBuf;

use clap::Parser;
use vres::{BuildError, Resizer};

/// Resize videos to a given width with ffmpeg, preserving aspect ratio.
///
/// Options can also be set via environment variables or a `.env` file.
#[derive(Debug, Parser)]
#[command(version, about)]
pub struct Cli {
    /// Video files and/or directories to process
    #[arg(required = true)]
    pub inputs: Vec<PathBuf>,

    /// Target width in pixels (odd values are bumped to the next even number)
    #[arg(short, long, env = "VRES_WIDTH", default_value_t = vres::DEFAULT_WIDTH)]
    pub width: u32,

    /// Output frame rate
    #[arg(short = 'r', long, env = "VRES_FPS", default_value_t = vres::DEFAULT_FPS,
          value_parser = clap::value_parser!(u32).range(1..))]
    pub fps: u32,

    /// Output directory [default: same directory as each source]
    #[arg(short, long, env = "VRES_OUTPUT")]
    pub output: Option<PathBuf>,

    /// Output file name format; supports {source_name} and {width}
    #[arg(short, long, env = "VRES_FORMAT", default_value = vres::DEFAULT_FORMAT)]
    pub format: String,

    /// Scan directories recursively
    #[arg(short = 'R', long, env = "VRES_RECURSIVE")]
    pub recursive: bool,

    /// Also process input files whose name matches the output format at the
    /// current width (e.g. clip_720.mp4); these are ignored by default
    #[arg(long, env = "VRES_INCLUDE_OUTPUTS")]
    pub include_outputs: bool,

    /// Overwrite existing output files instead of skipping them
    #[arg(long, env = "VRES_OVERWRITE")]
    pub overwrite: bool,
}

impl Cli {
    /// Builds a [`Resizer`] from the parsed options.
    pub fn resizer(&self) -> Result<Resizer, BuildError> {
        let mut builder = Resizer::builder()
            .width(self.width)
            .fps(self.fps)
            .format(&self.format)
            .recursive(self.recursive)
            .include_outputs(self.include_outputs)
            .overwrite(self.overwrite);
        if let Some(dir) = &self.output {
            builder = builder.output_dir(dir);
        }
        builder.build()
    }
}
