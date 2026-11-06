//! Tests for margin bindings.

use rust_decimal::Decimal;
use vyapaar_core::types::{Exchange, InstrumentId, Price, Quantity};
use vyapaar_ui_api_wasm::margin::margin_required;

fn dec(s: &str) -> Decimal {
    s.parse().unwrap()
}

#[test]
fn leverage_two_halves_margin() {
    let inst = InstrumentId::new(Exchange::Nfo, "NIFTY");
    let m = margin_required(&inst, Price::new(dec("20000")), Quantity::from_u64(50), dec("2"));
    // notional = 1,000,000 / 2 = 500,000
    assert_eq!(m.amount, dec("500000"));
}

#[test]
fn leverage_one_equals_notional() {
    let inst = InstrumentId::new(Exchange::Nse, "RELIANCE");
    let m = margin_required(&inst, Price::new(dec("2500")), Quantity::from_u64(10), Decimal::ONE);
    assert_eq!(m.amount, dec("25000"));
}

#[test]
fn zero_or_negative_leverage_defaults_to_one() {
    let inst = InstrumentId::new(Exchange::Nse, "TCS");
    let m = margin_required(&inst, Price::new(dec("1000")), Quantity::from_u64(5), Decimal::ZERO);
    assert_eq!(m.amount, dec("5000"));
}
