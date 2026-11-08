use clap::Parser;
use rust_decimal::Decimal;
use vyapaar_cli::args::{
    Cli, CliArgs, Command, OutputFormat, StrategyArgs, StrategyRunArgs, StrategySuggestArgs,
    StrategyTuneArgs,
};
use vyapaar_cli::runner::{CommandOutcome, run};
use vyapaar_cli::strategy_bridge::StrategyBridge;

#[test]
fn test_strategy_bridge_discovery() {
    let strategies = StrategyBridge::discover_strategies();
    assert!(strategies.len() >= 4);

    let ids: Vec<&str> = strategies.iter().map(|s| s.id.as_str()).collect();
    assert!(ids.contains(&"turtle_breakout"));
    assert!(ids.contains(&"sma_crossover"));
    assert!(ids.contains(&"rsi_reversal"));
    assert!(ids.contains(&"buy_and_hold"));
}

#[test]
fn test_strategy_bridge_backtest() {
    let report = StrategyBridge::run_backtest(
        "turtle_breakout",
        "NSE:NIFTY50-INDEX",
        None,
        Decimal::new(1000000, 0),
    )
    .expect("backtest should succeed");

    assert_eq!(report.initial_capital, Decimal::new(1000000, 0));
    assert!(report.total_trades > 0);
    assert!(report.max_drawdown_pct >= Decimal::ZERO);
}

#[test]
fn test_strategy_bridge_tuning() {
    let opt = StrategyBridge::tune_strategy(
        "turtle_breakout",
        "NSE:NIFTY50-INDEX",
        Decimal::new(1000000, 0),
        "sharpe",
    )
    .expect("tuning should succeed");

    assert_eq!(opt.strategy_id, "turtle_breakout");
    assert!(opt.total_trials > 1);
    assert!(opt.tuned_params.contains_key("entry_period"));
    assert!(opt.tuned_params.contains_key("exit_period"));
    assert!(opt.deflated_sharpe > Decimal::ZERO);
}

#[test]
fn test_strategy_bridge_suggestion() {
    let sugg = StrategyBridge::generate_trade_suggestion(
        "turtle_breakout",
        "NSE:NIFTY50-INDEX",
        Decimal::new(1000000, 0),
        Decimal::new(10, 1), // 1.0%
    )
    .expect("suggestion should succeed");

    assert_eq!(sugg.strategy.id, "turtle_breakout");
    assert_eq!(sugg.symbol, "NSE:NIFTY50-INDEX");
    assert!(sugg.signal.entry_price > Decimal::ZERO);
    assert!(sugg.signal.stop_loss < sugg.signal.entry_price);
    assert!(sugg.signal.target_1 > sugg.signal.entry_price);
    assert!(sugg.signal.target_2 > sugg.signal.target_1);
    assert_eq!(sugg.sizing.account_capital, Decimal::new(1000000, 0));
    assert_eq!(sugg.sizing.risk_amount, Decimal::new(10000, 0));
    assert!(sugg.sizing.quantity > 0);
    assert!(sugg.statutory_charges.total_charges > Decimal::ZERO);
}

#[test]
fn test_cli_parse_strategy_list() {
    let parsed = Cli::try_parse_from(["vyapaar", "strategy", "list"]).expect("parse list");
    let args: CliArgs = parsed.into();

    assert_eq!(args.command, Command::Strategy(StrategyArgs::List));
}

#[test]
fn test_cli_parse_strategy_run() {
    let parsed = Cli::try_parse_from([
        "vyapaar",
        "strategy",
        "run",
        "sma_crossover",
        "--symbol",
        "NSE:RELIANCE-EQ",
        "--capital",
        "500000",
        "--format",
        "json",
    ])
    .expect("parse run");
    let args: CliArgs = parsed.into();

    match args.command {
        Command::Strategy(StrategyArgs::Run(run_args)) => {
            assert_eq!(run_args.name, "sma_crossover");
            assert_eq!(run_args.symbol, "NSE:RELIANCE-EQ");
            assert_eq!(run_args.capital, Decimal::new(500000, 0));
            assert_eq!(run_args.format, OutputFormat::Json);
        }
        _ => panic!("expected StrategyArgs::Run"),
    }
}

#[test]
fn test_cli_parse_strategy_tune() {
    let parsed = Cli::try_parse_from([
        "vyapaar",
        "strategy",
        "tune",
        "rsi_reversal",
        "--symbol",
        "NSE:HDFCBANK-EQ",
        "--metric",
        "pnl",
    ])
    .expect("parse tune");
    let args: CliArgs = parsed.into();

    match args.command {
        Command::Strategy(StrategyArgs::Tune(tune_args)) => {
            assert_eq!(tune_args.name, "rsi_reversal");
            assert_eq!(tune_args.symbol, "NSE:HDFCBANK-EQ");
            assert_eq!(tune_args.metric, "pnl");
        }
        _ => panic!("expected StrategyArgs::Tune"),
    }
}

#[test]
fn test_cli_parse_strategy_suggest() {
    let parsed = Cli::try_parse_from([
        "vyapaar",
        "strategy",
        "suggest",
        "turtle_breakout",
        "--capital",
        "2000000",
        "--risk-pct",
        "2.0",
    ])
    .expect("parse suggest");
    let args: CliArgs = parsed.into();

    match args.command {
        Command::Strategy(StrategyArgs::Suggest(sugg_args)) => {
            assert_eq!(sugg_args.name, "turtle_breakout");
            assert_eq!(sugg_args.capital, Decimal::new(2000000, 0));
            assert_eq!(sugg_args.risk_pct, Decimal::new(20, 1));
        }
        _ => panic!("expected StrategyArgs::Suggest"),
    }
}

#[test]
fn test_cli_runner_strategy_list() {
    let cli = CliArgs {
        command: Command::Strategy(StrategyArgs::List),
        ..Default::default()
    };
    let outcome = run(&cli).expect("runner should succeed");
    match outcome {
        CommandOutcome::StrategySuccess(msg) => {
            assert!(msg.contains("turtle_breakout"));
            assert!(msg.contains("sma_crossover"));
            assert!(msg.contains("rsi_reversal"));
        }
        _ => panic!("expected StrategySuccess"),
    }
}

#[test]
fn test_cli_runner_strategy_run() {
    let cli = CliArgs {
        command: Command::Strategy(StrategyArgs::Run(StrategyRunArgs {
            name: "turtle_breakout".to_string(),
            symbol: "NSE:NIFTY50-INDEX".to_string(),
            params: None,
            capital: Decimal::new(1000000, 0),
            format: OutputFormat::Text,
        })),
        ..Default::default()
    };
    let outcome = run(&cli).expect("runner should succeed");
    match outcome {
        CommandOutcome::StrategySuccess(msg) => {
            assert!(msg.contains("Strategy Backtest Execution Report: turtle_breakout"));
            assert!(msg.contains("Initial Capital:"));
            assert!(msg.contains("Sharpe Ratio:"));
        }
        _ => panic!("expected StrategySuccess"),
    }
}

#[test]
fn test_cli_runner_strategy_tune() {
    let cli = CliArgs {
        command: Command::Strategy(StrategyArgs::Tune(StrategyTuneArgs {
            name: "turtle_breakout".to_string(),
            symbol: "NSE:NIFTY50-INDEX".to_string(),
            metric: "sharpe".to_string(),
            capital: Decimal::new(1000000, 0),
            format: OutputFormat::Text,
        })),
        ..Default::default()
    };
    let outcome = run(&cli).expect("runner should succeed");
    match outcome {
        CommandOutcome::StrategySuccess(msg) => {
            assert!(msg.contains("Strategy Parameter Tuning & Overfitting Audit: turtle_breakout"));
            assert!(msg.contains("Tuned Value"));
            assert!(msg.contains("Deflated Sharpe:"));
        }
        _ => panic!("expected StrategySuccess"),
    }
}

#[test]
fn test_cli_runner_strategy_suggest_json_and_text() {
    // 1. Text format
    let cli_text = CliArgs {
        command: Command::Strategy(StrategyArgs::Suggest(StrategySuggestArgs {
            name: "turtle_breakout".to_string(),
            symbol: "NSE:NIFTY50-INDEX".to_string(),
            capital: Decimal::new(1000000, 0),
            risk_pct: Decimal::new(10, 1),
            format: OutputFormat::Text,
        })),
        ..Default::default()
    };
    let outcome_text = run(&cli_text).expect("runner should succeed");
    match outcome_text {
        CommandOutcome::StrategySuccess(msg) => {
            assert!(msg.contains("VYAPAAR OPTIMAL TRADE SUGGESTION: TURTLE DONCHIAN BREAKOUT"));
            assert!(msg.contains("PARAMETER TUNING & STATISTICAL AUDIT"));
            assert!(msg.contains("OPTIMAL TRADE EXECUTION CARD"));
            assert!(msg.contains("CAPITAL ALLOCATION & RISK BUDGETING"));
            assert!(msg.contains("Recommended Size"));
            assert!(msg.contains("Net Risk-Reward"));
        }
        _ => panic!("expected StrategySuccess"),
    }

    // 2. JSON format
    let cli_json = CliArgs {
        command: Command::Strategy(StrategyArgs::Suggest(StrategySuggestArgs {
            name: "turtle_breakout".to_string(),
            symbol: "NSE:NIFTY50-INDEX".to_string(),
            capital: Decimal::new(1000000, 0),
            risk_pct: Decimal::new(10, 1),
            format: OutputFormat::Json,
        })),
        ..Default::default()
    };
    let outcome_json = run(&cli_json).expect("runner should succeed");
    match outcome_json {
        CommandOutcome::StrategySuccess(json_str) => {
            let parsed: serde_json::Value =
                serde_json::from_str(&json_str).expect("should be valid JSON");
            assert_eq!(parsed["strategy"]["id"], "turtle_breakout");
            assert_eq!(parsed["symbol"], "NSE:NIFTY50-INDEX");
            assert!(parsed["signal"]["entry_price"].is_string());
            assert!(parsed["sizing"]["quantity"].is_number());
            assert!(parsed["statutory_charges"]["total_charges"].is_string());
        }
        _ => panic!("expected StrategySuccess"),
    }
}
