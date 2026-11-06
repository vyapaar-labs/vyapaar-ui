//! Tests for the WASM-safe client event stream.

use vyapaar_core::market::Tick;
use vyapaar_core::order::{OrderAck, OrderEvent, OrderStatus};
use vyapaar_core::types::{Exchange, InstrumentId, Price, Quantity};
use vyapaar_ui_client::events::{ClientEvent, EventKind, SystemStatus};
use vyapaar_ui_client::stream::EventStream;

fn instrument() -> InstrumentId {
    InstrumentId::new(Exchange::Nse, "RELIANCE")
}

fn tick() -> Tick {
    Tick::Ltp {
        instrument: instrument(),
        price: Price::new(10.into()),
        volume: Quantity::from_u64(1),
        exchange_ts: vyapaar_core::chrono::Utc::now(),
        receive_ts: vyapaar_core::chrono::Utc::now(),
    }
}

#[test]
fn event_kind_classification() {
    assert_eq!(ClientEvent::Tick(tick()).kind(), EventKind::MarketData);
    assert_eq!(ClientEvent::System(SystemStatus { healthy: true, message: "ok".into() }).kind(), EventKind::System);
}

#[test]
fn subscriber_receives_published_events_in_order() {
    let mut stream = EventStream::new();
    let sub = stream.subscribe();
    stream.publish(ClientEvent::Tick(tick()));
    stream.publish(ClientEvent::System(SystemStatus { healthy: true, message: "ok".into() }));
    let events = sub.drain();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].kind(), EventKind::MarketData);
    assert_eq!(events[1].kind(), EventKind::System);
}

#[test]
fn subscriber_starting_after_publish_misses_earlier_events() {
    let mut stream = EventStream::new();
    stream.publish(ClientEvent::Tick(tick()));
    let sub = stream.subscribe();
    assert!(sub.drain().is_empty());
    stream.publish(ClientEvent::Tick(tick()));
    assert_eq!(sub.drain().len(), 1);
}

#[test]
fn subscription_drop_unsubscribes() {
    let mut stream = EventStream::new();
    assert_eq!(stream.subscriber_count(), 0);
    let sub = stream.subscribe();
    assert_eq!(stream.subscriber_count(), 1);
    drop(sub);
    assert_eq!(stream.subscriber_count(), 0);
}

#[test]
fn published_count_monotonic() {
    let mut stream = EventStream::new();
    assert_eq!(stream.published(), 0);
    stream.publish(ClientEvent::Tick(tick()));
    stream.publish(ClientEvent::Tick(tick()));
    assert_eq!(stream.published(), 2);
}

#[test]
fn order_event_payload_passes_through() {
    let mut stream = EventStream::new();
    let sub = stream.subscribe();
    stream.publish(ClientEvent::Order(OrderEvent::Acked(OrderAck::new(
        "order-1",
        vyapaar_core::uuid::Uuid::new_v4(),
        OrderStatus::New,
        vyapaar_core::chrono::Utc::now(),
    ))));
    let events = sub.drain();
    match &events[0] {
        ClientEvent::Order(OrderEvent::Acked(ack)) => {
            assert_eq!(ack.order_id, "order-1");
            assert_eq!(ack.status, OrderStatus::New);
        }
        other => panic!("expected order ack event, got {other:?}"),
    }
}

#[test]
fn many_publishes_retain_recent_events() {
    let mut stream = EventStream::new();
    let sub = stream.subscribe();
    for _ in 0..5000 {
        stream.publish(ClientEvent::Tick(tick()));
    }
    // The log retains the most recent 4096 events; older ones are dropped.
    assert_eq!(sub.drain().len(), 4096);
    // A subscriber that missed the early events sees the retained tail.
    let late = stream.subscribe();
    stream.publish(ClientEvent::Tick(tick()));
    assert_eq!(late.drain().len(), 1);
}
