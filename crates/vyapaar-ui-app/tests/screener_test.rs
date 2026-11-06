//! Tests for the screener feature.

use rust_decimal::Decimal;
use std::collections::BTreeMap;
use vyapaar_core::screener::{FilterOp, ScreenerRow};
use vyapaar_core::types::{Exchange, InstrumentId};
use vyapaar_ui_app::screener::{ScreenerFeature, ScreenerFeatureError};

fn dec(s: &str) -> Decimal {
    s.parse().unwrap()
}

fn inst(symbol: &str) -> InstrumentId {
    InstrumentId::new(Exchange::Nse, symbol)
}

fn row(symbol: &str, pe: &str, roe: &str) -> ScreenerRow {
    let mut m = BTreeMap::new();
    m.insert("pe".to_string(), dec(pe));
    m.insert("roe".to_string(), dec(roe));
    ScreenerRow {
        instrument: inst(symbol),
        metrics: m,
    }
}

#[test]
fn new_feature_starts_empty() {
    let f = ScreenerFeature::new();
    assert!(!f.scanning);
    assert!(f.request.filters.predicates.is_empty());
    assert_eq!(f.request.limit, Some(50));
}

#[test]
fn add_filter_appends_predicate() {
    let mut f = ScreenerFeature::new();
    f.add_filter("pe", FilterOp::Between, dec("10"), Some(dec("30")))
        .unwrap();
    assert_eq!(f.request.filters.predicates.len(), 1);
    assert_eq!(f.request.filters.predicates[0].metric.0.metric, "pe");
}

#[test]
fn add_filter_rejects_empty_metric() {
    let mut f = ScreenerFeature::new();
    assert_eq!(
        f.add_filter("   ", FilterOp::Gt, dec("1"), None),
        Err(ScreenerFeatureError::InvalidMetric)
    );
}

#[test]
fn clear_filters_empties_query_and_chips() {
    let mut f = ScreenerFeature::new();
    f.add_filter("pe", FilterOp::Gt, dec("10"), None).unwrap();
    f.clear_filters();
    assert!(f.request.filters.predicates.is_empty());
    assert!(f.view.filters.is_empty());
}

#[test]
fn run_scan_populates_view_model() {
    let universe = vec![
        row("RELIANCE", "14.5", "18"),
        row("TCS", "25", "22"),
        row("INFY", "9", "12"),
    ];
    let mut f = ScreenerFeature::new();
    f.add_filter("roe", FilterOp::Gt, dec("15"), None).unwrap();
    f.run_scan(&universe);
    assert_eq!(f.view.rows.len(), 2, "RELIANCE + TCS pass roe>15");
    assert_eq!(f.view.total_matched, 2);
    assert!(!f.scanning);
    // columns include the instrument plus each filter metric.
    assert!(f.view.columns.iter().any(|c| c.key == "roe"));
}

#[test]
fn run_scan_with_clear_filter_returns_all() {
    let universe = vec![row("RELIANCE", "14.5", "18")];
    let mut f = ScreenerFeature::new();
    f.run_scan(&universe);
    assert_eq!(f.view.rows.len(), 1);
    assert_eq!(f.view.total_matched, 1);
}
