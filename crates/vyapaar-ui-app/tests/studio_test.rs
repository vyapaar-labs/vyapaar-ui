//! Tests for the Strategy Studio canvas model.

use rust_decimal::Decimal;
use vyapaar_core::types::{Exchange, InstrumentId};
use vyapaar_ui_app::studio::{CanvasError, Connection, NodeId, StudioCanvas, StudioNode, StudioNodeKind};

#[test]
fn empty_canvas() {
    let c = StudioCanvas::new();
    assert_eq!(c.node_count(), 0);
    assert_eq!(c.connection_count(), 0);
    assert!(c.is_consistent());
}

#[test]
fn add_and_connect_nodes() {
    let mut c = StudioCanvas::new();
    let source = StudioNode::new(
        NodeId(1),
        StudioNodeKind::Source(InstrumentId::new(Exchange::Nse, "RELIANCE")),
        "RELIANCE source",
    );
    let sma = StudioNode::new(NodeId(2), StudioNodeKind::Indicator("SMA".into()), "SMA(10)");
    c.add_node(source);
    c.add_node(sma);
    c.connect(NodeId(1), NodeId(2)).unwrap();
    assert_eq!(c.connection_count(), 1);
    assert!(c.is_consistent());
    assert_eq!(c.connections[0], Connection { from: NodeId(1), to: NodeId(2) });
}

#[test]
fn remove_node_prunes_connections() {
    let mut c = StudioCanvas::new();
    c.add_node(StudioNode::new(NodeId(1), StudioNodeKind::Constant(Decimal::from(1)), "c"));
    c.add_node(StudioNode::new(NodeId(2), StudioNodeKind::Constant(Decimal::from(2)), "d"));
    c.connect(NodeId(1), NodeId(2)).unwrap();
    c.remove_node(NodeId(1));
    assert_eq!(c.node_count(), 1);
    assert_eq!(c.connection_count(), 0);
}

#[test]
fn connect_missing_node_errors() {
    let mut c = StudioCanvas::new();
    c.add_node(StudioNode::new(NodeId(1), StudioNodeKind::Constant(Decimal::from(1)), "c"));
    assert_eq!(c.connect(NodeId(1), NodeId(99)), Err(CanvasError::NodeNotFound));
}

#[test]
fn duplicate_connection_errors() {
    let mut c = StudioCanvas::new();
    c.add_node(StudioNode::new(NodeId(1), StudioNodeKind::Constant(Decimal::from(1)), "a"));
    c.add_node(StudioNode::new(NodeId(2), StudioNodeKind::Constant(Decimal::from(2)), "b"));
    c.connect(NodeId(1), NodeId(2)).unwrap();
    assert_eq!(c.connect(NodeId(1), NodeId(2)), Err(CanvasError::DuplicateConnection));
}

#[test]
fn cycles_are_rejected() {
    let mut c = StudioCanvas::new();
    for id in 1..=3u32 {
        c.add_node(StudioNode::new(NodeId(id), StudioNodeKind::Constant(Decimal::from(id)), "n"));
    }
    c.connect(NodeId(1), NodeId(2)).unwrap();
    c.connect(NodeId(2), NodeId(3)).unwrap();
    // 3 -> 1 would close a cycle.
    assert_eq!(c.connect(NodeId(3), NodeId(1)), Err(CanvasError::Cycle));
    // A forward 1 -> 3 is fine (already implicitly reachable but not a cycle).
    c.connect(NodeId(1), NodeId(3)).unwrap();
    assert_eq!(c.connection_count(), 3);
}

#[test]
fn self_connection_is_a_cycle() {
    let mut c = StudioCanvas::new();
    c.add_node(StudioNode::new(NodeId(7), StudioNodeKind::Constant(Decimal::from(7)), "x"));
    assert_eq!(c.connect(NodeId(7), NodeId(7)), Err(CanvasError::Cycle));
}
