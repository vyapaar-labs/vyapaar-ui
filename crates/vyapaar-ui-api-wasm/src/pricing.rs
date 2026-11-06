//! Simple pricing helpers: notional, P&L, and mark-to-market.

use rust_decimal::Decimal;
use vyapaar_core::types::{InstrumentId, Money, Price, Quantity};

/// Notional value of an order/position in INR.
pub fn notional(price: Price, quantity: Quantity) -> Money {
    Money::notional(price, quantity)
}

/// Realized P&L for a buy-then-sell or sell-then-buy round trip.
///
/// Positive when the exit price is better than the entry price for the given
/// side. For a long (buy) position, P&L = `(exit − entry) × qty`; for a
/// short, P&L = `(entry − exit) × qty`.
pub fn pnl(
    side: vyapaar_core::order::OrderSide,
    entry: Price,
    exit: Price,
    quantity: Quantity,
) -> Money {
    let qty = quantity.value();
    let pnl = match side {
        vyapaar_core::order::OrderSide::Buy => (exit.value() - entry.value()) * qty,
        vyapaar_core::order::OrderSide::Sell => (entry.value() - exit.value()) * qty,
    };
    Money::inr(pnl)
}

/// Mark-to-market P&L of a position given its current price.
pub fn mtm(
    _instrument: &InstrumentId,
    side: vyapaar_core::order::OrderSide,
    entry: Price,
    current: Price,
    quantity: Quantity,
) -> Money {
    pnl(side, entry, current, quantity)
}

/// Break-even price for a position including an entry cost per unit.
pub fn break_even(entry: Price, per_unit_cost: Decimal) -> Price {
    Price::new(entry.value() + per_unit_cost)
}
