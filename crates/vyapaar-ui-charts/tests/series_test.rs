//! Tests for line/series geometry (`vyapaar-ui-charts`).

use rust_decimal::Decimal;
use vyapaar_ui_charts::series::{ChartPoint, ChartSeries, LineChart};

fn dec(s: &str) -> Decimal {
    s.parse().unwrap()
}

fn pt(x: &str, y: &str) -> ChartPoint {
    ChartPoint::new(dec(x), dec(y))
}

#[test]
fn series_y_range_includes_extremes() {
    let s = ChartSeries::new("eq", vec![pt("0", "10"), pt("1", "20"), pt("2", "5")]);
    let r = s.y_range();
    assert_eq!(r.min, dec("5"));
    assert_eq!(r.max, dec("20"));
}

#[test]
fn empty_series_has_zero_range() {
    let s = ChartSeries::new("eq", Vec::new());
    assert!(s.is_empty());
    let r = s.y_range();
    assert_eq!(r.min, Decimal::ZERO);
    assert_eq!(r.max, Decimal::ZERO);
}

#[test]
fn union_y_range_spans_multiple_series() {
    let a = ChartSeries::new("a", vec![pt("0", "1"), pt("1", "9")]);
    let b = ChartSeries::new("b", vec![pt("0", "4"), pt("1", "2")]);
    let r = LineChart::union_y_range(&[a, b]);
    assert_eq!(r.min, dec("1"));
    assert_eq!(r.max, dec("9"));
}

#[test]
fn line_chart_from_single_series() {
    let s = ChartSeries::new("eq", vec![pt("0", "100"), pt("1", "110")]);
    let chart = LineChart::from_series(s);
    assert_eq!(chart.series.len(), 1);
    assert_eq!(chart.y_range.min, dec("100"));
    assert_eq!(chart.y_range.max, dec("110"));
}

#[test]
fn y_ticks_cover_range() {
    let s = ChartSeries::new("eq", vec![pt("0", "0"), pt("1", "10")]);
    let chart = LineChart::from_series(s);
    let ticks = chart.y_ticks(3);
    assert_eq!(ticks.values.len(), 3);
    assert_eq!(ticks.values[0], dec("0"));
    assert_eq!(ticks.values[2], dec("10"));
}
