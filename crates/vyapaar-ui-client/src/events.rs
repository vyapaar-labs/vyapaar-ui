//! Client event model.

use serde::{Deserialize, Serialize};
use vyapaar_core::market::{Bar, Tick};
use vyapaar_core::order::{FillReport, OrderEvent};
use vyapaar_core::portfolio::PortfolioSnapshot;
use vyapaar_core::screener::ScreenerScanResponse;

/// Coarse classification of a client event, used for routing and icons.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    /// Market data (tick or bar).
    MarketData,
    /// Order lifecycle (ack, status, cancel, reject).
    Order,
    /// A fill arrived.
    Fill,
    /// Portfolio snapshot refresh.
    Portfolio,
    /// Screener scan results.
    Screener,
    /// System health / status.
    System,
}

/// System status payload carried by [`ClientEvent::System`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemStatus {
    /// Whether the connected backend reports healthy.
    pub healthy: bool,
    /// Human-readable status line.
    pub message: String,
}

/// A single client-facing event delivered to the workstation.
///
/// The payloads are the canonical `vyapaar-core` wire contracts so any
/// renderer (WASM app, TUI, headless CLI) consumes the same shapes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum ClientEvent {
    /// A single market tick.
    Tick(Tick),
    /// A completed bar/candle.
    Bar(Bar),
    /// An order lifecycle event (ack/status/cancel/reject).
    Order(OrderEvent),
    /// An execution fill.
    Fill(FillReport),
    /// A portfolio snapshot.
    Portfolio(PortfolioSnapshot),
    /// Screener scan results.
    Screener(ScreenerScanResponse),
    /// System health/status.
    System(SystemStatus),
}

impl ClientEvent {
    /// Classify the event.
    pub fn kind(&self) -> EventKind {
        match self {
            Self::Tick(_) | Self::Bar(_) => EventKind::MarketData,
            Self::Order(_) => EventKind::Order,
            Self::Fill(_) => EventKind::Fill,
            Self::Portfolio(_) => EventKind::Portfolio,
            Self::Screener(_) => EventKind::Screener,
            Self::System(_) => EventKind::System,
        }
    }
}
