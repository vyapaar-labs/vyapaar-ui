//! Tests for series pairing bindings.

use rust_decimal::Decimal;
use vyapaar_ui_api_wasm::pairing::{pair_series, pair_series_truncate};

fn dec(s: &str) -> Decimal {
    s.parse().unwrap()
}

#[test]
fn pair_series_equal_lengths() {
    let a = vec![dec("1"), dec("2")];
    let b = vec![dec("3"), dec("4")];
    let rows = pair_series(&a, &b);
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].left, Some(dec("1")));
    assert_eq!(rows[0].right, Some(dec("3")));
}

#[test]
fn pair_series_pads_missing_tail() {
    let a = vec![dec("1"), dec("2"), dec("3")];
    let b = vec![dec("1")];
    let rows = pair_series(&a, &b);
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[2].left, Some(dec("3")));
    assert_eq!(rows[2].right, None);
}

#[test]
fn pair_series_truncate_drops_tail() {
    let a = vec![dec("1"), dec("2"), dec("3")];
    let b = vec![dec("1"), dec("2")];
    let rows = pair_series_truncate(&a, &b);
    assert_eq!(rows.len(), 2);
}

#[test]
fn pair_series_both_empty() {
    let rows = pair_series(&[], &[]);
    assert!(rows.is_empty());
}
