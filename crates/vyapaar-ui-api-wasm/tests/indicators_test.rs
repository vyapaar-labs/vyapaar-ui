//! Tests for indicator bindings.

use rust_decimal::Decimal;
use vyapaar_ui_api_wasm::ApiError;
use vyapaar_ui_api_wasm::indicators::{ema, rsi, sma};

fn dec(s: &str) -> Decimal {
    s.parse().unwrap()
}

#[test]
fn sma_window_averages() {
    let values = vec![dec("1"), dec("2"), dec("3"), dec("4"), dec("5")];
    let out = sma(&values, 3).unwrap();
    assert_eq!(
        out,
        vec![None, None, Some(dec("2")), Some(dec("3")), Some(dec("4"))]
    );
}

#[test]
fn sma_zero_period_errors() {
    assert_eq!(
        sma(&[dec("1")], 0),
        Err(ApiError::InvalidArgument("period must be > 0".to_string()))
    );
}

#[test]
fn ema_starts_at_first_value_and_tracks_series() {
    let values = vec![dec("10"), dec("11"), dec("12"), dec("13"), dec("14")];
    let out = ema(&values, 3).unwrap();
    assert_eq!(out.len(), values.len());
    // Seeded at the first value.
    assert_eq!(out[0], dec("10"));
    // Series rises, so EMA keeps rising.
    assert!(out[4] > out[0]);
}

#[test]
fn ema_empty_is_empty() {
    assert!(ema(&[], 5).unwrap().is_empty());
}

#[test]
fn rsi_all_gains_is_100() {
    let values = (1..=10).map(Decimal::from).collect::<Vec<_>>();
    let out = rsi(&values, 4).unwrap();
    assert_eq!(out[4], Some(dec("100")));
}

#[test]
fn rsi_short_series_is_all_none() {
    let values = vec![dec("1"), dec("2")];
    let out = rsi(&values, 5).unwrap();
    assert!(out.iter().all(Option::is_none));
}

#[test]
fn rsi_known_value() {
    // A gentle mixed series: RSI(5) should be strictly between 0 and 100.
    let values = vec![
        dec("10"), dec("11"), dec("10.5"), dec("12"), dec("11"), dec("12.5"), dec("13"),
    ];
    let out = rsi(&values, 5).unwrap();
    let r = out[5].expect("rsi at index 5");
    assert!(r > Decimal::ZERO && r < dec("100"));
}
