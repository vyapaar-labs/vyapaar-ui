//! Unit tests for the Vyapaar CLI.

use std::sync::Arc;

use vyapaar_agent_llm::ScriptedFakeLlm;
use vyapaar_cli::args::{AskArgs, ChatArgs, CliArgs, Command, ScanArgs};
use vyapaar_cli::runner::{CliError, CommandOutcome, run, run_with_provider};

#[test]
fn parse_cli_roundtrip_tui() {
    let cli = vyapaar_cli::args::Cli {
        command: vyapaar_cli::args::CliCommand::Tui,
        model_path: None,
        temperature: None,
    };
    let args: CliArgs = cli.into();
    assert_eq!(args.command, Command::Tui);
    assert_eq!(args.model_path, None);
}

#[test]
fn parse_cli_roundtrip_ask() {
    let cli = vyapaar_cli::args::Cli {
        command: vyapaar_cli::args::CliCommand::Ask {
            prompt: "what is NIFTY?".into(),
        },
        model_path: None,
        temperature: None,
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
fn parse_cli_roundtrip_chat() {
    let cli = vyapaar_cli::args::Cli {
        command: vyapaar_cli::args::CliCommand::Chat {
            system_prompt: Some("trade NSE only".into()),
        },
        model_path: None,
        temperature: None,
    };
    let args: CliArgs = cli.into();
    assert_eq!(
        args.command,
        Command::Chat(ChatArgs {
            system_prompt: Some("trade NSE only".into()),
        })
    );
}

#[test]
fn parse_cli_roundtrip_scan() {
    let cli = vyapaar_cli::args::Cli {
        command: vyapaar_cli::args::CliCommand::Scan {
            query: "roe>15".into(),
        },
        model_path: None,
        temperature: None,
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
        model_path: None,
        temperature: None,
    };
    assert_eq!(run(&cli).unwrap(), CommandOutcome::LaunchedTui);
}

#[test]
fn run_ask_fails_with_not_wired() {
    let cli = CliArgs {
        command: Command::Ask(AskArgs {
            prompt: "hello".into(),
        }),
        model_path: None,
        temperature: None,
    };
    assert_eq!(run(&cli), Err(CliError::NotWired("vyapaar ask".into())));
}

#[test]
fn run_ask_succeeds_with_injected_provider() {
    let fake = ScriptedFakeLlm::new("test-fake-llm");
    fake.push_text_response("NIFTY 50 is the benchmark index of the NSE.");

    let cli = CliArgs {
        command: Command::Ask(AskArgs {
            prompt: "what is NIFTY?".into(),
        }),
        model_path: None,
        temperature: None,
    };

    let outcome = run_with_provider(&cli, Some(Arc::new(fake))).unwrap();
    match outcome {
        CommandOutcome::Answer(ans) => {
            assert!(ans.contains("NIFTY 50 is the benchmark"));
        }
        other => panic!("expected Answer, got {other:?}"),
    }
}

#[test]
fn run_scan_fails_with_not_wired() {
    let cli = CliArgs {
        command: Command::Scan(ScanArgs {
            query: "pe<20".into(),
        }),
        model_path: None,
        temperature: None,
    };
    assert_eq!(run(&cli), Err(CliError::NotWired("vyapaar scan".into())));
}

#[test]
fn run_health_reports_environment() {
    let cli = CliArgs {
        command: Command::Health,
        model_path: None,
        temperature: None,
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
        model_path: None,
        temperature: None,
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
