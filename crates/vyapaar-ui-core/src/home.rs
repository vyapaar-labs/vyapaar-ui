//! Home dashboard primitives: market status, watchlist, and account summary.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::atoms::Delta;
use vyapaar_core::types::{InstrumentId, Money, Price};

/// Market session status shown on the home page.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MarketStatusKind {
    /// Trading session is open.
    Open,
    /// Trading session is closed.
    Closed,
    /// Pre-market.
    PreMarket,
    /// Post-market.
    PostMarket,
}

/// A market status widget.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarketStatus {
    /// Session kind.
    pub kind: MarketStatusKind,
    /// Human-readable session label (e.g. `"09:15 – 15:30 IST"`).
    pub session_label: String,
    /// Optional next-session hint (e.g. `"Reopens 09:15 IST"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_session: Option<String>,
}

impl MarketStatus {
    /// Create a new market status widget.
    pub fn new(kind: MarketStatusKind, session_label: impl Into<String>) -> Self {
        Self {
            kind,
            session_label: session_label.into(),
            next_session: None,
        }
    }

    /// Set the next-session hint.
    pub fn with_next_session(mut self, hint: impl Into<String>) -> Self {
        self.next_session = Some(hint.into());
        self
    }

    /// Whether the market is open.
    pub fn is_open(&self) -> bool {
        matches!(self.kind, MarketStatusKind::Open)
    }
}

/// One row of the home watchlist.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WatchlistRow {
    /// The instrument.
    pub instrument: InstrumentId,
    /// Company / scrip short name.
    pub name: String,
    /// Last traded price.
    pub last_price: Price,
    /// Day delta (absolute + pct).
    pub delta: Delta,
    /// Small sparkline of recent closes (oldest first).
    pub spark: Vec<Decimal>,
}

impl WatchlistRow {
    /// Create a new watchlist row.
    pub fn new(
        instrument: InstrumentId,
        name: impl Into<String>,
        last_price: Price,
        delta: Delta,
    ) -> Self {
        Self {
            instrument,
            name: name.into(),
            last_price,
            delta,
            spark: Vec::new(),
        }
    }

    /// Attach a sparkline.
    pub fn with_spark(mut self, spark: Vec<Decimal>) -> Self {
        self.spark = spark;
        self
    }
}

/// Account-level summary card on the home page.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountSummary {
    /// Free cash balance.
    pub cash: Money,
    /// Margin used.
    pub margin_used: Money,
    /// Margin available.
    pub margin_available: Money,
    /// Total P&L (realized + unrealized).
    pub total_pnl: Money,
    /// Net worth.
    pub net_worth: Money,
}

impl AccountSummary {
    /// Create a new account summary.
    pub fn new(
        cash: Money,
        margin_used: Money,
        margin_available: Money,
        total_pnl: Money,
        net_worth: Money,
    ) -> Self {
        Self {
            cash,
            margin_used,
            margin_available,
            total_pnl,
            net_worth,
        }
    }
}

/// The full watchlist/dashboard view: rows, market status, and account card.
///
/// A thin presentational model over `vyapaar-market` live feed data (per
/// `docs/plans/0005-fincept-terminal-screen-inventory.md` §2 item 1): it holds
/// whatever the feed has most recently delivered and renders it; it computes
/// nothing itself.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WatchlistViewModel {
    /// Watchlist rows, in display order.
    pub rows: Vec<WatchlistRow>,
    /// Market session status, if the feed has reported one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<MarketStatus>,
    /// Account summary card, if one is available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub account: Option<AccountSummary>,
}

impl WatchlistViewModel {
    /// An empty watchlist view model.
    pub fn empty() -> Self {
        Self {
            rows: Vec::new(),
            status: None,
            account: None,
        }
    }

    /// Insert or replace a row (matched by instrument), keeping stable order.
    ///
    /// Returns `true` when an existing row was replaced, `false` when a new
    /// row was appended.
    pub fn upsert(&mut self, row: WatchlistRow) -> bool {
        if let Some(existing) = self
            .rows
            .iter_mut()
            .find(|r| r.instrument == row.instrument)
        {
            *existing = row;
            true
        } else {
            self.rows.push(row);
            false
        }
    }

    /// Remove the row for `instrument`.
    ///
    /// Returns `false` when no such row existed.
    pub fn remove(&mut self, instrument: &InstrumentId) -> bool {
        let before = self.rows.len();
        self.rows.retain(|r| &r.instrument != instrument);
        self.rows.len() != before
    }

    /// The number of rows currently shown.
    pub fn row_count(&self) -> usize {
        self.rows.len()
    }
}
