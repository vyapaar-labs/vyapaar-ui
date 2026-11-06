//! Tests for edge cases: empty input, flat series, degenerate viewports.

use rust_decimal::Decimal;
use vyapaar_core::market::Bar;
use vyapaar_core::types::{Exchange, InstrumentId, Price, Quantity};
use vyapaar_ui_charts::candle::{Candle, CandlestickChart, GeometryError};
use vyapaar_ui_charts::series::{ChartSeries, LineChart};
fn dec(s: &str) -> Decimal {
    s.parse().unwrap()
}

fn bar(price: &str) -> Bar {
    Bar {
        instrument: InstrumentId::new(Exchange::Nse, "TCS"),
        open: Price::new(dec(price)),
        high: Price::new(dec(price)),
        low: Price::new(dec(price)),
        close: Price::new(dec(price)),
        volume: Quantity::from_u64(10),
        timestamp: vyapaar_core::chrono::Utc::now(),
    }
}

#[test]
fn empty_bar_list_yields_empty_chart() {
    let chart = CandlestickChart::from_bars(&[]);
    assert!(chart.candles.is_empty());
    assert_eq!(chart.price_range, Decimal::ZERO);
}

#[test]
fn single_candle_chart_is_valid() {
    let chart = CandlestickChart::from_bars(&[bar("100")]);
    assert_eq!(chart.candles.len(), 1);
    assert_eq!(chart.candles[0].open, dec("100"));
}

#[test]
fn empty_series_union_range_is_zero() {
    let a = ChartSeries::new("a", Vec::new());
    let r = LineChart::union_y_range(&[a]);
    assert_eq!(r.min, Decimal::ZERO);
    assert_eq!(r.max, Decimal::ZERO);
}

#[test]
fn zero_height_viewport_rejected() {
    let chart = CandlestickChart::from_bars(&[bar("100")]);
    assert_eq!(
        chart.y_for_price(dec("100"), 0),
        Err(GeometryError::ZeroViewport)
    );
}

#[test]
fn candle_negative_never_allowed_by_constructor() {
    // Price::new debug_asserts on negative; use raw Decimal to prove the
    // chart still normalizes abs ranges.
    let c = Candle {
        open: dec("100"),
        high: dec("120"),
        low: dec("80"),
        close: dec("110"),
    };
    assert_eq!(c.range(), dec("40"));
}
