//! Tests for analytics bindings.

use rust_decimal::Decimal;
use vyapaar_ui_api_wasm::ApiError;
use vyapaar_ui_api_wasm::analytics::{drawdown, sharpe_ratio, sortino_ratio};

fn dec(s: &str) -> Decimal {
    s.parse().unwrap()
}

#[test]
fn sharpe_of_constant_returns_errors() {
    let returns = vec![dec("0.01"), dec("0.01")];
    assert_eq!(sharpe_ratio(&returns, 252), Err(ApiError::InsufficientData));
}

#[test]
fn sharpe_positive_for_uptrending_series() {
    let returns = vec![
        dec("0.01"),
        dec("0.02"),
        dec("0.015"),
        dec("0.03"),
        dec("0.02"),
    ];
    let s = sharpe_ratio(&returns, 252).unwrap();
    assert!(s > Decimal::ZERO);
}

#[test]
fn sharpe_insufficient_data() {
    assert_eq!(
        sharpe_ratio(&[dec("0.01")], 252),
        Err(ApiError::InsufficientData)
    );
    assert_eq!(
        sharpe_ratio(&[dec("0.01"), dec("0.02")], 0),
        Err(ApiError::InvalidArgument(
            "periods_per_year must be > 0".to_string()
        ))
    );
}

#[test]
fn sortino_positive_for_uptrending_series() {
    let returns = vec![
        dec("0.01"),
        dec("0.02"),
        dec("0.015"),
        dec("0.03"),
        dec("0.02"),
    ];
    let s = sortino_ratio(&returns, 252).unwrap();
    assert!(s > Decimal::ZERO);
}

#[test]
fn drawdown_of_peak_then_trough() {
    // equity 100 -> 120 -> 90: peak 120, trough 90 => 25% drawdown.
    let equity = vec![dec("100"), dec("120"), dec("90")];
    assert_eq!(drawdown(&equity).unwrap(), dec("0.25"));
}

#[test]
fn drawdown_monotonic_is_zero() {
    let equity = vec![dec("100"), dec("110"), dec("120")];
    assert_eq!(drawdown(&equity).unwrap(), Decimal::ZERO);
}

#[test]
fn drawdown_empty_errors() {
    assert_eq!(drawdown(&[]), Err(ApiError::InsufficientData));
}
