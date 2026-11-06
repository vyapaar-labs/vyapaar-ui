use vyapaar_cli::{CliArgs, Command, run_command};

#[tokio::test]
async fn test_cli_tui_command() {
    let args = CliArgs::new(Command::Tui);
    let out = run_command(args).await.expect("tui run");
    assert!(out.contains("TUI initialized"));
}

#[tokio::test]
async fn test_cli_ask_command() {
    let args = CliArgs::new(Command::Ask {
        query: "What is my current margin?".to_string(),
        profile: Some("algo_client".to_string()),
        mode: Some("paper".to_string()),
        persona: None,
    });
    let out = run_command(args).await.expect("ask run");
    assert!(out.contains("Answering query 'What is my current margin?'"));
    assert!(out.contains("algo_client"));
}

#[tokio::test]
async fn test_cli_ask_with_persona_command() {
    let args = CliArgs::new(Command::Ask {
        query: "Compute walk-forward Sharpe".to_string(),
        profile: Some("quant_desk".to_string()),
        mode: Some("backtest".to_string()),
        persona: Some("quant".to_string()),
    });
    let out = run_command(args).await.expect("ask with persona run");
    assert!(out.contains("[Quant]"));
    assert!(out.contains("Compute walk-forward Sharpe"));
}

#[tokio::test]
async fn test_cli_skills_list_command() {
    let args = CliArgs::new(Command::Skills { action: None });
    let out = run_command(args).await.expect("skills list");
    assert!(out.contains("Installed Default Vyapaar Agent Skills"));
    assert!(out.contains("quant"));
    assert!(out.contains("analyst"));
    assert!(out.contains("risk-manager"));
    assert!(out.contains("trader"));
}

#[tokio::test]
async fn test_cli_skills_info_command() {
    let args = CliArgs::new(Command::Skills {
        action: Some("info trader".to_string()),
    });
    let out = run_command(args).await.expect("skills info");
    assert!(out.contains("Skill: trader"));
    assert!(out.contains("Title: Trader"));
    assert!(out.contains("vyapaar_ticker_price"));
}

#[tokio::test]
async fn test_cli_scan_command() {
    let args = CliArgs::new(Command::Scan {
        universe: Some("nifty_bank".to_string()),
        metric: Some("bollinger_pct_b".to_string()),
    });
    let out = run_command(args).await.expect("scan run");
    assert!(out.contains("nifty_bank"));
}

#[tokio::test]
async fn test_cli_health_command() {
    let args = CliArgs::new(Command::Health);
    let out = run_command(args).await.expect("health run");
    assert!(out.contains("System health: OK"));
}
