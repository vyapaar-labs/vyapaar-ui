//! Strategy Studio: visual node-canvas model.
//!
//! A pure graph model of a strategy: nodes (sources, indicators, signals,
//! orders) and connections between their ports. The host renderer (Leptos
//! canvas, Ratatui) draws this graph; this module owns topology, validation,
//! and layout ordering — all WASM-safe and Decimal-exact.

use std::collections::BTreeMap;

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use vyapaar_core::types::InstrumentId;

/// A node identifier within a canvas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct NodeId(pub u32);

/// What a studio node computes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StudioNodeKind {
    /// Market data source (a symbol + timeframe).
    Source(InstrumentId),
    /// An indicator applied to an upstream series.
    Indicator(String),
    /// A signal/condition node.
    Signal(String),
    /// An order/sizing node.
    Order(String),
    /// A constant / parameter value.
    Constant(Decimal),
}

/// A node in the canvas.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StudioNode {
    /// Unique id within the canvas.
    pub id: NodeId,
    /// Kind and payload.
    pub kind: StudioNodeKind,
    /// Human label.
    pub label: String,
    /// X position (canvas units).
    pub x: u32,
    /// Y position (canvas units).
    pub y: u32,
}

impl StudioNode {
    /// Create a new node.
    pub fn new(id: NodeId, kind: StudioNodeKind, label: impl Into<String>) -> Self {
        Self {
            id,
            kind,
            label: label.into(),
            x: 0,
            y: 0,
        }
    }

    /// Place the node at a canvas position.
    pub fn at(mut self, x: u32, y: u32) -> Self {
        self.x = x;
        self.y = y;
        self
    }
}

/// A directed connection between two nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Connection {
    /// Source node (output).
    pub from: NodeId,
    /// Target node (input).
    pub to: NodeId,
}

/// Errors produced by canvas operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CanvasError {
    /// Referenced a node id that does not exist.
    NodeNotFound,
    /// A connection would form a cycle.
    Cycle,
    /// A duplicate connection already exists.
    DuplicateConnection,
}

/// The studio canvas: an ordered set of nodes and connections.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StudioCanvas {
    /// Nodes keyed by id.
    pub nodes: BTreeMap<NodeId, StudioNode>,
    /// Directed connections.
    pub connections: Vec<Connection>,
}

impl StudioCanvas {
    /// Create an empty canvas.
    pub fn new() -> Self {
        Self {
            nodes: BTreeMap::new(),
            connections: Vec::new(),
        }
    }

    /// Add a node. If the id is taken, the existing node is replaced.
    pub fn add_node(&mut self, node: StudioNode) {
        self.nodes.insert(node.id, node);
    }

    /// Remove a node and every connection touching it.
    pub fn remove_node(&mut self, id: NodeId) {
        self.nodes.remove(&id);
        self.connections.retain(|c| c.from != id && c.to != id);
    }

    /// Connect two nodes, rejecting missing nodes, duplicates, and cycles.
    ///
    /// # Errors
    /// [`CanvasError::NodeNotFound`], [`CanvasError::DuplicateConnection`],
    /// or [`CanvasError::Cycle`].
    pub fn connect(&mut self, from: NodeId, to: NodeId) -> Result<(), CanvasError> {
        if !self.nodes.contains_key(&from) || !self.nodes.contains_key(&to) {
            return Err(CanvasError::NodeNotFound);
        }
        if self.connections.contains(&Connection { from, to }) {
            return Err(CanvasError::DuplicateConnection);
        }
        if self.would_cycle(from, to) {
            return Err(CanvasError::Cycle);
        }
        self.connections.push(Connection { from, to });
        Ok(())
    }

    /// Whether adding an edge `from -> to` would create a cycle — i.e. there
    /// is already a path from `to` back to `from`.
    fn would_cycle(&self, from: NodeId, to: NodeId) -> bool {
        if from == to {
            return true;
        }
        let mut frontier = vec![to];
        let mut seen = vec![to];
        while let Some(cur) = frontier.pop() {
            for c in &self.connections {
                if c.from == cur && !seen.contains(&c.to) {
                    if c.to == from {
                        return true;
                    }
                    seen.push(c.to);
                    frontier.push(c.to);
                }
            }
        }
        false
    }

    /// Number of nodes.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Number of connections.
    pub fn connection_count(&self) -> usize {
        self.connections.len()
    }

    /// Whether every connection references existing nodes.
    pub fn is_consistent(&self) -> bool {
        self.connections
            .iter()
            .all(|c| self.nodes.contains_key(&c.from) && self.nodes.contains_key(&c.to))
    }
}

impl Default for StudioCanvas {
    fn default() -> Self {
        Self::new()
    }
}
