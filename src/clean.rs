//! `vres clean`: move source videos whose output exists to the Recycle Bin.

use std::io::{self, BufRead, IsTerminal, Write};
use std::process::ExitCode;

use serde_json::{Value, json};
use vres::{Converted, TrashFailed};

use crate::cli::CleanArgs;
use crate::console::{self, Console, path};

pub fn run(args: &CleanArgs) -> ExitCode {
    let common = &args.common;
    let console = Console::new(common.quiet, common.json);

    let resizer = match args.resizer() {
        Ok(resizer) => resizer,
        Err(e) => {
            console::error(&e);
            return ExitCode::FAILURE;
        }
    };
    if resizer.height() != common.height {
        console.height_note(common.height, resizer.height());
    }

    let found = match resizer.sources_with_outputs(&common.inputs) {
        Ok(found) => found,
        Err(e) => {
            console::error(&e);
            return ExitCode::FAILURE;
        }
    };

    if found.is_empty() {
        if common.json {
            print_json(&[], &[]);
        } else if !common.quiet {
            println!("nothing to clean");
        }
        return ExitCode::SUCCESS;
    }

    if !args.yes {
        if !io::stdin().is_terminal() {
            console::error(&"refusing to delete without confirmation; pass --yes");
            return ExitCode::FAILURE;
        }
        // The user must see what they are confirming, even with --quiet.
        list(&found);
        match confirm(found.len()) {
            Ok(true) => {}
            Ok(false) => {
                eprintln!("cancelled");
                return ExitCode::SUCCESS;
            }
            Err(e) => {
                console::error(&format_args!("cannot read confirmation: {e}"));
                return ExitCode::FAILURE;
            }
        }
    }

    let summary = vres::trash_sources(found);
    if !common.quiet {
        for item in &summary.trashed {
            status(common.json, format_args!("trashed {}", item.src.display()));
        }
    }
    for failed in &summary.failed {
        eprintln!("failed {}: {}", failed.src.display(), failed.error);
    }

    if common.json {
        print_json(&summary.trashed, &summary.failed);
    } else if !common.quiet {
        println!(
            "trashed: {}, failed: {}",
            summary.trashed.len(),
            summary.failed.len()
        );
    }
    if summary.is_success() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

/// Prints the files that will be trashed to stderr, next to the prompt.
fn list(found: &[Converted]) {
    for item in found {
        eprintln!("{} (output: {})", item.src.display(), item.dst.display());
    }
}

/// Asks on stderr and reads a yes/no answer from stdin; anything but y/yes is no.
fn confirm(count: usize) -> io::Result<bool> {
    let files = if count == 1 { "file" } else { "files" };
    eprint!("Move {count} {files} to the Recycle Bin? [y/N] ");
    io::stderr().flush()?;
    let mut answer = String::new();
    io::stdin().lock().read_line(&mut answer)?;
    let answer = answer.trim();
    Ok(answer.eq_ignore_ascii_case("y") || answer.eq_ignore_ascii_case("yes"))
}

fn status(to_stderr: bool, args: std::fmt::Arguments) {
    if to_stderr {
        eprintln!("{args}");
    } else {
        println!("{args}");
    }
}

fn print_json(trashed: &[Converted], failed: &[TrashFailed]) {
    let trashed: Vec<Value> = trashed
        .iter()
        .map(|c| json!({ "src": path(&c.src), "dst": path(&c.dst) }))
        .collect();
    let failed: Vec<Value> = failed
        .iter()
        .map(|f| {
            json!({
                "src": path(&f.src),
                "dst": path(&f.dst),
                "error": f.error.to_string(),
            })
        })
        .collect();
    println!("{:#}", json!({ "trashed": trashed, "failed": failed }));
}
