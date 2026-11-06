//! Unit tests for `vyapaar-ui-core` home primitives.

use rust_decimal::Decimal;
use vyapaar_core::types::{Currency, Exchange, InstrumentId, Money, Price};
use vyapaar_ui_core::atoms::Delta;
use vyapaar_ui_core::home::{AccountSummary, MarketStatus, MarketStatusKind, WatchlistRow};

fn dec(s: &str) -> Decimal {
    s.parse().unwrap()
}

fn inr(amount: &str) -> Money {
    Money::new(dec(amount), Currency::Inr)
}

#[test]
fn market_status_open() {
    let status = MarketStatus::new(MarketStatusKind::Open, "09:15 – 15:30 IST");
    assert!(status.is_open());
    assert!(status.next_session.is_none());
}

#[test]
fn market_status_closed_with_next_session() {
    let status = MarketStatus::new(MarketStatusKind::Closed, "09:15 – 15:30 IST")
        .with_next_session("Reopens 09:15 IST");
    assert!(!status.is_open());
    assert_eq!(status.next_session.as_deref(), Some("Reopens 09:15 IST"));
}

#[test]
fn watchlist_row_with_spark() {
    let instrument = InstrumentId::new(Exchange::Nse, "TCS");
    let row = WatchlistRow::new(
        instrument.clone(),
        "TCS",
        Price::new(dec("3400.50")),
        Delta::new(dec("12.50"), dec("0.37")),
    )
    .with_spark(vec![dec("3380"), dec("3390"), dec("3400.50")]);
    assert_eq!(row.instrument, instrument);
    assert_eq!(row.name, "TCS");
    assert_eq!(row.spark.len(), 3);
}

#[test]
fn account_summary_holds_totals() {
    let summary = AccountSummary::new(
        inr("50000"),
        inr("20000"),
        inr("30000"),
        inr("1250.75"),
        inr("70000"),
    );
    assert_eq!(summary.cash.amount, dec("50000"));
    assert_eq!(summary.margin_available.amount, dec("30000"));
    assert_eq!(summary.total_pnl.amount, dec("1250.75"));
    assert_eq!(summary.net_worth.amount, dec("70000"));
}
