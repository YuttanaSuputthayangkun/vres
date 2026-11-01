mod cli;
mod error;
mod ffmpeg;
mod inputs;
mod output;
mod width;

use std::fs;
use std::path::Path;
use std::process::ExitCode;

use clap::Parser;

use cli::Cli;
use error::{FileError, RunError};

fn main() -> ExitCode {
    // Load .env before parsing so clap can read VRES_* variables from it.
    dotenvy::dotenv().ok();
    let cli = Cli::parse();

    match run(&cli) {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

/// Returns Ok(true) if every file succeeded or was skipped.
fn run(cli: &Cli) -> Result<bool, RunError> {
    ffmpeg::check_available().map_err(RunError::FfmpegUnavailable)?;

    let width = width::normalize_width(cli.width);
    if width != cli.width {
        eprintln!("note: width {} is not valid, using {width}", cli.width);
    }

    let mut videos = inputs::collect_videos(&cli.inputs, cli.recursive)?;
    if videos.is_empty() {
        return Err(RunError::NoVideos);
    }

    let mut skipped = 0;
    if !cli.include_outputs {
        let pattern =
            output::output_pattern(&cli.format, width).map_err(RunError::OutputPattern)?;
        videos.retain(|src| {
            let ignore = output::is_output(src, &pattern);
            if ignore {
                println!("skip {}: matches output format", src.display());
                skipped += 1;
            }
            !ignore
        });
    }

    if let Some(dir) = &cli.output {
        fs::create_dir_all(dir).map_err(|source| RunError::CreateOutputDir {
            path: dir.clone(),
            source,
        })?;
    }

    let (mut done, mut failed) = (0, 0);
    for (i, src) in videos.iter().enumerate() {
        let counter = format!("{}/{}", i + 1, videos.len());
        match process_file(cli, src, width, &counter) {
            Ok(true) => done += 1,
            Ok(false) => skipped += 1,
            Err(e) => {
                eprintln!("failed {}: {e}", src.display());
                failed += 1;
            }
        }
    }

    println!("done: {done}, skipped: {skipped}, failed: {failed}");
    Ok(failed == 0)
}

/// Resizes one video. Returns Ok(true) if converted, Ok(false) if skipped.
/// `counter` (e.g. "2/5") is shown on the progress bar.
fn process_file(cli: &Cli, src: &Path, width: u32, counter: &str) -> Result<bool, FileError> {
    let dst = output::output_path(src, cli.output.as_deref(), &cli.format, width);
    if dst == src {
        return Err(FileError::SameAsSource);
    }

    let probe = ffmpeg::probe(src)?;
    if probe.width < width {
        println!(
            "skip {}: width {} is less than {width}",
            src.display(),
            probe.width
        );
        return Ok(false);
    }

    if dst.exists() && !cli.overwrite {
        println!(
            "skip {}: {} exists (use --overwrite)",
            src.display(),
            dst.display()
        );
        return Ok(false);
    }

    println!("{} -> {}", src.display(), dst.display());
    ffmpeg::resize(
        src,
        &dst,
        width,
        cli.fps,
        cli.overwrite,
        probe.duration,
        counter,
    )?;
    Ok(true)
}
