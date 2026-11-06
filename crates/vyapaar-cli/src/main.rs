//! The `vyapaar` binary entrypoint.

use std::process::ExitCode;

use vyapaar_cli::args::parse;
use vyapaar_cli::runner::{CommandOutcome, run};

fn main() -> ExitCode {
    let cli = parse();
    match run(&cli) {
        Ok(outcome) => {
            match outcome {
                CommandOutcome::LaunchedTui => {
                    // TODO(tier-5 follow-up): enter the real ratatui event loop
                    // (vyapaar_tui). Keeping the process alive here would block
                    // CI; the TUI is exercised through its own test suite.
                    println!("vyapaar: tui would launch");
                }
                CommandOutcome::Answer(answer) => println!("{answer}"),
                CommandOutcome::ScanResults { rows } => {
                    println!("vyapaar: scan returned {rows} rows")
                }
                CommandOutcome::HealthReport(report) => println!("{report}"),
            }
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("vyapaar: error: {err}");
            ExitCode::from(1)
        }
    }
}
