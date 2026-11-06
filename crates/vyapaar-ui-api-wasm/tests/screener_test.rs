//! Tests for screener-scan bindings.

use rust_decimal::Decimal;
use std::collections::BTreeMap;
use vyapaar_core::screener::{
    FilterOp, MetricKeySpec, MetricPeriod, MetricRef, ScreenerFilterGroup, ScreenerFilterPredicate,
    ScreenerRow, ScreenerScanRequest, SortSpec,
};
use vyapaar_core::types::{Exchange, InstrumentId};
use vyapaar_ui_api_wasm::screener::{apply_scan, filter_group_matches, row};

fn dec(s: &str) -> Decimal {
    s.parse().unwrap()
}

fn inst(symbol: &str) -> InstrumentId {
    InstrumentId::new(Exchange::Nse, symbol)
}

fn metric(name: &str) -> MetricRef {
    MetricRef(MetricKeySpec {
        metric: name.to_string(),
        period: MetricPeriod::D1,
    })
}

fn row_with(symbol: &str, metrics: &[(&str, &str)]) -> ScreenerRow {
    let mut m = BTreeMap::new();
    for (k, v) in metrics {
        m.insert((*k).to_string(), dec(v));
    }
    row(inst(symbol), m)
}

fn universe() -> Vec<ScreenerRow> {
    vec![
        row_with("RELIANCE", &[("pe", "14.5"), ("roe", "18")]),
        row_with("TCS", &[("pe", "25"), ("roe", "22")]),
        row_with("INFY", &[("pe", "9"), ("roe", "12")]),
    ]
}

#[test]
fn filter_gt_plus_between() {
    let u = universe();
    let request = ScreenerScanRequest {
        universe: None,
        filters: ScreenerFilterGroup {
            predicates: vec![
                ScreenerFilterPredicate {
                    metric: metric("roe"),
                    op: FilterOp::Gt,
                    value: dec("15"),
                    value_upper: None,
                },
                ScreenerFilterPredicate {
                    metric: metric("pe"),
                    op: FilterOp::Between,
                    value: dec("10"),
                    value_upper: Some(dec("30")),
                },
            ],
            match_all: true,
        },
        sort: None,
        limit: None,
    };
    let resp = apply_scan(&u, &request).unwrap();
    // RELIANCE (roe 18, pe 14.5) matches; TCS (roe 22, pe 25) matches;
    // INFY fails roe > 15.
    assert_eq!(resp.rows.len(), 2);
    assert_eq!(resp.total_matched, 2);
    let symbols = resp
        .rows
        .iter()
        .map(|r| r.instrument.symbol.clone())
        .collect::<Vec<_>>();
    assert!(symbols.contains(&"RELIANCE".to_string()));
    assert!(symbols.contains(&"TCS".to_string()));
}

#[test]
fn match_any_group() {
    let u = universe();
    let request = ScreenerScanRequest {
        universe: None,
        filters: ScreenerFilterGroup {
            predicates: vec![
                ScreenerFilterPredicate {
                    metric: metric("pe"),
                    op: FilterOp::Lt,
                    value: dec("10"),
                    value_upper: None,
                },
                ScreenerFilterPredicate {
                    metric: metric("roe"),
                    op: FilterOp::Gt,
                    value: dec("21"),
                    value_upper: None,
                },
            ],
            match_all: false,
        },
        sort: None,
        limit: None,
    };
    let resp = apply_scan(&u, &request).unwrap();
    // INFY (pe 9) OR TCS (roe 22).
    assert_eq!(resp.rows.len(), 2);
}

#[test]
fn sort_desc_and_limit() {
    let u = universe();
    let request = ScreenerScanRequest {
        universe: None,
        filters: ScreenerFilterGroup {
            predicates: Vec::new(),
            match_all: true,
        },
        sort: Some(SortSpec {
            metric: metric("pe"),
            descending: true,
        }),
        limit: Some(2),
    };
    let resp = apply_scan(&u, &request).unwrap();
    assert_eq!(resp.rows.len(), 2);
    // Descending by pe: TCS (25), RELIANCE (14.5) are the top two.
    assert_eq!(resp.rows[0].instrument.symbol, "TCS");
    assert_eq!(resp.rows[1].instrument.symbol, "RELIANCE");
    // total_matched counts before limit.
    assert_eq!(resp.total_matched, 3);
}

#[test]
fn missing_metric_row_fails_filter() {
    let mut u = universe();
    u.push(row_with("HDFCBANK", &[("pe", "20")])); // no roe metric
    let group = ScreenerFilterGroup {
        predicates: vec![ScreenerFilterPredicate {
            metric: metric("roe"),
            op: FilterOp::Gt,
            value: dec("0"),
            value_upper: None,
        }],
        match_all: true,
    };
    assert!(!filter_group_matches(&group, u.last().unwrap()));
}
