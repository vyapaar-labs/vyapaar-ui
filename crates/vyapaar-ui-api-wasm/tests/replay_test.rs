//! Tests for tick-replay bindings.

use vyapaar_core::market::Tick;
use vyapaar_core::types::{Exchange, InstrumentId, Price, Quantity};
use vyapaar_ui_api_wasm::replay::replay_ticks;

fn inst() -> InstrumentId {
    InstrumentId::new(Exchange::Nse, "RELIANCE")
}

fn ts() -> vyapaar_core::chrono::DateTime<vyapaar_core::chrono::Utc> {
    vyapaar_core::chrono::Utc::now()
}

fn ltp(price: i32, vol: u64) -> Tick {
    Tick::Ltp {
        instrument: inst(),
        price: Price::new(price.into()),
        volume: Quantity::from_u64(vol),
        exchange_ts: ts(),
        receive_ts: ts(),
    }
}

#[test]
fn replay_tracks_high_low_last_volume() {
    let ticks = vec![ltp(100, 10), ltp(105, 5), ltp(90, 20)];
    let view = replay_ticks(&ticks).expect("non-empty ticks");
    assert_eq!(view.instrument, inst());
    assert_eq!(view.last_price.value(), 90.into());
    assert_eq!(view.high.value(), 105.into());
    assert_eq!(view.low.value(), 90.into());
    assert_eq!(view.total_volume.value(), 35.into());
}

#[test]
fn replay_empty_returns_none() {
    assert!(replay_ticks(&[]).is_none());
}

#[test]
fn replay_single_tick() {
    let view = replay_ticks(&[ltp(99, 7)]).unwrap();
    assert_eq!(view.high, view.low);
    assert_eq!(view.total_volume.value(), 7.into());
}
