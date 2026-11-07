//! The `vyapaar` binary entrypoint.

use std::io::IsTerminal;
use std::process::ExitCode;
use std::sync::Arc;

use vyapaar_agent_llm::{CandleGgufConfig, CandleGgufProvider};
use vyapaar_cli::args::parse;
use vyapaar_cli::runner::{CliError, CommandOutcome, resolve_provider, run};
use vyapaar_tui::TuiApp;

fn main() -> ExitCode {
    let cli = parse();
    match run(&cli) {
        Ok(outcome) => {
            match outcome {
                CommandOutcome::LaunchedTui => {
                    if !std::io::stdout().is_terminal() {
                        println!("vyapaar: tui would launch");
                        return ExitCode::SUCCESS;
                    }

                    let provider = resolve_provider(cli.model_path.as_deref(), cli.temperature)
                        .ok()
                        .flatten()
                        .unwrap_or_else(|| {
                            Arc::new(CandleGgufProvider::new(CandleGgufConfig::default()))
                        });

                    let app = TuiApp::new().with_model_name(provider.model_name());
                    let rt = tokio::runtime::Runtime::new().expect("failed to start tokio runtime");

                    if let Err(e) = rt.block_on(vyapaar_tui::run_terminal(app, provider)) {
                        eprintln!("vyapaar: tui error: {e}");
                        return ExitCode::from(1);
                    }
                }
                CommandOutcome::Answer(answer) => {
                    if answer.is_empty() {
                        println!("(empty response)");
                    }
                }
                CommandOutcome::ChatCompleted => {}
                CommandOutcome::ScanResults { rows } => {
                    println!("vyapaar: scan returned {rows} rows")
                }
                CommandOutcome::JobsSuccess(msg) => println!("{msg}"),
                CommandOutcome::VerifySuccess { message, .. } => println!("{message}"),
                CommandOutcome::MemorySuccess(msg) => println!("{msg}"),
                CommandOutcome::PluginSuccess(msg) => println!("{msg}"),
                CommandOutcome::DoctorReport(report) => println!("{report}"),
                CommandOutcome::PurgeSuccess(msg) => println!("{msg}"),
                CommandOutcome::HealthReport(report) => println!("{report}"),
            }
            ExitCode::SUCCESS
        }
        Err(err) => match &err {
            CliError::MissingFile(msg) => {
                eprintln!("vyapaar: file error: {msg}");
                ExitCode::from(2)
            }
            CliError::VerificationFailed(msg) => {
                eprintln!("vyapaar: verification failed: {msg}");
                ExitCode::from(1)
            }
            other => {
                eprintln!("vyapaar: error: {other}");
                ExitCode::from(1)
            }
        },
    }
}
