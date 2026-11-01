use std::fmt::Display;
use std::time::Duration;

use indicatif::{ProgressBar, ProgressStyle};
use vres::{Converted, Failed, Job, Reporter, SkipReason, Skipped, Summary};

/// Prints run events to the terminal, with a progress bar per conversion.
#[derive(Default)]
pub struct Console {
    bar: Option<ProgressBar>,
}

impl Reporter for Console {
    fn skipped(&mut self, skipped: &Skipped) {
        let src = skipped.src.display();
        match &skipped.reason {
            SkipReason::MatchesOutputFormat => println!("skip {src}: matches output format"),
            SkipReason::TooNarrow { width, target } => {
                println!("skip {src}: width {width} is less than {target}")
            }
            SkipReason::OutputExists => {
                println!(
                    "skip {src}: {} exists (use --overwrite)",
                    skipped.dst.display()
                )
            }
        }
    }

    fn started(&mut self, job: &Job<'_>) {
        println!("{} -> {}", job.src.display(), job.dst.display());
        let bar = progress_bar(job.duration);
        bar.set_prefix(format!("{}/{}", job.index + 1, job.total));
        self.bar = Some(bar);
    }

    fn progress(&mut self, position: Duration) {
        if let Some(bar) = &self.bar {
            let ms = position.as_millis() as u64;
            bar.set_position(ms.min(bar.length().unwrap_or(u64::MAX)));
        }
    }

    fn finished(&mut self, _converted: &Converted) {
        if let Some(bar) = self.bar.take() {
            bar.finish();
        }
    }

    fn failed(&mut self, failed: &Failed) {
        if let Some(bar) = self.bar.take() {
            bar.abandon();
        }
        eprintln!("failed {}: {}", failed.src.display(), failed.error);
    }
}

pub fn width_note(requested: u32, used: u32) {
    eprintln!("note: width {requested} is not valid, using {used}");
}

pub fn summary(summary: &Summary) {
    println!(
        "done: {}, skipped: {}, failed: {}",
        summary.converted.len(),
        summary.skipped.len(),
        summary.failed.len()
    );
    if !summary.failed.is_empty() {
        eprintln!("failed files:");
        for failed in &summary.failed {
            eprintln!("  {}: {}", failed.src.display(), failed.error);
        }
    }
}

pub fn error(error: &impl Display) {
    eprintln!("error: {error}");
}

/// A bar measured in milliseconds of output when `duration` is known, else a spinner.
fn progress_bar(duration: Option<Duration>) -> ProgressBar {
    let bar = match duration {
        Some(d) => ProgressBar::new(d.as_millis() as u64).with_style(
            ProgressStyle::with_template(
                "  [{prefix}] [{elapsed_precise}] [{wide_bar:.cyan/blue}] {percent:>3}%  ETA {eta_precise}",
            )
            .expect("valid template")
            .progress_chars("=> "),
        ),
        None => ProgressBar::new_spinner().with_style(
            ProgressStyle::with_template("  [{prefix}] {spinner} [{elapsed_precise}] converting...")
                .expect("valid template"),
        ),
    };
    bar.enable_steady_tick(Duration::from_millis(200));
    bar
}
