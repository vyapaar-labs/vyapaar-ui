//! Line / series chart geometry.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::axis::{ChartRange, tick_positions, AxisTicks};

/// A single x/y point in a series (both exact Decimal).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChartPoint {
    /// X coordinate (usually bar index or timestamp-ordered key).
    pub x: Decimal,
    /// Y value.
    pub y: Decimal,
}

impl ChartPoint {
    /// Create a new point.
    pub fn new(x: Decimal, y: Decimal) -> Self {
        Self { x, y }
    }
}

/// A named series of points.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChartSeries {
    /// Series name.
    pub name: String,
    /// Points in display order.
    pub points: Vec<ChartPoint>,
}

impl ChartSeries {
    /// Create a new named series.
    pub fn new(name: impl Into<String>, points: Vec<ChartPoint>) -> Self {
        Self {
            name: name.into(),
            points,
        }
    }

    /// Compute the inclusive y-range across all points.
    pub fn y_range(&self) -> ChartRange {
        let mut min = None;
        let mut max = None;
        for p in &self.points {
            min = Some(min.map_or(p.y, |m: Decimal| m.min(p.y)));
            max = Some(max.map_or(p.y, |m: Decimal| m.max(p.y)));
        }
        match (min, max) {
            (Some(lo), Some(hi)) => ChartRange::new(lo, hi),
            _ => ChartRange::new(Decimal::ZERO, Decimal::ZERO),
        }
    }

    /// Whether the series has no points.
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }
}

/// A line chart laid out over a pixel viewport.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LineChart {
    /// The series to draw.
    pub series: Vec<ChartSeries>,
    /// Y-axis range.
    pub y_range: ChartRange,
}

impl LineChart {
    /// Build a chart from a single series.
    pub fn from_series(series: ChartSeries) -> Self {
        let y_range = series.y_range();
        Self {
            series: vec![series],
            y_range,
        }
    }

    /// Compute the union y-range across all series.
    pub fn union_y_range(series: &[ChartSeries]) -> ChartRange {
        let mut min = None;
        let mut max = None;
        for s in series {
            let r = s.y_range();
            min = Some(min.map_or(r.min, |m: Decimal| m.min(r.min)));
            max = Some(max.map_or(r.max, |m: Decimal| m.max(r.max)));
        }
        match (min, max) {
            (Some(lo), Some(hi)) => ChartRange::new(lo, hi),
            _ => ChartRange::new(Decimal::ZERO, Decimal::ZERO),
        }
    }

    /// Y-axis ticks covering [`Self::y_range`].
    pub fn y_ticks(&self, n: usize) -> AxisTicks {
        tick_positions(self.y_range, n)
    }
}
