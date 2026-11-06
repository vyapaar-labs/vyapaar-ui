//! Apply a screener filter/sort/limit to rows (pure, no I/O).

use std::collections::BTreeMap;

use rust_decimal::Decimal;
use vyapaar_core::screener::{
    FilterOp, ScreenerFilterGroup, ScreenerFilterPredicate, ScreenerRow, ScreenerScanRequest,
    ScreenerScanResponse,
};

/// Errors produced while scanning rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScanError {
    /// A filter references a metric no row in the universe provides.
    MissingMetric,
}

/// Apply a [`ScreenerScanRequest`] to a row universe in memory.
///
/// Rows that fail any filter predicate are excluded; remaining rows are
/// sorted (if a sort spec is present) and truncated to `limit`.
pub fn apply_scan(
    universe: &[ScreenerRow],
    request: &ScreenerScanRequest,
) -> Result<ScreenerScanResponse, ScanError> {
    let matches = universe
        .iter()
        .filter(|row| filter_group_matches(&request.filters, row))
        .cloned()
        .collect::<Vec<_>>();
    let total_matched = matches.len() as u32;

    let mut rows = matches;
    if let Some(sort) = &request.sort {
        let key = sort.metric.0.metric.clone();
        rows.sort_by(|a, b| {
            let av = a.metrics.get(&key).copied().unwrap_or_default();
            let bv = b.metrics.get(&key).copied().unwrap_or_default();
            if sort.descending {
                bv.cmp(&av)
            } else {
                av.cmp(&bv)
            }
        });
    }
    if let Some(limit) = request.limit {
        rows.truncate(limit as usize);
    }
    Ok(ScreenerScanResponse {
        rows,
        total_matched,
    })
}

/// Whether every predicate in a group matches a row.
pub fn filter_group_matches(group: &ScreenerFilterGroup, row: &ScreenerRow) -> bool {
    if group.match_all {
        group.predicates.iter().all(|p| predicate_matches(p, row))
    } else {
        group.predicates.iter().any(|p| predicate_matches(p, row))
    }
}

fn predicate_matches(pred: &ScreenerFilterPredicate, row: &ScreenerRow) -> bool {
    let Some(value) = row.metrics.get(&pred.metric.0.metric) else {
        return false;
    };
    match pred.op {
        FilterOp::Eq => *value == pred.value,
        FilterOp::Ne => *value != pred.value,
        FilterOp::Gt => *value > pred.value,
        FilterOp::Gte => *value >= pred.value,
        FilterOp::Lt => *value < pred.value,
        FilterOp::Lte => *value <= pred.value,
        FilterOp::Between => {
            let upper = pred.value_upper.unwrap_or(pred.value);
            *value >= pred.value && *value <= upper
        }
    }
}

/// Build a `ScreenerRow` helper for tests/consumers.
pub fn row(
    instrument: vyapaar_core::types::InstrumentId,
    metrics: BTreeMap<String, Decimal>,
) -> ScreenerRow {
    ScreenerRow {
        instrument,
        metrics,
    }
}
