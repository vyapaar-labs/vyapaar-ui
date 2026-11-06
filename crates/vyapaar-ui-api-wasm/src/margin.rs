//! Margin estimation for a prospective order.

use rust_decimal::Decimal;
use vyapaar_core::types::{InstrumentId, Money, Price, Quantity};

/// Estimate the margin blocked by an order.
///
/// Margin is `notional / leverage` when a leverage factor is given; for
/// fully-collateralized products (leverage of 1) this equals the notional.
/// Returns [`Money`] in INR.
pub fn margin_required(
    _instrument: &InstrumentId,
    price: Price,
    quantity: Quantity,
    leverage: Decimal,
) -> Money {
    let notional = Money::notional(price, quantity);
    let safe_leverage = if leverage <= Decimal::ZERO {
        Decimal::ONE
    } else {
        leverage
    };
    let amount = notional.amount / safe_leverage;
    Money::inr(amount)
}
