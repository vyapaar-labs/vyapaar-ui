//! Unit tests for the Vyapaar CLI.

use vyapaar_cli::args::{AskArgs, CliArgs, Command, ScanArgs};
use vyapaar_cli::runner::{CliError, CommandOutcome, run};

#[test]
fn parse_cli_roundtrip_tui() {
    let cli = vyapaar_cli::args::Cli {
        command: vyapaar_cli::args::CliCommand::Tui,
    };
    let args: CliArgs = cli.into();
    assert_eq!(args.command, Command::Tui);
}

#[test]
fn parse_cli_roundtrip_ask() {
    let cli = vyapaar_cli::args::Cli {
        command: vyapaar_cli::args::CliCommand::Ask {
            prompt: "what is NIFTY?".into(),
        },
    };
    let args: CliArgs = cli.into();
    assert_eq!(
        args.command,
        Command::Ask(AskArgs {
            prompt: "what is NIFTY?".into()
        })
    );
}

#[test]
fn parse_cli_roundtrip_scan() {
    let cli = vyapaar_cli::args::Cli {
        command: vyapaar_cli::args::CliCommand::Scan {
            query: "roe>15".into(),
        },
    };
    let args: CliArgs = cli.into();
    assert_eq!(
        args.command,
        Command::Scan(ScanArgs {
            query: "roe>15".into()
        })
    );
}

#[test]
fn run_tui_reports_launch() {
    let cli = CliArgs {
        command: Command::Tui,
    };
    assert_eq!(run(&cli).unwrap(), CommandOutcome::LaunchedTui);
}

#[test]
fn run_ask_fails_with_not_wired() {
    let cli = CliArgs {
        command: Command::Ask(AskArgs {
            prompt: "hello".into(),
        }),
    };
    assert_eq!(run(&cli), Err(CliError::NotWired("vyapaar ask".into())));
}

#[test]
fn run_scan_fails_with_not_wired() {
    let cli = CliArgs {
        command: Command::Scan(ScanArgs {
            query: "pe<20".into(),
        }),
    };
    assert_eq!(run(&cli), Err(CliError::NotWired("vyapaar scan".into())));
}

#[test]
fn run_health_reports_environment() {
    let cli = CliArgs {
        command: Command::Health,
    };
    let outcome = run(&cli).unwrap();
    match outcome {
        CommandOutcome::HealthReport(report) => {
            assert!(report.contains("vyapaar"));
            assert!(report.contains("tui ready"));
        }
        other => panic!("expected health report, got {other:?}"),
    }
}

#[test]
fn run_ask_empty_prompt_errors() {
    let cli = CliArgs {
        command: Command::Ask(AskArgs {
            prompt: "  ".into(),
        }),
    };
    assert!(run(&cli).is_err());
}

#[test]
fn error_display_is_human_readable() {
    let err = CliError::NotWired("vyapaar ask".into());
    assert_eq!(
        err.to_string(),
        "`vyapaar ask` is not wired to a backend yet"
    );
}
