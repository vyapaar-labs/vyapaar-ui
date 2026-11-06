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
    });
    let out = run_command(args).await.expect("ask run");
    assert!(out.contains("Answering query 'What is my current margin?'"));
    assert!(out.contains("algo_client"));
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
