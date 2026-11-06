//! Pairing of two aligned series (e.g. price vs. indicator, two symbols).

use rust_decimal::Decimal;

/// A paired row: either both sides present, or one side missing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PairRow {
    /// Left series value.
    pub left: Option<Decimal>,
    /// Right series value.
    pub right: Option<Decimal>,
}

/// Pair two series positionally. The result has `max(a.len(), b.len())`
/// rows; a shorter series yields `None` for the missing tail positions.
pub fn pair_series(left: &[Decimal], right: &[Decimal]) -> Vec<PairRow> {
    let len = left.len().max(right.len());
    (0..len)
        .map(|i| PairRow {
            left: left.get(i).copied(),
            right: right.get(i).copied(),
        })
        .collect()
}

/// Pair two series, truncating to the shorter length (drops the tails).
pub fn pair_series_truncate(left: &[Decimal], right: &[Decimal]) -> Vec<PairRow> {
    let len = left.len().min(right.len());
    (0..len)
        .map(|i| PairRow {
            left: Some(left[i]),
            right: Some(right[i]),
        })
        .collect()
}
