//! Candlestick chart geometry.

use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};
use vyapaar_core::market::Bar;
use vyapaar_core::types::Price;

/// A single candlestick (OHLC) in exact Decimal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Candle {
    /// Open price.
    pub open: Decimal,
    /// High price.
    pub high: Decimal,
    /// Low price.
    pub low: Decimal,
    /// Close price.
    pub close: Decimal,
}

impl Candle {
    /// Create a new candle from four prices.
    pub fn new(open: Price, high: Price, low: Price, close: Price) -> Self {
        Self {
            open: open.value(),
            high: high.value(),
            low: low.value(),
            close: close.value(),
        }
    }

    /// Whether this is a bullish (close >= open) candle.
    pub fn is_bullish(&self) -> bool {
        self.close >= self.open
    }

    /// Body (|open − close|).
    pub fn body(&self) -> Decimal {
        (self.close - self.open).abs()
    }

    /// Total wick/range (high − low).
    pub fn range(&self) -> Decimal {
        (self.high - self.low).abs()
    }
}

/// Conversion of a canonical [`Bar`] into a [`Candle`].
impl From<&Bar> for Candle {
    fn from(bar: &Bar) -> Self {
        Self::new(bar.open, bar.high, bar.low, bar.close)
    }
}

/// Errors produced while laying candles out into a viewport.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GeometryError {
    /// Empty input series.
    EmptySeries,
    /// The viewport has zero height or width.
    ZeroViewport,
    /// The price range has zero span (flat series) — cannot map prices.
    FlatPriceRange,
}

impl core::fmt::Display for GeometryError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::EmptySeries => write!(f, "chart series is empty"),
            Self::ZeroViewport => write!(f, "chart viewport has zero size"),
            Self::FlatPriceRange => write!(f, "chart price range is flat"),
        }
    }
}

/// A candlestick chart laid out into a pixel viewport.
///
/// Geometry is computed in exact Decimal and converted to integer pixel
/// coordinates only at the final rounding step — no intermediate floats.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CandlestickChart {
    /// Candles in display order (left → right).
    pub candles: Vec<Candle>,
    /// Full price range covered by the series.
    pub price_range: Decimal,
}

impl CandlestickChart {
    /// Build a chart from canonical bars.
    pub fn from_bars(bars: &[Bar]) -> Self {
        let candles = bars.iter().map(Candle::from).collect::<Vec<_>>();
        let price_range = Self::price_range(&candles);
        Self { candles, price_range }
    }

    /// The overall min/max price span of a candle series.
    pub fn price_range(candles: &[Candle]) -> Decimal {
        let mut min = None;
        let mut max = None;
        for c in candles {
            for p in [c.open, c.high, c.low, c.close] {
                match min {
                    None => min = Some(p),
                    Some(m) if p < m => min = Some(p),
                    _ => {}
                }
                match max {
                    None => max = Some(p),
                    Some(m) if p > m => max = Some(p),
                    _ => {}
                }
            }
        }
        match (min, max) {
            (Some(lo), Some(hi)) => hi - lo,
            _ => Decimal::ZERO,
        }
    }

    /// Map a price to a `y` pixel within `[0, height)`, using `price_range`.
    ///
    /// Returns [`GeometryError::FlatPriceRange`] when the series is flat.
    pub fn y_for_price(&self, price: Decimal, height: u32) -> Result<u16, GeometryError> {
        if height == 0 {
            return Err(GeometryError::ZeroViewport);
        }
        if self.candles.is_empty() {
            return Err(GeometryError::EmptySeries);
        }
        // Determine the min/max across candles.
        let mut min = None;
        let mut max = None;
        for c in &self.candles {
            for p in [c.open, c.high, c.low, c.close] {
                min = Some(min.map_or(p, |m: Decimal| m.min(p)));
                max = Some(max.map_or(p, |m: Decimal| m.max(p)));
            }
        }
        let (Some(lo), Some(hi)) = (min, max) else {
            return Err(GeometryError::FlatPriceRange);
        };
        let span = hi - lo;
        if span.is_zero() {
            return Err(GeometryError::FlatPriceRange);
        }
        let height_d = Decimal::from(height.saturating_sub(1));
        let ratio = (price - lo) / span;
        // y grows downward: min price → bottom (height-1), max price → 0.
        let y = height_d - (ratio * height_d);
        let y = y.round();
        Ok(y
            .to_u16()
            .unwrap_or(0)
            .min(u16::try_from(height.saturating_sub(1)).unwrap_or(u16::MAX)))
    }

    /// The `x` pixel for candle `index` of `total` within `[0, width)`.
    pub fn x_for_candle(index: usize, total: usize, width: u32) -> u16 {
        if total == 0 || width == 0 {
            return 0;
        }
        let width_d = Decimal::from(width.saturating_sub(1));
        let total_d = Decimal::from(total.saturating_sub(1).max(1));
        let x = width_d * Decimal::from(index) / total_d;
        x.round()
            .to_u16()
            .unwrap_or(0)
            .min(u16::try_from(width.saturating_sub(1)).unwrap_or(u16::MAX))
    }
}
