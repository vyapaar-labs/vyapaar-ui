//! Unit tests for `vyapaar-ui-core` atoms/molecules primitives.

use rust_decimal::Decimal;
use vyapaar_ui_core::atoms::{Delta, EquityPoint, MetricValue, TrendDirection, TrendPill};

fn dec(s: &str) -> Decimal {
    s.parse().unwrap()
}

#[test]
fn delta_direction_positive() {
    let delta = Delta::new(dec("12.50"), dec("1.25"));
    assert_eq!(delta.direction(), TrendDirection::Up);
}

#[test]
fn delta_direction_negative() {
    let delta = Delta::new(dec("-4.00"), dec("-0.80"));
    assert_eq!(delta.direction(), TrendDirection::Down);
}

#[test]
fn delta_direction_zero_is_flat() {
    let delta = Delta::new(Decimal::ZERO, Decimal::ZERO);
    assert_eq!(delta.direction(), TrendDirection::Flat);
}

#[test]
fn trend_pill_from_delta() {
    let delta = Delta::new(dec("2.00"), dec("0.50"));
    let pill = TrendPill::from_delta(delta);
    assert_eq!(pill.direction, TrendDirection::Up);
    assert_eq!(pill.delta.absolute, dec("2.00"));
}

#[test]
fn trend_direction_opposite() {
    assert_eq!(TrendDirection::Up.opposite(), TrendDirection::Down);
    assert_eq!(TrendDirection::Down.opposite(), TrendDirection::Up);
    assert_eq!(TrendDirection::Flat.opposite(), TrendDirection::Flat);
}

#[test]
fn metric_value_with_unit() {
    let mv = MetricValue::new("roe", "ROE", dec("18.5")).with_unit("%");
    assert_eq!(mv.key, "roe");
    assert_eq!(mv.unit.as_deref(), Some("%"));
}

#[test]
fn equity_points_are_ordered_by_index() {
    let mut points = [
        EquityPoint::new(2, dec("1020")),
        EquityPoint::new(0, dec("1000")),
        EquityPoint::new(1, dec("1010")),
    ];
    points.sort_by_key(|p| p.index);
    assert_eq!(
        points.iter().map(|p| p.value).collect::<Vec<_>>(),
        vec![dec("1000"), dec("1010"), dec("1020")]
    );
}
