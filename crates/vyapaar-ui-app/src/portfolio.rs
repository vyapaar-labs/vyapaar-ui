//! Portfolio / positions feature: reactive state over `PortfolioSnapshot`.
//!
//! Consumes the canonical `vyapaar-core` `PortfolioSnapshot` wire contract (as
//! delivered by `vyapaar-ui-client`'s `ClientEvent::Portfolio`) and projects it
//! into the presentational [`PortfolioViewModel`] (per
//! `docs/plans/0005-fincept-terminal-screen-inventory.md` §2 item 4 — a thin
//! view over `vyapaar-ledger-portfolio`). WASM-safe: no I/O, no tokio.

use serde::{Deserialize, Serialize};
use vyapaar_core::portfolio::PortfolioSnapshot;
use vyapaar_ui_core::portfolio::{PortfolioTotals, PortfolioViewModel};

/// The portfolio feature state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortfolioFeature {
    /// The current presentational portfolio view.
    pub view: PortfolioViewModel,
}

impl PortfolioFeature {
    /// Create an empty portfolio feature.
    pub fn new() -> Self {
        Self {
            view: PortfolioViewModel::empty(),
        }
    }

    /// Fold a canonical [`PortfolioSnapshot`] into the view.
    pub fn ingest_snapshot(&mut self, snapshot: &PortfolioSnapshot) {
        self.view = PortfolioViewModel::from_snapshot(snapshot);
    }

    /// Whether the current view has any positions or holdings.
    pub fn has_data(&self) -> bool {
        !self.view.is_empty()
    }

    /// The account-level totals currently shown.
    pub fn totals(&self) -> &PortfolioTotals {
        &self.view.totals
    }
}

impl Default for PortfolioFeature {
    fn default() -> Self {
        Self::new()
    }
}
