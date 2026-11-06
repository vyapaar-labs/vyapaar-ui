//! Algo / strategy management feature.
//!
//! A thin view over `vyapaar-agent-genetic` synthesizers and
//! `vyapaar-strategies`' versioned manifests (per
//! `docs/plans/0005-fincept-terminal-screen-inventory.md` §2 item 5). The host
//! loads manifest rows into this feature; the feature owns selection and
//! status transitions. WASM-safe: no I/O, no tokio.

use serde::{Deserialize, Serialize};
use vyapaar_ui_core::algos::{AlgoManagerViewModel, AlgoRow, AlgoStatus};

/// The algo management feature state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AlgosFeature {
    /// The presentational algo list.
    pub view: AlgoManagerViewModel,
    /// Index of the currently selected row, if any.
    pub selected: Option<usize>,
}

impl AlgosFeature {
    /// Create an empty algo management feature.
    pub fn new() -> Self {
        Self {
            view: AlgoManagerViewModel::empty(),
            selected: None,
        }
    }

    /// Insert or replace an algo row (matched by name).
    ///
    /// Returns `true` when an existing row was replaced.
    pub fn upsert(&mut self, row: AlgoRow) -> bool {
        self.view.upsert(row)
    }

    /// Set the status of the named algo.
    ///
    /// Returns `false` when no algo with that name exists.
    pub fn set_status(&mut self, name: &str, status: AlgoStatus) -> bool {
        self.view.set_status(name, status)
    }

    /// Select the row at `index`, clamped to the list bounds.
    pub fn select(&mut self, index: usize) {
        if self.view.algos.is_empty() {
            self.selected = None;
        } else {
            self.selected = Some(index.min(self.view.algos.len() - 1));
        }
    }

    /// The currently selected row, if any.
    pub fn selected_row(&self) -> Option<&AlgoRow> {
        self.selected.and_then(|i| self.view.algos.get(i))
    }

    /// The number of algos currently live.
    pub fn live_count(&self) -> usize {
        self.view.live_count()
    }
}

impl Default for AlgosFeature {
    fn default() -> Self {
        Self::new()
    }
}
