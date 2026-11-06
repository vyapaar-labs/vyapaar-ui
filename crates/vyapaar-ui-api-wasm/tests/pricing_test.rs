//! Tests for pricing bindings.

use rust_decimal::Decimal;
use vyapaar_core::order::OrderSide;
use vyapaar_core::types::{Exchange, InstrumentId, Price, Quantity};
use vyapaar_ui_api_wasm::pricing::{break_even, mtm, notional, pnl};

fn dec(s: &str) -> Decimal {
    s.parse().unwrap()
}

#[test]
fn notional_is_price_times_quantity() {
    let n = notional(Price::new(dec("2500")), Quantity::from_u64(10));
    assert_eq!(n.amount, dec("25000"));
}

#[test]
fn long_pnl_profit_and_loss() {
    let profit = pnl(OrderSide::Buy, Price::new(dec("100")), Price::new(dec("110")), Quantity::from_u64(5));
    assert_eq!(profit.amount, dec("50"));

    let loss = pnl(OrderSide::Buy, Price::new(dec("100")), Price::new(dec("90")), Quantity::from_u64(5));
    assert_eq!(loss.amount, dec("-50"));
}

#[test]
fn short_pnl_inverts() {
    let profit = pnl(OrderSide::Sell, Price::new(dec("100")), Price::new(dec("90")), Quantity::from_u64(5));
    assert_eq!(profit.amount, dec("50"));
}

#[test]
fn mtm_uses_current_price() {
    let inst = InstrumentId::new(Exchange::Nse, "TCS");
    let m = mtm(&inst, OrderSide::Buy, Price::new(dec("100")), Price::new(dec("105")), Quantity::from_u64(2));
    assert_eq!(m.amount, dec("10"));
}

#[test]
fn break_even_adds_per_unit_cost() {
    let be = break_even(Price::new(dec("100")), dec("0.50"));
    assert_eq!(be.value(), dec("100.5"));
}
