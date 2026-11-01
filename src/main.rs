mod cli;
mod console;

use std::process::ExitCode;

use clap::Parser;

use cli::Cli;
use console::Console;

fn main() -> ExitCode {
    // Load .env before parsing so clap can read VRES_* variables from it.
    dotenvy::dotenv().ok();
    let cli = Cli::parse();

    let resizer = match cli.resizer() {
        Ok(resizer) => resizer,
        Err(e) => {
            console::error(&e);
            return ExitCode::FAILURE;
        }
    };
    if resizer.width() != cli.width {
        console::width_note(cli.width, resizer.width());
    }

    match resizer.run(&cli.inputs, &mut Console::default()) {
        Ok(summary) => {
            console::summary(&summary);
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
