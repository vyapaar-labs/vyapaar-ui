//! Axis range and tick geometry.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// A value range with explicit bounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChartRange {
    /// Inclusive minimum.
    pub min: Decimal,
    /// Inclusive maximum.
    pub max: Decimal,
}

impl ChartRange {
    /// Create a new range.
    pub fn new(min: Decimal, max: Decimal) -> Self {
        Self { min, max }
    }

    /// The span `max - min` (non-negative).
    pub fn span(&self) -> Decimal {
        (self.max - self.min).abs()
    }

    /// Normalize a value into `0..=1` within this range.
    pub fn normalize(&self, value: Decimal) -> Decimal {
        let span = self.span();
        if span.is_zero() {
            return Decimal::ZERO;
        }
        ((value - self.min) / span).clamp(Decimal::ZERO, Decimal::ONE)
    }
}

/// Tick marks for an axis.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AxisTicks {
    /// Tick values.
    pub values: Vec<Decimal>,
    /// The range they cover.
    pub range: ChartRange,
}

/// Produce `n` evenly-spaced tick values over `range`.
///
/// The number of returned ticks is `n.max(2)` and always includes both the
/// range minimum and maximum.
pub fn tick_positions(range: ChartRange, n: usize) -> AxisTicks {
    let count = n.max(2);
    let span = range.span();
    let mut values = Vec::with_capacity(count);
    if span.is_zero() {
        values.push(range.min);
        values.push(range.max);
        return AxisTicks { values, range };
    }
    let step = span / Decimal::from(count.saturating_sub(1));
    for i in 0..count {
        let offset = step * Decimal::from(i);
        values.push((range.min + offset).round_dp(2));
    }
    // Guarantee the range bounds are present as first/last.
    if count > 0 {
        values[0] = range.min;
        values[count - 1] = range.max;
    }
    AxisTicks { values, range }
}
