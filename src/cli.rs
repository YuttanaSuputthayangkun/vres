use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};
use vres::{BuildError, Resizer, ResizerBuilder};

/// Resize videos to a given height with ffmpeg, preserving aspect ratio.
///
/// Options can also be set via environment variables or a `.env` file.
#[derive(Debug, Parser)]
#[command(
    version,
    about,
    args_conflicts_with_subcommands = true,
    subcommand_negates_reqs = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,

    #[command(flatten)]
    pub resize: ResizeArgs,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Move source videos whose output already exists to the Recycle Bin.
    ///
    /// Use the same --height, --format and --output as the resize run so the
    /// outputs are found. Asks for confirmation unless --yes is given.
    Clean(CleanArgs),
}

/// Options shared by resizing and `clean`.
#[derive(Debug, Args)]
pub struct CommonArgs {
    /// Video files and/or directories to process
    #[arg(required = true)]
    pub inputs: Vec<PathBuf>,

    /// Target height in pixels (odd values are bumped to the next even number)
    #[arg(short = 'H', long, env = "VRES_HEIGHT", default_value_t = vres::DEFAULT_HEIGHT)]
    pub height: u32,

    /// Output directory [default: same directory as each source]
    #[arg(short, long, env = "VRES_OUTPUT")]
    pub output: Option<PathBuf>,

    /// Output file name format; supports {source_name} and {height}
    #[arg(short, long, env = "VRES_FORMAT", default_value = vres::DEFAULT_FORMAT)]
    pub format: String,

    /// Scan directories recursively
    #[arg(short = 'R', long, env = "VRES_RECURSIVE")]
    pub recursive: bool,

    /// Only print errors
    #[arg(short, long, env = "VRES_QUIET")]
    pub quiet: bool,

    /// Print a JSON summary to stdout when done; status lines go to stderr
    #[arg(long, env = "VRES_JSON")]
    pub json: bool,
}

impl CommonArgs {
    fn builder(&self) -> ResizerBuilder {
        let mut builder = Resizer::builder()
            .height(self.height)
            .format(&self.format)
            .recursive(self.recursive);
        if let Some(dir) = &self.output {
            builder = builder.output_dir(dir);
        }
        builder
    }
}

#[derive(Debug, Args)]
pub struct ResizeArgs {
    #[command(flatten)]
    pub common: CommonArgs,

    /// Output frame rate
    #[arg(short = 'r', long, env = "VRES_FPS", default_value_t = vres::DEFAULT_FPS,
          value_parser = clap::value_parser!(u32).range(1..))]
    pub fps: u32,

    /// Also process input files whose name matches the output format at the
    /// current height (e.g. clip_720.mp4); these are ignored by default
    #[arg(long, env = "VRES_INCLUDE_OUTPUTS")]
    pub include_outputs: bool,

    /// Overwrite existing output files instead of skipping them
    #[arg(long, env = "VRES_OVERWRITE")]
    pub overwrite: bool,
}

impl ResizeArgs {
    /// Builds a [`Resizer`] from the parsed options.
    pub fn resizer(&self) -> Result<Resizer, BuildError> {
        self.common
            .builder()
            .fps(self.fps)
            .include_outputs(self.include_outputs)
            .overwrite(self.overwrite)
            .build()
    }
}

#[derive(Debug, Args)]
pub struct CleanArgs {
    #[command(flatten)]
    pub common: CommonArgs,

    /// Skip the confirmation prompt (required when stdin is not a terminal)
    #[arg(short, long)]
    pub yes: bool,
}

impl CleanArgs {
    /// Builds a [`Resizer`] used to locate outputs.
    pub fn resizer(&self) -> Result<Resizer, BuildError> {
        self.common.builder().build()
    }
}
