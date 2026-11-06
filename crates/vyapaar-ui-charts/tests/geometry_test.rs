//! Tests for full-chart geometry invariants: x/y mapping consistency.

use rust_decimal::Decimal;
use vyapaar_core::market::Bar;
use vyapaar_core::types::{Exchange, InstrumentId, Price, Quantity};
use vyapaar_ui_charts::candle::CandlestickChart;
use vyapaar_ui_charts::series::ChartPoint;

fn dec(s: &str) -> Decimal {
    s.parse().unwrap()
}

fn bars() -> Vec<Bar> {
    let inst = InstrumentId::new(Exchange::Nse, "NIFTY50-INDEX");
    let prices = [
        ("15000", "15050", "14900", "15040"),
        ("15040", "15100", "15010", "15090"),
    ];
    prices
        .iter()
        .enumerate()
        .map(|(i, (o, h, l, c))| Bar {
            instrument: inst.clone(),
            open: Price::new(dec(o)),
            high: Price::new(dec(h)),
            low: Price::new(dec(l)),
            close: Price::new(dec(c)),
            volume: Quantity::from_u64(500 + i as u64),
            timestamp: vyapaar_core::chrono::Utc::now(),
        })
        .collect()
}

#[test]
fn full_chart_maps_high_and_low_to_opposite_edges() {
    let chart = CandlestickChart::from_bars(&bars());
    // The overall max high (15100 in bar 2) is the range top; y should be 0.
    assert_eq!(chart.y_for_price(dec("15100"), 50), Ok(0));
    // The overall min low (14900 in bar 1) is the range bottom; y = 49.
    assert_eq!(chart.y_for_price(dec("14900"), 50), Ok(49));
}

#[test]
fn x_positions_cover_full_width() {
    let chart = CandlestickChart::from_bars(&bars());
    let total = chart.candles.len();
    assert_eq!(CandlestickChart::x_for_candle(0, total, 100), 0);
    assert_eq!(CandlestickChart::x_for_candle(total - 1, total, 100), 99);
}

#[test]
fn line_series_of_highs_is_monotonic() {
    let chart = CandlestickChart::from_bars(&bars());
    let points = chart
        .candles
        .iter()
        .enumerate()
        .map(|(i, c)| ChartPoint::new(Decimal::from(i), c.high))
        .collect::<Vec<_>>();
    for w in points.windows(2) {
        assert!(w[1].y >= w[0].y, "highs must be non-decreasing in fixture");
    }
}

#[test]
fn single_candle_chart_never_panics() {
    let chart = CandlestickChart::from_bars(&bars()[..1]);
    assert_eq!(chart.candles.len(), 1);
    let y = chart.y_for_price(dec("15000"), 10).unwrap();
    assert!(y <= 9);
}
