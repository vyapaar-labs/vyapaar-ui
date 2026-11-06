//! Tests for axis/tick geometry (`vyapaar-ui-charts`).

use rust_decimal::Decimal;
use vyapaar_ui_charts::axis::{ChartRange, tick_positions};

fn dec(s: &str) -> Decimal {
    s.parse().unwrap()
}

#[test]
fn range_span_is_absolute() {
    let r = ChartRange::new(dec("10"), dec("25"));
    assert_eq!(r.span(), dec("15"));
    // Reverse bounds still give a positive span.
    let r = ChartRange::new(dec("25"), dec("10"));
    assert_eq!(r.span(), dec("15"));
}

#[test]
fn normalize_maps_to_unit_interval() {
    let r = ChartRange::new(dec("0"), dec("100"));
    assert_eq!(r.normalize(dec("0")), Decimal::ZERO);
    assert_eq!(r.normalize(dec("100")), Decimal::ONE);
    assert_eq!(r.normalize(dec("50")), dec("0.5"));
    // Values outside the range are clamped.
    assert_eq!(r.normalize(dec("-50")), Decimal::ZERO);
    assert_eq!(r.normalize(dec("150")), Decimal::ONE);
}

#[test]
fn normalize_zero_span_returns_zero() {
    let r = ChartRange::new(dec("5"), dec("5"));
    assert_eq!(r.normalize(dec("5")), Decimal::ZERO);
}

#[test]
fn tick_positions_includes_bounds() {
    let r = ChartRange::new(dec("0"), dec("20"));
    let ticks = tick_positions(r, 5);
    assert_eq!(ticks.values.len(), 5);
    assert_eq!(ticks.values[0], dec("0"));
    assert_eq!(ticks.values[4], dec("20"));
    assert_eq!(ticks.range, r);
}

#[test]
fn tick_positions_minimum_two() {
    let r = ChartRange::new(dec("10"), dec("10"));
    let ticks = tick_positions(r, 0);
    assert!(ticks.values.len() >= 2);
}

#[test]
fn tick_values_are_monotonic() {
    let r = ChartRange::new(dec("100"), dec("200"));
    let ticks = tick_positions(r, 6);
    let mut prev = None;
    for v in &ticks.values {
        if let Some(p) = prev {
            assert!(v >= &p, "ticks not monotonic");
        }
        prev = Some(*v);
    }
}
