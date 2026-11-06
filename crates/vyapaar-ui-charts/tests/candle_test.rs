//! Tests for candlestick geometry (`vyapaar-ui-charts`).

use rust_decimal::Decimal;
use vyapaar_core::market::Bar;
use vyapaar_core::types::{Exchange, InstrumentId, Price, Quantity};
use vyapaar_ui_charts::candle::{Candle, CandlestickChart, GeometryError};

fn dec(s: &str) -> Decimal {
    s.parse().unwrap()
}

fn bar(open: &str, high: &str, low: &str, close: &str) -> Bar {
    Bar {
        instrument: InstrumentId::new(Exchange::Nse, "RELIANCE"),
        open: Price::new(dec(open)),
        high: Price::new(dec(high)),
        low: Price::new(dec(low)),
        close: Price::new(dec(close)),
        volume: Quantity::from_u64(1000),
        timestamp: vyapaar_core::chrono::Utc::now(),
    }
}

#[test]
fn candle_from_bar_maps_ohlc() {
    let b = bar("100", "105", "99", "104");
    let c = Candle::from(&b);
    assert_eq!(c.open, dec("100"));
    assert_eq!(c.high, dec("105"));
    assert_eq!(c.low, dec("99"));
    assert_eq!(c.close, dec("104"));
}

#[test]
fn candle_direction_and_ranges() {
    let bull = Candle::new(
        Price::new(dec("100")),
        Price::new(dec("105")),
        Price::new(dec("99")),
        Price::new(dec("104")),
    );
    assert!(bull.is_bullish());
    assert_eq!(bull.body(), dec("4"));
    assert_eq!(bull.range(), dec("6"));

    let bear = Candle::new(
        Price::new(dec("100")),
        Price::new(dec("101")),
        Price::new(dec("90")),
        Price::new(dec("92")),
    );
    assert!(!bear.is_bullish());
    assert_eq!(bear.body(), dec("8"));
}

#[test]
fn chart_from_bars_preserves_order() {
    let bars = vec![
        bar("100", "105", "99", "104"),
        bar("104", "108", "103", "107"),
    ];
    let chart = CandlestickChart::from_bars(&bars);
    assert_eq!(chart.candles.len(), 2);
    assert_eq!(chart.candles[1].close, dec("107"));
}

#[test]
fn price_range_covers_all_extremes() {
    let candles = vec![
        Candle::new(
            Price::new(dec("100")),
            Price::new(dec("110")),
            Price::new(dec("90")),
            Price::new(dec("105")),
        ),
        Candle::new(
            Price::new(dec("105")),
            Price::new(dec("120")),
            Price::new(dec("95")),
            Price::new(dec("115")),
        ),
    ];
    assert_eq!(CandlestickChart::price_range(&candles), dec("30"));
}

#[test]
fn y_for_price_maps_high_to_top() {
    let candles = vec![Candle::new(
        Price::new(dec("100")),
        Price::new(dec("110")),
        Price::new(dec("90")),
        Price::new(dec("105")),
    )];
    let chart = CandlestickChart {
        candles,
        price_range: dec("20"),
    };
    let h = 100u32;
    // max price (110) → y=0
    assert_eq!(chart.y_for_price(dec("110"), h), Ok(0));
    // min price (90) → y=height-1
    assert_eq!(chart.y_for_price(dec("90"), h), Ok(99));
    // mid price (100) → y≈50
    let mid = chart.y_for_price(dec("100"), h).unwrap();
    assert!((45..=55).contains(&mid), "mid y out of band: {mid}");
}

#[test]
fn y_for_price_errors() {
    let chart = CandlestickChart::from_bars(&[]);
    assert_eq!(
        chart.y_for_price(dec("100"), 100),
        Err(GeometryError::EmptySeries)
    );

    let bars = vec![bar("100", "105", "99", "104")];
    let chart = CandlestickChart::from_bars(&bars);
    assert_eq!(
        chart.y_for_price(dec("100"), 0),
        Err(GeometryError::ZeroViewport)
    );

    let flat = vec![bar("100", "100", "100", "100")];
    let chart = CandlestickChart::from_bars(&flat);
    assert_eq!(
        chart.y_for_price(dec("100"), 100),
        Err(GeometryError::FlatPriceRange)
    );
}

#[test]
fn x_for_candle_distributes_evenly() {
    assert_eq!(CandlestickChart::x_for_candle(0, 4, 100), 0);
    assert_eq!(CandlestickChart::x_for_candle(3, 4, 100), 99);
    let mid = CandlestickChart::x_for_candle(1, 4, 100);
    assert!((30..=36).contains(&mid), "mid x out of band: {mid}");
    // Degenerate inputs are clamped to 0.
    assert_eq!(CandlestickChart::x_for_candle(0, 0, 100), 0);
    assert_eq!(CandlestickChart::x_for_candle(0, 4, 0), 0);
}
