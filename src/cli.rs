use std::path::PathBuf;

use clap::Parser;

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
    #[arg(short, long, env = "VRES_WIDTH", default_value_t = 720)]
    pub width: u32,

    /// Output frame rate
    #[arg(short = 'r', long, env = "VRES_FPS", default_value_t = 30,
          value_parser = clap::value_parser!(u32).range(1..))]
    pub fps: u32,

    /// Output directory [default: same directory as each source]
    #[arg(short, long, env = "VRES_OUTPUT")]
    pub output: Option<PathBuf>,

    /// Output file name format; supports {source_name} and {width}
    #[arg(
        short,
        long,
        env = "VRES_FORMAT",
        default_value = "{source_name}_{width}"
    )]
    pub format: String,

    /// Scan directories recursively
    #[arg(short = 'R', long, env = "VRES_RECURSIVE")]
    pub recursive: bool,

    /// Overwrite existing output files instead of skipping them
    #[arg(long, env = "VRES_OVERWRITE")]
    pub overwrite: bool,
}
