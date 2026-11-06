//! Unit tests for `vyapaar-ui-core` screener primitives.

use rust_decimal::Decimal;
use std::collections::BTreeMap;
use vyapaar_core::screener::{FilterOp, MetricKeySpec, MetricPeriod, MetricRef};
use vyapaar_core::types::{Exchange, InstrumentId};
use vyapaar_ui_core::screener::{
    ColumnAlign, ScreenerColumn, ScreenerFilterChip, ScreenerRowModel, ScreenerSort,
    ScreenerSortOrder, ScreenerViewModel,
};

fn dec(s: &str) -> Decimal {
    s.parse().unwrap()
}

fn metric(name: &str) -> MetricRef {
    MetricRef(MetricKeySpec {
        metric: name.to_string(),
        period: MetricPeriod::D1,
    })
}

fn instrument(symbol: &str) -> InstrumentId {
    InstrumentId::new(Exchange::Nse, symbol)
}

#[test]
fn column_align_defaults_left() {
    let col = ScreenerColumn::new("ltp", "LTP");
    assert_eq!(col.align, ColumnAlign::Left);
    assert_eq!(col.width, 5);
}

#[test]
fn column_right_aligned_and_width_clamped() {
    let col = ScreenerColumn::new("ltp", "LTP")
        .right_aligned()
        .with_width(42);
    assert_eq!(col.align, ColumnAlign::Right);
    assert_eq!(col.width, 10);
}

#[test]
fn sort_to_spec_desc() {
    let sort = ScreenerSort {
        metric: metric("rsi"),
        order: ScreenerSortOrder::Desc,
    };
    let spec = sort.to_spec();
    assert!(spec.descending);
    assert_eq!(spec.metric, metric("rsi"));
}

#[test]
fn filter_chip_between_upper_bound() {
    let chip = ScreenerFilterChip::new("pe", FilterOp::Between, dec("10")).with_upper(dec("20"));
    assert_eq!(chip.value_upper, Some(dec("20")));
}

#[test]
fn row_model_metric_lookup() {
    let mut metrics = BTreeMap::new();
    metrics.insert("pe".to_string(), dec("14.5"));
    let row = ScreenerRowModel {
        instrument: instrument("RELIANCE"),
        metrics,
    };
    assert_eq!(row.metric("pe"), Some(dec("14.5")));
    assert_eq!(row.metric("missing"), None);
}

#[test]
fn view_model_empty_and_to_core_rows() {
    let vm = ScreenerViewModel::empty();
    assert!(vm.rows.is_empty());
    assert!(vm.columns.is_empty());
    assert!(!vm.loading);

    let mut vm = vm;
    let mut metrics = BTreeMap::new();
    metrics.insert("pe".to_string(), dec("14.5"));
    vm.rows.push(ScreenerRowModel {
        instrument: instrument("RELIANCE"),
        metrics,
    });

    let core_rows = vm.to_core_rows();
    assert_eq!(core_rows.len(), 1);
    assert_eq!(core_rows[0].instrument, instrument("RELIANCE"));
    assert_eq!(core_rows[0].metrics.get("pe"), Some(&dec("14.5")));
}
