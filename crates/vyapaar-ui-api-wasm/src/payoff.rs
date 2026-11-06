//! Option payoff at expiry (exact Decimal).

use rust_decimal::Decimal;

/// Option direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OptionType {
    /// Call option.
    Call,
    /// Put option.
    Put,
}

/// Intrinsic payoff of a single option at expiry, ignoring premium.
///
/// - Call: `max(spot − strike, 0)`
/// - Put:  `max(strike − spot, 0)`
pub fn payoff(option: OptionType, spot: Decimal, strike: Decimal) -> Decimal {
    match option {
        OptionType::Call => (spot - strike).max(Decimal::ZERO),
        OptionType::Put => (strike - spot).max(Decimal::ZERO),
    }
}

/// Net payoff including the premium paid (negative premium = cost).
pub fn payoff_net(option: OptionType, spot: Decimal, strike: Decimal, premium: Decimal) -> Decimal {
    payoff(option, spot, strike) - premium
}

/// Payoff curve across a range of spot prices (exclusive of premium).
pub fn payoff_curve(
    option: OptionType,
    strike: Decimal,
    spots: &[Decimal],
) -> Vec<(Decimal, Decimal)> {
    spots
        .iter()
        .map(|&spot| (spot, payoff(option, spot, strike)))
        .collect()
}
