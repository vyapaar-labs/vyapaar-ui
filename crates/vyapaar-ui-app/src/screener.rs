//! Screener feature: reactive scan state over the screener view model.

use serde::{Deserialize, Serialize};
use vyapaar_core::screener::{
    FilterOp, MetricKeySpec, MetricPeriod, MetricRef, ScreenerFilterGroup, ScreenerFilterPredicate,
    ScreenerScanRequest, ScreenerScanResponse,
};
use vyapaar_ui_api_wasm::screener::apply_scan;
use vyapaar_ui_core::screener::{
    ScreenerColumn, ScreenerFilterChip, ScreenerSort, ScreenerViewModel,
};

/// Errors produced by the screener feature.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScreenerFeatureError {
    /// A filter predicate could not be built (empty metric name).
    InvalidMetric,
}

/// The screener feature state: query drafting + result view model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScreenerFeature {
    /// The scan request being drafted.
    pub request: ScreenerScanRequest,
    /// The rendered result table.
    pub view: ScreenerViewModel,
    /// Whether a scan is in flight (host async renders a spinner).
    pub scanning: bool,
}

impl ScreenerFeature {
    /// Create a new screener feature.
    pub fn new() -> Self {
        Self {
            request: ScreenerScanRequest {
                universe: None,
                filters: ScreenerFilterGroup {
                    predicates: Vec::new(),
                    match_all: true,
                },
                sort: None,
                limit: Some(50),
            },
            view: ScreenerViewModel::empty(),
            scanning: false,
        }
    }

    /// Add a filter predicate to the query.
    ///
    /// # Errors
    /// [`ScreenerFeatureError::InvalidMetric`] when the metric name is empty.
    pub fn add_filter(
        &mut self,
        metric: impl Into<String>,
        op: FilterOp,
        value: rust_decimal::Decimal,
        value_upper: Option<rust_decimal::Decimal>,
    ) -> Result<(), ScreenerFeatureError> {
        let name = metric.into();
        if name.trim().is_empty() {
            return Err(ScreenerFeatureError::InvalidMetric);
        }
        self.request
            .filters
            .predicates
            .push(ScreenerFilterPredicate {
                metric: MetricRef(MetricKeySpec {
                    metric: name,
                    period: MetricPeriod::D1,
                }),
                op,
                value,
                value_upper,
            });
        Ok(())
    }

    /// Clear all filter predicates.
    pub fn clear_filters(&mut self) {
        self.request.filters.predicates.clear();
        self.view.filters.clear();
    }

    /// Run the drafted scan against a row universe, producing the view model.
    ///
    /// This is a pure, synchronous computation over the in-memory universe —
    /// it is what the WASM host calls once rows are available.
    pub fn run_scan(&mut self, universe: &[vyapaar_core::screener::ScreenerRow]) {
        self.scanning = true;
        let response = apply_scan(universe, &self.request);
        self.scanning = false;
        if let Ok(resp) = response {
            self.apply_response(resp);
        }
    }

    fn apply_response(&mut self, response: ScreenerScanResponse) {
        let rows = response
            .rows
            .into_iter()
            .map(|r| vyapaar_ui_core::screener::ScreenerRowModel {
                instrument: r.instrument,
                metrics: r.metrics,
            })
            .collect();
        self.view.rows = rows;
        self.view.total_matched = response.total_matched;
        self.view.columns = Self::default_columns(&self.request);
        self.view.filters = self
            .request
            .filters
            .predicates
            .iter()
            .map(|p| ScreenerFilterChip {
                metric: p.metric.0.metric.clone(),
                op: p.op,
                value: p.value,
                value_upper: p.value_upper,
            })
            .collect();
        self.view.sort = self.request.sort.as_ref().map(|s| ScreenerSort {
            metric: s.metric.clone(),
            order: if s.descending {
                vyapaar_ui_core::screener::ScreenerSortOrder::Desc
            } else {
                vyapaar_ui_core::screener::ScreenerSortOrder::Asc
            },
        });
    }

    fn default_columns(request: &ScreenerScanRequest) -> Vec<ScreenerColumn> {
        let mut columns = vec![ScreenerColumn::new("instrument", "Instrument")];
        for pred in &request.filters.predicates {
            let key = pred.metric.0.metric.clone();
            columns.push(ScreenerColumn::new(key.clone(), key).right_aligned());
        }
        columns
    }
}

impl Default for ScreenerFeature {
    fn default() -> Self {
        Self::new()
    }
}
