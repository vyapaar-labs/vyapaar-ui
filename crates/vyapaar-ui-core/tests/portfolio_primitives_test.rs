//! Unit tests for `vyapaar-ui-core` portfolio view primitives.

use rust_decimal::Decimal;
use vyapaar_core::instrument::PositionSide;
use vyapaar_core::portfolio::{
    PortfolioAccount, PortfolioMode, PortfolioPosition, PortfolioSnapshot, PortfolioSource,
    PortfolioTotals,
};
use vyapaar_core::types::{Currency, Exchange, InstrumentId, Money, Price, Quantity};
use vyapaar_ui_core::portfolio::{PortfolioHoldingRow, PortfolioViewModel};

fn dec(s: &str) -> Decimal {
    s.parse().unwrap()
}

fn inr(amount: &str) -> Money {
    Money::new(dec(amount), Currency::Inr)
}

fn inst(symbol: &str) -> InstrumentId {
    InstrumentId::new(Exchange::Nse, symbol)
}

fn snapshot_with_positions() -> PortfolioSnapshot {
    PortfolioSnapshot {
        account: PortfolioAccount {
            account_id: "a1".to_string(),
            mode: PortfolioMode::Paper,
            source: PortfolioSource::Simulated,
        },
        positions: vec![
            PortfolioPosition {
                instrument: inst("RELIANCE"),
                side: PositionSide::Long,
                quantity: Quantity::from_u64(50),
                avg_price: Price::new(dec("2390.00")),
                unrealized_pnl: inr("587.50"),
                realized_pnl: inr("120.00"),
            },
            PortfolioPosition {
                instrument: inst("TCS"),
                side: PositionSide::Short,
                quantity: Quantity::from_u64(20),
                avg_price: Price::new(dec("3410.00")),
                unrealized_pnl: inr("195.00"),
                realized_pnl: inr("-40.00"),
            },
        ],
        holdings: Vec::new(),
        totals: PortfolioTotals {
            cash: inr("450000.00"),
            margin_used: inr("120000.00"),
            margin_available: inr("330000.00"),
            total_pnl: inr("862.50"),
            net_worth: inr("583862.50"),
        },
        greeks: None,
        timestamp: chrono::Utc::now(),
    }
}

#[test]
fn empty_view_is_empty() {
    let view = PortfolioViewModel::empty();
    assert!(view.is_empty());
    assert_eq!(view.unrealized_pnl_total(), Decimal::ZERO);
    assert_eq!(view.realized_pnl_total(), Decimal::ZERO);
}

#[test]
fn from_snapshot_projects_rows_and_totals() {
    let view = PortfolioViewModel::from_snapshot(&snapshot_with_positions());
    assert!(!view.is_empty());
    assert_eq!(view.positions.len(), 2);
    assert_eq!(view.positions[0].instrument, inst("RELIANCE"));
    assert_eq!(view.positions[0].side, PositionSide::Long);
    assert_eq!(view.positions[1].side, PositionSide::Short);
    assert_eq!(view.totals.cash, inr("450000.00"));
    assert_eq!(view.totals.net_worth, inr("583862.50"));
}

#[test]
fn pnl_totals_sum_position_amounts() {
    let view = PortfolioViewModel::from_snapshot(&snapshot_with_positions());
    assert_eq!(view.unrealized_pnl_total(), dec("782.50"));
    assert_eq!(view.realized_pnl_total(), dec("80.00"));
}

#[test]
fn holding_row_constructor() {
    let row = PortfolioHoldingRow::new(
        inst("TCS"),
        Quantity::from_u64(10),
        Price::new(dec("3400.00")),
        inr("34000.00"),
    );
    assert_eq!(row.current_value.amount, dec("34000.00"));
}
