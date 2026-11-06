//! Tests for the backtest results feature.

use rust_decimal::Decimal;
use vyapaar_ui_app::backtest::BacktestFeature;
use vyapaar_ui_core::atoms::EquityPoint;
use vyapaar_ui_core::backtest::{BacktestStatus, BacktestSummary};

fn dec(s: &str) -> Decimal {
    s.parse().unwrap()
}

fn summary() -> BacktestSummary {
    BacktestSummary {
        initial_cash: dec("100000"),
        final_equity: dec("120000"),
        net_pnl: dec("20000"),
        gross_pnl: dec("22000"),
        total_fees: dec("2000"),
        total_trades: 12,
        total_signals: 20,
        refused_orders: 2,
        max_drawdown_pct: dec("8.5"),
        price_impact: dec("150"),
    }
}

#[test]
fn new_feature_is_idle() {
    let f = BacktestFeature::new();
    assert_eq!(f.view.status, BacktestStatus::Idle);
    assert!(!f.has_results());
}

#[test]
fn run_lifecycle_start_push_complete() {
    let mut f = BacktestFeature::new();
    f.start_run();
    assert_eq!(f.view.status, BacktestStatus::Running);
    f.push_equity(EquityPoint::new(0, dec("100000")));
    f.push_equity(EquityPoint::new(1, dec("120000")));
    assert_eq!(f.view.equity_curve.len(), 2);
    f.complete_run(summary());
    assert_eq!(f.view.status, BacktestStatus::Complete);
    assert!(f.has_results());
    assert_eq!(f.total_return_pct(), dec("20"));
    assert_eq!(f.view.peak_equity(), Some(dec("120000")));
}

#[test]
fn push_equity_ignored_when_not_running() {
    let mut f = BacktestFeature::new();
    f.push_equity(EquityPoint::new(0, dec("100000")));
    assert!(f.view.equity_curve.is_empty());
    f.fail_run();
    assert_eq!(f.view.status, BacktestStatus::Failed);
    assert!(!f.has_results());
}

#[test]
fn start_run_clears_previous_results() {
    let mut f = BacktestFeature::new();
    f.start_run();
    f.complete_run(summary());
    assert!(f.has_results());
    f.start_run();
    assert!(f.view.summary.is_none());
    assert!(f.view.equity_curve.is_empty());
}
