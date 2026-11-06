//! Round-trip tests: bars → candles → chart geometry stays consistent.

use rust_decimal::Decimal;
use vyapaar_core::market::Bar;
use vyapaar_core::types::{Exchange, InstrumentId, Price, Quantity};
use vyapaar_ui_charts::candle::CandlestickChart;
use vyapaar_ui_charts::series::{ChartPoint, ChartSeries};

fn dec(s: &str) -> Decimal {
    s.parse().unwrap()
}

fn make_bars() -> Vec<Bar> {
    let inst = InstrumentId::new(Exchange::Nse, "INFY");
    let data = [("100", "102", "98", "101"), ("101", "105", "100", "104")];
    data.iter()
        .enumerate()
        .map(|(i, (o, h, l, c))| Bar {
            instrument: inst.clone(),
            open: Price::new(dec(o)),
            high: Price::new(dec(h)),
            low: Price::new(dec(l)),
            close: Price::new(dec(c)),
            volume: Quantity::from_u64(100 + i as u64),
            timestamp: vyapaar_core::chrono::Utc::now(),
        })
        .collect()
}

#[test]
fn bars_to_candles_preserves_values() {
    let bars = make_bars();
    let chart = CandlestickChart::from_bars(&bars);
    assert_eq!(chart.candles.len(), bars.len());
    for (candle, bar) in chart.candles.iter().zip(bars.iter()) {
        assert_eq!(candle.open, bar.open.value());
        assert_eq!(candle.high, bar.high.value());
        assert_eq!(candle.low, bar.low.value());
        assert_eq!(candle.close, bar.close.value());
    }
}

#[test]
fn price_range_equals_extreme_minus_extreme() {
    let bars = make_bars();
    let chart = CandlestickChart::from_bars(&bars);
    // min low = 98, max high = 105
    assert_eq!(chart.price_range, dec("7"));
}

#[test]
fn closes_form_a_line_series() {
    let bars = make_bars();
    let points = bars
        .iter()
        .enumerate()
        .map(|(i, b)| ChartPoint::new(Decimal::from(i), b.close.value()))
        .collect();
    let series = ChartSeries::new("close", points);
    assert_eq!(series.y_range().min, dec("101"));
    assert_eq!(series.y_range().max, dec("104"));
}
