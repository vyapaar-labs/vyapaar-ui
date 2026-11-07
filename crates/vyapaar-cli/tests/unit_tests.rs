//! Unit tests for the Vyapaar CLI implementing the Quantcept command hierarchy.

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use vyapaar_agent_llm::ScriptedFakeLlm;
use vyapaar_cli::args::{
    AskArgs, ChatArgs, Cli, CliArgs, CliCommand, Command, DoctorArgs, JobsArgs, MemoryArgs,
    PluginArgs, PurgeArgs, ScanArgs, VerifyArgs,
};
use vyapaar_cli::runner::{CliError, CommandOutcome, run, run_with_provider};

#[test]
fn parse_cli_roundtrip_tui() {
    let cli = Cli {
        command: Some(CliCommand::Tui),
        ..Default::default()
    };
    let args: CliArgs = cli.into();
    assert_eq!(args.command, Command::Tui);
    assert_eq!(args.model_path, None);
}

#[test]
fn parse_cli_roundtrip_ask() {
    let cli = Cli {
        command: Some(CliCommand::Ask {
            prompt: "what is NIFTY?".into(),
            timeout_min: 20,
        }),
        ..Default::default()
    };
    let args: CliArgs = cli.into();
    assert_eq!(args.command, Command::Ask(AskArgs::new("what is NIFTY?")));
}

#[test]
fn parse_cli_roundtrip_chat() {
    let cli = Cli {
        command: Some(CliCommand::Chat {
            system_prompt: Some("trade NSE only".into()),
        }),
        ..Default::default()
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
    let cli = Cli {
        command: Some(CliCommand::Scan {
            query: "roe>15".into(),
        }),
        ..Default::default()
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
fn parse_cli_roundtrip_doctor_and_purge() {
    let doc_cli = Cli {
        command: Some(CliCommand::Doctor {
            fix: true,
            json: true,
        }),
        ..Default::default()
    };
    let doc_args: CliArgs = doc_cli.into();
    assert_eq!(
        doc_args.command,
        Command::Doctor(DoctorArgs {
            fix: true,
            json: true
        })
    );

    let purge_cli = Cli {
        command: Some(CliCommand::Purge {
            dry_run: true,
            yes: false,
        }),
        ..Default::default()
    };
    let purge_args: CliArgs = purge_cli.into();
    assert_eq!(
        purge_args.command,
        Command::Purge(PurgeArgs {
            dry_run: true,
            yes: false
        })
    );
}

#[test]
fn run_tui_reports_launch() {
    let cli = CliArgs {
        command: Command::Tui,
        ..Default::default()
    };
    assert_eq!(run(&cli).unwrap(), CommandOutcome::LaunchedTui);
}

#[test]
fn run_ask_fails_with_not_wired_without_provider() {
    let cli = CliArgs {
        command: Command::Ask(AskArgs::new("hello")),
        ..Default::default()
    };
    assert_eq!(run(&cli), Err(CliError::NotWired("vyapaar ask".into())));
}

#[test]
fn run_ask_succeeds_with_injected_provider() {
    let fake = ScriptedFakeLlm::new("test-fake-llm");
    fake.push_text_response("NIFTY 50 is the benchmark index of the NSE.");

    let cli = CliArgs {
        command: Command::Ask(AskArgs::new("what is NIFTY?")),
        ..Default::default()
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
        ..Default::default()
    };
    assert_eq!(run(&cli), Err(CliError::NotWired("vyapaar scan".into())));
}

#[test]
fn run_health_reports_environment() {
    let cli = CliArgs {
        command: Command::Health,
        ..Default::default()
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
fn run_doctor_reports_diagnostics() {
    let cli = CliArgs {
        command: Command::Doctor(DoctorArgs {
            fix: false,
            json: true,
        }),
        ..Default::default()
    };
    let outcome = run(&cli).unwrap();
    match outcome {
        CommandOutcome::DoctorReport(report) => {
            assert!(report.contains("healthy"));
            assert!(report.contains("SEBI pre-trade"));
        }
        other => panic!("expected doctor report, got {other:?}"),
    }
}

#[test]
fn run_purge_dry_run_lists_targets() {
    let cli = CliArgs {
        command: Command::Purge(PurgeArgs {
            dry_run: true,
            yes: false,
        }),
        ..Default::default()
    };
    let outcome = run(&cli).unwrap();
    match outcome {
        CommandOutcome::PurgeSuccess(msg) => {
            assert!(msg.contains("dry-run preview"));
        }
        other => panic!("expected purge preview, got {other:?}"),
    }
}

#[test]
fn run_purge_without_yes_refuses() {
    let cli = CliArgs {
        command: Command::Purge(PurgeArgs {
            dry_run: false,
            yes: false,
        }),
        ..Default::default()
    };
    assert!(run(&cli).is_err());
}

#[test]
fn run_jobs_lifecycle() {
    let add_cli = CliArgs {
        command: Command::Jobs(JobsArgs::Add {
            goal: "Scan momentum break in Nifty midcaps".into(),
            schedule: Some("0 9 * * 1-5".into()),
            max_turns: 20,
            max_inr: None,
            max_tokens: Some(4000),
            max_tool_calls: Some(10),
            missed_policy: vyapaar_cli::args::MissedPolicy::Skip,
            read_only: true,
            once: false,
        }),
        ..Default::default()
    };
    let outcome = run(&add_cli).unwrap();
    match outcome {
        CommandOutcome::JobsSuccess(summary) => {
            assert!(summary.contains("Created job"));
            assert!(summary.contains("0 9 * * 1-5"));
        }
        other => panic!("expected JobsSuccess, got {other:?}"),
    }

    let list_cli = CliArgs {
        command: Command::Jobs(JobsArgs::List),
        ..Default::default()
    };
    let outcome = run(&list_cli).unwrap();
    match outcome {
        CommandOutcome::JobsSuccess(list) => {
            assert!(list.contains("STATUS") || list.contains("registered"));
        }
        other => panic!("expected JobsSuccess, got {other:?}"),
    }

    let install_cli = CliArgs {
        command: Command::Jobs(JobsArgs::Install),
        ..Default::default()
    };
    let outcome = run(&install_cli).unwrap();
    match outcome {
        CommandOutcome::JobsSuccess(snippet) => {
            assert!(snippet.contains("cron scheduler setup"));
        }
        other => panic!("expected JobsSuccess, got {other:?}"),
    }
}

#[test]
fn run_verify_detects_lookahead_bias() {
    let tmp_dir = std::env::temp_dir();
    let bad_strategy = tmp_dir.join("test_bad_strategy.py");
    fs::write(
        &bad_strategy,
        "import pandas as pd\n\
         df['target'] = df['close'].shift(-1)\n\
         print('bad lookahead')\n",
    )
    .unwrap();

    let cli = CliArgs {
        command: Command::Verify(VerifyArgs {
            file: Some(bad_strategy.clone()),
            strict: false,
        }),
        ..Default::default()
    };

    let err = run(&cli).unwrap_err();
    match err {
        CliError::VerificationFailed(msg) => {
            assert!(msg.contains("Lookahead bias detected"));
            assert!(msg.contains("Negative shift access"));
        }
        other => panic!("expected VerificationFailed, got {other:?}"),
    }

    let _ = fs::remove_file(bad_strategy);
}

#[test]
fn run_verify_passes_clean_strategy() {
    let tmp_dir = std::env::temp_dir();
    let good_strategy = tmp_dir.join("test_good_strategy.py");
    fs::write(
        &good_strategy,
        "import pandas as pd\n\
         df['sma20'] = df['close'].rolling(20).mean()\n\
         signal = df['close'] > df['sma20']\n",
    )
    .unwrap();

    let cli = CliArgs {
        command: Command::Verify(VerifyArgs {
            file: Some(good_strategy.clone()),
            strict: true,
        }),
        ..Default::default()
    };

    let outcome = run(&cli).unwrap();
    match outcome {
        CommandOutcome::VerifySuccess { passed, message } => {
            assert!(passed);
            assert!(message.contains("0 lookahead errors"));
        }
        other => panic!("expected VerifySuccess, got {other:?}"),
    }

    let _ = fs::remove_file(good_strategy);
}

#[test]
fn run_verify_missing_file_returns_file_error() {
    let cli = CliArgs {
        command: Command::Verify(VerifyArgs {
            file: Some(PathBuf::from("/nonexistent/file/path.py")),
            strict: false,
        }),
        ..Default::default()
    };

    let err = run(&cli).unwrap_err();
    match err {
        CliError::MissingFile(msg) => {
            assert!(msg.contains("does not exist"));
        }
        other => panic!("expected MissingFile, got {other:?}"),
    }
}

#[test]
fn run_memory_and_plugin_commands() {
    let mem_cli = CliArgs {
        command: Command::Memory(MemoryArgs::Stats),
        ..Default::default()
    };
    let outcome = run(&mem_cli).unwrap();
    match outcome {
        CommandOutcome::MemorySuccess(stats) => {
            assert!(stats.contains("Local Semantic Memory"));
        }
        other => panic!("expected MemorySuccess, got {other:?}"),
    }

    let plug_cli = CliArgs {
        command: Command::Plugin(PluginArgs::List),
        ..Default::default()
    };
    let outcome = run(&plug_cli).unwrap();
    match outcome {
        CommandOutcome::PluginSuccess(list) => {
            assert!(list.contains("Installed plugins"));
        }
        other => panic!("expected PluginSuccess, got {other:?}"),
    }
}

#[test]
fn run_ask_empty_prompt_errors() {
    let cli = CliArgs {
        command: Command::Ask(AskArgs::new("  ")),
        ..Default::default()
    };
    assert!(run(&cli).is_err());
}
