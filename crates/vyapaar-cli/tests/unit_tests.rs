//! Unit tests for the Vyapaar CLI implementing the Quantcept command hierarchy.

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use vyapaar_agent_llm::ScriptedFakeLlm;
use vyapaar_cli::args::{
    AskArgs, ChatArgs, Cli, CliArgs, CliCommand, Command, DoctorArgs, JobsArgs, LogLevel,
    MarketplaceArgs, McpArgs, MemoryArgs, OutputFormat, PluginArgs, PurgeArgs, ScanArgs,
    SkillsArgs, VerifyArgs, resolve_log_level,
};
use vyapaar_cli::runner::{CliError, CommandOutcome, run, run_jobs_with_home, run_with_provider};

#[test]
fn parse_cli_roundtrip_tui() {
    let cli = Cli {
        command: Some(CliCommand::Tui),
        ..Default::default()
    };
    let args: CliArgs = cli.into();
    assert_eq!(args.command, Command::Tui { message: None });
    assert_eq!(args.model_path, None);
}

#[test]
fn parse_cli_with_initial_message() {
    let cli = Cli {
        message: Some("What is NIFTY?".into()),
        ..Default::default()
    };
    let args: CliArgs = cli.into();
    assert_eq!(
        args.command,
        Command::Tui {
            message: Some("What is NIFTY?".into())
        }
    );
}

#[test]
fn parse_cli_roundtrip_ask() {
    let cli = Cli {
        command: Some(CliCommand::Ask {
            prompt: "what is NIFTY?".into(),
            timeout_min: 20,
            persona: Some("quant".into()),
            mode: "paper".into(),
            profile: "default".into(),
            format: OutputFormat::Json,
        }),
        ..Default::default()
    };
    let args: CliArgs = cli.into();
    assert_eq!(
        args.command,
        Command::Ask(AskArgs {
            prompt: "what is NIFTY?".into(),
            timeout_min: 20,
            persona: Some("quant".into()),
            mode: "paper".into(),
            profile: "default".into(),
            format: OutputFormat::Json,
        })
    );
}

#[test]
fn test_log_level_precedence() {
    // 1. Quiet flag sets Silent
    assert_eq!(
        resolve_log_level(Some(LogLevel::Debug), false, true),
        LogLevel::Silent
    );
    // 2. Verbose flag sets Debug
    assert_eq!(
        resolve_log_level(Some(LogLevel::Info), true, false),
        LogLevel::Debug
    );
    // 3. Explicit flag
    assert_eq!(
        resolve_log_level(Some(LogLevel::Warn), false, false),
        LogLevel::Warn
    );
    // 4. Default is Info
    assert_eq!(resolve_log_level(None, false, false), LogLevel::Info);
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
        command: Command::Tui { message: None },
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
    assert_eq!(
        run_with_provider(&cli, None),
        Err(CliError::NotWired("vyapaar ask".into()))
    );
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
fn run_ask_json_format() {
    let fake = ScriptedFakeLlm::new("test-fake-llm");
    fake.push_text_response("Target return achieved.");

    let mut ask_args = AskArgs::new("calculate Sharpe");
    ask_args.format = OutputFormat::Json;
    ask_args.persona = Some("quant".into());

    let cli = CliArgs {
        command: Command::Ask(ask_args),
        ..Default::default()
    };

    let outcome = run_with_provider(&cli, Some(Arc::new(fake))).unwrap();
    match outcome {
        CommandOutcome::Answer(ans) => {
            assert!(ans.contains("Target return achieved."));
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
    let temp = tempfile::tempdir().unwrap();
    let add_args = JobsArgs::Add {
        goal: "Scan momentum break in Nifty midcaps".into(),
        schedule: Some("0 9 * * 1-5".into()),
        max_turns: 20,
        max_inr: None,
        max_usd: None,
        max_tokens: Some(4000),
        max_tool_calls: Some(10),
        max_data_calls: Some(5),
        missed_policy: vyapaar_cli::args::MissedPolicy::Skip,
        approval: vyapaar_cli::args::ApprovalPolicyArg::Auto,
        strategy: Some(String::from("sma_crossover")),
        symbol: Some(String::from("NSE:RELIANCE-EQ")),
        capital: Some(rust_decimal::Decimal::new(1_000_000, 0)),
        params: Some(String::from("{\"fast\":10}")),
        success: None,
        read_only: true,
        once: false,
    };
    let outcome = run_jobs_with_home(&add_args, temp.path()).unwrap();
    match outcome {
        CommandOutcome::JobsSuccess(summary) => {
            assert!(summary.contains("Created job"));
            assert!(summary.contains("0 9 * * 1-5"));
            assert!(summary.contains("sma_crossover"));
            assert!(summary.contains("NSE:RELIANCE-EQ"));
        }
        other => panic!("expected JobsSuccess, got {other:?}"),
    }

    let outcome = run_jobs_with_home(&JobsArgs::List, temp.path()).unwrap();
    match outcome {
        CommandOutcome::JobsSuccess(list) => {
            assert!(list.contains("STATUS") || list.contains("registered"));
            assert!(
                list.contains("Scan momentum break in Nifty midcaps"),
                "job list must show the goal: {list}"
            );
        }
        other => panic!("expected JobsSuccess, got {other:?}"),
    }

    let outcome = run_jobs_with_home(&JobsArgs::Install, temp.path()).unwrap();
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

    let market_cli = CliArgs {
        command: Command::Plugin(PluginArgs::Marketplace(MarketplaceArgs::List)),
        ..Default::default()
    };
    let outcome = run(&market_cli).unwrap();
    match outcome {
        CommandOutcome::PluginSuccess(list) => {
            assert!(list.contains("Marketplace"));
        }
        other => panic!("expected Marketplace, got {other:?}"),
    }
}

#[test]
fn run_skills_and_mcp_commands() {
    let skills_cli = CliArgs {
        command: Command::Skills(SkillsArgs::List),
        ..Default::default()
    };
    let outcome = run(&skills_cli).unwrap();
    match outcome {
        CommandOutcome::SkillsSuccess(skills) => {
            assert!(skills.contains("The 8 Quant Desks"));
            assert!(skills.contains("quant"));
            assert!(skills.contains("analyst"));
            assert!(skills.contains("trader"));
        }
        other => panic!("expected SkillsSuccess, got {other:?}"),
    }

    let mcp_cli = CliArgs {
        command: Command::Mcp(McpArgs::List),
        ..Default::default()
    };
    let outcome = run(&mcp_cli).unwrap();
    match outcome {
        CommandOutcome::McpSuccess(mcp) => {
            assert!(mcp.contains("MCP"));
            assert!(mcp.contains("context7"));
            assert!(mcp.contains("github"));
        }
        other => panic!("expected McpSuccess, got {other:?}"),
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

#[test]
fn test_parse_backtest_query() {
    use vyapaar_cli::backtest_gen::parse_backtest_query;
    let spec =
        parse_backtest_query("backtest 10L on Alpha30 stocks ; with 15 day rebalancing").unwrap();
    assert_eq!(spec.capital, rust_decimal::Decimal::from(1_000_000));
    assert_eq!(spec.universe, "Alpha30");
    assert_eq!(spec.rebalance_days, 15);
}

#[test]
fn test_run_ask_backtest_synthesis() {
    let cli = CliArgs {
        command: Command::Ask(AskArgs::new(
            "backtest 10L on Alpha30 stocks ; with 15 day rebalancing",
        )),
        ..Default::default()
    };
    let outcome = run(&cli).unwrap();
    match outcome {
        CommandOutcome::Answer(ans) => {
            assert!(ans.contains("Backtest Execution Report"));
            assert!(ans.contains("Alpha30"));
            assert!(ans.contains("15-Day Rebalance"));
            assert!(std::path::Path::new("backtest_alpha30.py").exists());
        }
        other => panic!("expected Answer with backtest report, got {other:?}"),
    }
}
