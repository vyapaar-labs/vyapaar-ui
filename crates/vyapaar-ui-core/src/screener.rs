//! Screener table primitives.
//!
//! These presentational types wrap the canonical `vyapaar_core::screener`
//! wire contracts into a form the UI can render: column definitions, filter
//! chips, sort state, and a row/view model pair. No filtering logic lives
//! here — that belongs to the screener feature in `vyapaar-ui-app` — but the
//! presentation of a scan result does.

use std::collections::BTreeMap;

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::atoms::TextAlign;
use vyapaar_core::screener::{FilterOp, MetricRef, SortSpec};
use vyapaar_core::types::InstrumentId;

/// How a screener result column is aligned.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ColumnAlign {
    /// Left.
    Left,
    /// Right.
    Right,
    /// Center.
    Center,
}

impl From<TextAlign> for ColumnAlign {
    fn from(value: TextAlign) -> Self {
        match value {
            TextAlign::Left => Self::Left,
            TextAlign::Right => Self::Right,
            TextAlign::Center => Self::Center,
        }
    }
}

/// A column in the screener result table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScreenerColumn {
    /// Stable column key (usually the metric name).
    pub key: String,
    /// Header label.
    pub header: String,
    /// Alignment for cells in this column.
    pub align: ColumnAlign,
    /// Relative width weight (1–10) used by layout engines.
    pub width: u8,
}

impl ScreenerColumn {
    /// Create a new column.
    pub fn new(key: impl Into<String>, header: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            header: header.into(),
            align: ColumnAlign::Left,
            width: 5,
        }
    }

    /// Right-align the column (numeric).
    pub fn right_aligned(mut self) -> Self {
        self.align = ColumnAlign::Right;
        self
    }

    /// Set the width weight.
    pub fn with_width(mut self, width: u8) -> Self {
        self.width = width.clamp(1, 10);
        self
    }
}

/// Sort order for a single metric.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScreenerSortOrder {
    /// Ascending.
    Asc,
    /// Descending.
    Desc,
}

/// The active sort state of the screener table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScreenerSort {
    /// Metric being sorted on.
    pub metric: MetricRef,
    /// Sort order.
    pub order: ScreenerSortOrder,
}

impl ScreenerSort {
    /// Convert into a canonical [`SortSpec`].
    pub fn to_spec(&self) -> SortSpec {
        SortSpec {
            metric: self.metric.clone(),
            descending: matches!(self.order, ScreenerSortOrder::Desc),
        }
    }
}

/// A rendered filter predicate (one chip in the filter bar).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScreenerFilterChip {
    /// Metric name.
    pub metric: String,
    /// Operator.
    pub op: FilterOp,
    /// Comparison value.
    pub value: Decimal,
    /// Upper bound for [`FilterOp::Between`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value_upper: Option<Decimal>,
}

impl ScreenerFilterChip {
    /// Create a new filter chip.
    pub fn new(metric: impl Into<String>, op: FilterOp, value: Decimal) -> Self {
        Self {
            metric: metric.into(),
            op,
            value,
            value_upper: None,
        }
    }

    /// Set the upper bound (for `Between` filters).
    pub fn with_upper(mut self, upper: Decimal) -> Self {
        self.value_upper = Some(upper);
        self
    }
}

/// One rendered screener result row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScreenerRowModel {
    /// The matching instrument.
    pub instrument: InstrumentId,
    /// Metric name → value.
    pub metrics: BTreeMap<String, Decimal>,
}

impl ScreenerRowModel {
    /// Look up a metric value.
    pub fn metric(&self, name: &str) -> Option<Decimal> {
        self.metrics.get(name).copied()
    }
}

/// The full presentational state of a screener result table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScreenerViewModel {
    /// Column definitions (stable order).
    pub columns: Vec<ScreenerColumn>,
    /// Rows to render.
    pub rows: Vec<ScreenerRowModel>,
    /// Total matched (before limit), for the footer.
    pub total_matched: u32,
    /// Active sort, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sort: Option<ScreenerSort>,
    /// Active filter chips.
    pub filters: Vec<ScreenerFilterChip>,
    /// Whether a scan is in flight.
    pub loading: bool,
}

impl ScreenerViewModel {
    /// Create an empty screener view model.
    pub fn empty() -> Self {
        Self {
            columns: Vec::new(),
            rows: Vec::new(),
            total_matched: 0,
            sort: None,
            filters: Vec::new(),
            loading: false,
        }
    }

    /// Convert rows into canonical `ScreenerRow` values by projecting metric
    /// maps. Returns the canonical rows without losing the instrument.
    pub fn to_core_rows(&self) -> Vec<vyapaar_core::screener::ScreenerRow> {
        self.rows
            .iter()
            .map(|row| vyapaar_core::screener::ScreenerRow {
                instrument: row.instrument.clone(),
                metrics: row.metrics.clone(),
            })
            .collect()
    }
}
