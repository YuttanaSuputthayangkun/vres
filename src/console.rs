use std::fmt::{self, Display};
use std::path::Path;
use std::time::Duration;

use indicatif::{ProgressBar, ProgressStyle};
use serde_json::{Value, json};
use vres::{Converted, Failed, Job, Reporter, SkipReason, Skipped, Summary};

/// Prints run events to the terminal, with a progress bar per conversion.
///
/// `quiet` hides everything except errors. `json` prints the summary as JSON
/// on stdout and moves status lines to stderr so stdout stays parseable.
pub struct Console {
    quiet: bool,
    json: bool,
    bar: Option<ProgressBar>,
}

impl Console {
    pub fn new(quiet: bool, json: bool) -> Self {
        Self {
            quiet,
            json,
            bar: None,
        }
    }

    /// Prints a status line unless quiet.
    fn status(&self, args: fmt::Arguments) {
        if self.quiet {
            return;
        }
        if self.json {
            eprintln!("{args}");
        } else {
            println!("{args}");
        }
    }

    pub fn height_note(&self, requested: u32, used: u32) {
        if !self.quiet {
            eprintln!("note: height {requested} is not valid, using {used}");
        }
    }

    pub fn summary(&self, summary: &Summary) {
        if self.json {
            println!("{:#}", summary_json(summary));
            return;
        }
        if self.quiet {
            return;
        }
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
}

impl Reporter for Console {
    fn skipped(&mut self, skipped: &Skipped) {
        let src = skipped.src.display();
        match &skipped.reason {
            SkipReason::MatchesOutputFormat => {
                self.status(format_args!("skip {src}: matches output format"))
            }
            SkipReason::TooShort { height, target } => self.status(format_args!(
                "skip {src}: height {height} is less than {target}"
            )),
            SkipReason::OutputExists => self.status(format_args!(
                "skip {src}: {} exists (use --overwrite)",
                skipped.dst.display()
            )),
        }
    }

    fn started(&mut self, job: &Job<'_>) {
        if self.quiet {
            return;
        }
        self.status(format_args!(
            "{} -> {}",
            job.src.display(),
            job.dst.display()
        ));
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

pub fn error(error: &impl Display) {
    eprintln!("error: {error}");
}

fn summary_json(summary: &Summary) -> Value {
    let converted: Vec<Value> = summary
        .converted
        .iter()
        .map(|c| json!({ "src": path(&c.src), "dst": path(&c.dst) }))
        .collect();
    let skipped: Vec<Value> = summary
        .skipped
        .iter()
        .map(|s| {
            let mut value = json!({ "src": path(&s.src), "dst": path(&s.dst) });
            let reason = match &s.reason {
                SkipReason::MatchesOutputFormat => "matches_output_format",
                SkipReason::TooShort { height, target } => {
                    value["height"] = json!(height);
                    value["target"] = json!(target);
                    "too_short"
                }
                SkipReason::OutputExists => "output_exists",
            };
            value["reason"] = json!(reason);
            value
        })
        .collect();
    let failed: Vec<Value> = summary
        .failed
        .iter()
        .map(|f| {
            json!({
                "src": path(&f.src),
                "dst": path(&f.dst),
                "error": f.error.to_string(),
            })
        })
        .collect();
    json!({ "converted": converted, "skipped": skipped, "failed": failed })
}

/// Paths that are not valid Unicode are written lossily.
pub fn path(path: &Path) -> String {
    path.to_string_lossy().into_owned()
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
