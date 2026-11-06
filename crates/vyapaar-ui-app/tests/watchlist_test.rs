//! Tests for the watchlist feature.

use rust_decimal::Decimal;
use vyapaar_core::market::{Bar, Tick};
use vyapaar_core::types::{Exchange, InstrumentId, Price, Quantity};
use vyapaar_ui_app::watchlist::WatchlistFeature;
use vyapaar_ui_core::atoms::Delta;
use vyapaar_ui_core::home::{MarketStatus, MarketStatusKind, WatchlistRow};

fn dec(s: &str) -> Decimal {
    s.parse().unwrap()
}

fn inst(symbol: &str) -> InstrumentId {
    InstrumentId::new(Exchange::Nse, symbol)
}

fn reliance_row() -> WatchlistRow {
    WatchlistRow::new(
        inst("RELIANCE"),
        "RELIANCE",
        Price::new(dec("2400.00")),
        Delta::new(dec("0"), dec("0")),
    )
}

#[test]
fn new_feature_starts_empty() {
    let f = WatchlistFeature::new();
    assert_eq!(f.row_count(), 0);
}

#[test]
fn upsert_row_replaces_and_keeps_count() {
    let mut f = WatchlistFeature::new();
    assert!(!f.upsert_row(reliance_row()));
    // Re-upserting the same instrument replaces, returning true.
    assert!(f.upsert_row(reliance_row().with_spark(vec![dec("2400")])));
    // A different instrument appends.
    let tcs = WatchlistRow::new(
        inst("TCS"),
        "TCS",
        Price::new(dec("3400.00")),
        Delta::new(dec("0"), dec("0")),
    );
    assert!(!f.upsert_row(tcs));
    assert_eq!(f.row_count(), 2);
}

#[test]
fn remove_instrument() {
    let mut f = WatchlistFeature::new();
    f.upsert_row(reliance_row());
    assert!(f.remove(&inst("RELIANCE")));
    assert_eq!(f.row_count(), 0);
    assert!(!f.remove(&inst("TCS")));
}

#[test]
fn ingest_tick_updates_price_and_delta() {
    let mut f = WatchlistFeature::new();
    f.upsert_row(reliance_row());
    let tick = Tick::Ltp {
        instrument: inst("RELIANCE"),
        price: Price::new(dec("2404.80")),
        volume: Quantity::from_u64(500),
        exchange_ts: vyapaar_core::chrono::Utc::now(),
        receive_ts: vyapaar_core::chrono::Utc::now(),
    };
    f.ingest_tick(&tick);
    assert_eq!(f.row_count(), 1);
    let row = &f.view.rows[0];
    assert_eq!(row.last_price.value(), dec("2404.80"));
    // Delta against the seed price 2400.00 -> +4.80 / +0.2%.
    assert_eq!(row.delta.absolute, dec("4.80"));
}

#[test]
fn ingest_bar_appends_spark_and_updates_price() {
    let mut f = WatchlistFeature::new();
    f.upsert_row(reliance_row());
    let bar = Bar {
        instrument: inst("RELIANCE"),
        open: Price::new(dec("2400.00")),
        high: Price::new(dec("2406.00")),
        low: Price::new(dec("2398.00")),
        close: Price::new(dec("2405.00")),
        volume: Quantity::from_u64(1000),
        timestamp: vyapaar_core::chrono::Utc::now(),
    };
    f.ingest_bar(&bar);
    assert_eq!(f.view.rows[0].last_price.value(), dec("2405.00"));
    assert_eq!(f.view.rows[0].spark, vec![dec("2405.00")]);
    assert_eq!(f.view.rows[0].delta.absolute, dec("5.00"));
}

#[test]
fn tick_for_unsubscribed_instrument_is_ignored() {
    let mut f = WatchlistFeature::new();
    f.upsert_row(reliance_row());
    let tick = Tick::Ltp {
        instrument: inst("TCS"),
        price: Price::new(dec("3400.25")),
        volume: Quantity::from_u64(100),
        exchange_ts: vyapaar_core::chrono::Utc::now(),
        receive_ts: vyapaar_core::chrono::Utc::now(),
    };
    f.ingest_tick(&tick);
    assert_eq!(f.row_count(), 1);
    assert_eq!(f.view.rows[0].instrument, inst("RELIANCE"));
}

#[test]
fn set_market_status_sticks() {
    let mut f = WatchlistFeature::new();
    f.set_market_status(MarketStatus::new(
        MarketStatusKind::Open,
        "09:15 – 15:30 IST",
    ));
    assert!(f.view.status.unwrap().is_open());
}
