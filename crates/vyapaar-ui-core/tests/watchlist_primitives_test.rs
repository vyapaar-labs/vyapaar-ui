//! Unit tests for the `vyapaar-ui-core` watchlist view model.

use rust_decimal::Decimal;
use vyapaar_core::types::{Exchange, InstrumentId};
use vyapaar_ui_core::atoms::Delta;
use vyapaar_ui_core::home::{
    AccountSummary, MarketStatus, MarketStatusKind, WatchlistRow, WatchlistViewModel,
};

fn dec(s: &str) -> Decimal {
    s.parse().unwrap()
}

fn inr(amount: &str) -> vyapaar_core::types::Money {
    vyapaar_core::types::Money::inr(dec(amount))
}

fn inst(symbol: &str) -> InstrumentId {
    InstrumentId::new(Exchange::Nse, symbol)
}

fn row(symbol: &str, price: &str) -> WatchlistRow {
    WatchlistRow::new(
        inst(symbol),
        symbol,
        vyapaar_core::types::Price::new(dec(price)),
        Delta::new(dec("0"), dec("0")),
    )
}

#[test]
fn empty_view_model() {
    let view = WatchlistViewModel::empty();
    assert_eq!(view.row_count(), 0);
    assert!(view.status.is_none());
    assert!(view.account.is_none());
}

#[test]
fn upsert_appends_and_replaces_by_instrument() {
    let mut view = WatchlistViewModel::empty();
    assert!(!view.upsert(row("RELIANCE", "2401.75")));
    assert!(!view.upsert(row("TCS", "3400.25")));
    assert_eq!(view.row_count(), 2);
    assert!(view.upsert(row("RELIANCE", "2405.00")));
    assert_eq!(view.row_count(), 2);
    assert_eq!(view.rows[0].last_price.value(), dec("2405.00"));
}

#[test]
fn remove_only_matching_instrument() {
    let mut view = WatchlistViewModel::empty();
    view.upsert(row("RELIANCE", "2401.75"));
    view.upsert(row("TCS", "3400.25"));
    assert!(view.remove(&inst("TCS")));
    assert_eq!(view.row_count(), 1);
    assert!(!view.remove(&inst("INFY")));
}

#[test]
fn carries_status_and_account() {
    let mut view = WatchlistViewModel::empty();
    view.status = Some(MarketStatus::new(
        MarketStatusKind::Open,
        "09:15 – 15:30 IST",
    ));
    view.account = Some(AccountSummary::new(
        inr("450000"),
        inr("120000"),
        inr("330000"),
        inr("862.50"),
        inr("583862.50"),
    ));
    assert!(view.status.unwrap().is_open());
    assert_eq!(view.account.unwrap().net_worth.amount, dec("583862.50"));
}
