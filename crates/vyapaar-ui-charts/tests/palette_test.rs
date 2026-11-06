//! Tests for the chart palette (`vyapaar-ui-charts`).

use vyapaar_ui_charts::theme::ChartPalette;

#[test]
fn default_palette_has_distinct_roles() {
    let p = ChartPalette::default_palette();
    assert_ne!(p.bullish, p.bearish);
    assert_ne!(p.bullish, p.line);
    assert_ne!(p.grid, p.line);
}

#[test]
fn palette_is_default_constructible() {
    let p = ChartPalette::default();
    assert_eq!(p, ChartPalette::default_palette());
}

#[test]
fn bullish_and_bearish_are_greenish_and_reddish() {
    let p = ChartPalette::default_palette();
    // bullish green channel dominant, bearish red channel dominant.
    assert!(p.bullish.1 > p.bullish.0);
    assert!(p.bearish.0 > p.bearish.1);
}
