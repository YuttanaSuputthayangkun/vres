mod clean;
mod cli;
mod console;

use std::process::ExitCode;

use clap::Parser;

use cli::{Cli, Command, ResizeArgs};
use console::Console;

fn main() -> ExitCode {
    // Load .env before parsing so clap can read VRES_* variables from it.
    dotenvy::dotenv().ok();
    let cli = Cli::parse();
    match &cli.command {
        Some(Command::Clean(args)) => clean::run(args),
        None => resize(&cli.resize),
    }
}

fn resize(args: &ResizeArgs) -> ExitCode {
    let common = &args.common;
    let mut console = Console::new(common.quiet, common.json);

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

    match resizer.run(&common.inputs, &mut console) {
        Ok(summary) => {
            console.summary(&summary);
            if summary.is_success() {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        Err(e) => {
            console::error(&e);
            ExitCode::FAILURE
        }
    }
}
