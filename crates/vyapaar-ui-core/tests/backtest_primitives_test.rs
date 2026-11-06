//! Unit tests for `vyapaar-ui-core` backtest results primitives.

use rust_decimal::Decimal;
use vyapaar_ui_core::atoms::EquityPoint;
use vyapaar_ui_core::backtest::{BacktestResultView, BacktestStatus, BacktestSummary};

fn dec(s: &str) -> Decimal {
    s.parse().unwrap()
}

#[test]
fn summary_zero_is_all_zeroes() {
    let s = BacktestSummary::zero();
    assert_eq!(s.initial_cash, Decimal::ZERO);
    assert_eq!(s.final_equity, Decimal::ZERO);
    assert_eq!(s.net_pnl, Decimal::ZERO);
    assert_eq!(s.total_trades, 0);
    assert_eq!(s.refused_orders, 0);
}

#[test]
fn total_return_pct_from_equity() {
    let s = BacktestSummary {
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
    };
    assert_eq!(s.total_return_pct(), dec("20"));
}

#[test]
fn total_return_pct_zero_initial_is_zero() {
    let s = BacktestSummary::zero();
    assert_eq!(s.total_return_pct(), Decimal::ZERO);
}

#[test]
fn result_view_empty_is_idle_without_results() {
    let view = BacktestResultView::empty();
    assert_eq!(view.status, BacktestStatus::Idle);
    assert!(!view.has_results());
    assert!(view.equity_curve.is_empty());
    assert_eq!(view.last_equity(), None);
    assert_eq!(view.peak_equity(), None);
    assert_eq!(view.trough_equity(), None);
}

#[test]
fn result_view_with_complete_summary_has_results() {
    let mut view = BacktestResultView::empty();
    view.status = BacktestStatus::Complete;
    view.summary = Some(BacktestSummary {
        initial_cash: dec("100000"),
        final_equity: dec("110000"),
        net_pnl: dec("10000"),
        gross_pnl: dec("10000"),
        total_fees: dec("0"),
        total_trades: 5,
        total_signals: 8,
        refused_orders: 0,
        max_drawdown_pct: dec("3"),
        price_impact: dec("0"),
    });
    view.equity_curve = vec![
        EquityPoint::new(0, dec("100000")),
        EquityPoint::new(1, dec("105000")),
        EquityPoint::new(2, dec("110000")),
    ];
    assert!(view.has_results());
    assert_eq!(view.last_equity(), Some(EquityPoint::new(2, dec("110000"))));
    assert_eq!(view.peak_equity(), Some(dec("110000")));
    assert_eq!(view.trough_equity(), Some(dec("100000")));
}

#[test]
fn running_status_is_not_has_results() {
    let mut view = BacktestResultView::empty();
    view.status = BacktestStatus::Running;
    assert!(!view.has_results());
}
