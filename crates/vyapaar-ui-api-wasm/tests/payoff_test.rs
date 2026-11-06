//! Tests for option payoff bindings.

use rust_decimal::Decimal;
use vyapaar_ui_api_wasm::payoff::{OptionType, payoff, payoff_curve, payoff_net};

fn dec(s: &str) -> Decimal {
    s.parse().unwrap()
}

#[test]
fn call_intrinsic_at_expiry() {
    assert_eq!(payoff(OptionType::Call, dec("105"), dec("100")), dec("5"));
    // At or below strike, call payoff is zero.
    assert_eq!(payoff(OptionType::Call, dec("100"), dec("100")), Decimal::ZERO);
    assert_eq!(payoff(OptionType::Call, dec("95"), dec("100")), Decimal::ZERO);
}

#[test]
fn put_intrinsic_at_expiry() {
    assert_eq!(payoff(OptionType::Put, dec("95"), dec("100")), dec("5"));
    assert_eq!(payoff(OptionType::Put, dec("105"), dec("100")), Decimal::ZERO);
}

#[test]
fn net_payoff_subtracts_premium() {
    let net = payoff_net(OptionType::Call, dec("105"), dec("100"), dec("2"));
    assert_eq!(net, dec("3"));
    // Below strike: only the premium is lost.
    let net = payoff_net(OptionType::Call, dec("90"), dec("100"), dec("2"));
    assert_eq!(net, dec("-2"));
}

#[test]
fn payoff_curve_spans_spot_range() {
    let spots = vec![dec("90"), dec("100"), dec("110")];
    let curve = payoff_curve(OptionType::Call, dec("100"), &spots);
    assert_eq!(curve, vec![(dec("90"), Decimal::ZERO), (dec("100"), Decimal::ZERO), (dec("110"), dec("10"))]);
}
