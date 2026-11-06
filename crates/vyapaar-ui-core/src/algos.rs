//! Algo / strategy management view primitives.
//!
//! A thin presentational mapping of strategy/algo manifests (per
//! `vyapaar-strategies`' versioned manifests and `vyapaar-agent-genetic`'s
//! synthesizers) into rows a screen can render. No domain logic lives here
//! (per `docs/plans/0005-fincept-terminal-screen-inventory.md` §2, the algo
//! management screen is a view over those domain crates).

use serde::{Deserialize, Serialize};

/// Lifecycle status of a deployed algo / strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlgoStatus {
    /// Running and taking signals.
    Active,
    /// Temporarily suspended; no new orders.
    Paused,
    /// Stopped; the manifest is loaded but the loop is not running.
    Stopped,
    /// Failed validation or crashed at runtime.
    Error,
}

impl AlgoStatus {
    /// Whether the algo is currently placing orders.
    pub fn is_live(&self) -> bool {
        matches!(self, Self::Active)
    }
}

/// One rendered algo/strategy management row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AlgoRow {
    /// Algo display name.
    pub name: String,
    /// Versioned manifest version (e.g. `"1.2.0"`).
    pub version: String,
    /// Lifecycle status.
    pub status: AlgoStatus,
    /// One-line description from the manifest.
    pub description: String,
}

impl AlgoRow {
    /// Construct a new algo row.
    pub fn new(name: impl Into<String>, version: impl Into<String>, status: AlgoStatus) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            status,
            description: String::new(),
        }
    }

    /// Attach a one-line description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }
}

/// The full presentational state of the algo/strategy management screen.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AlgoManagerViewModel {
    /// Algos in display order.
    pub algos: Vec<AlgoRow>,
}

impl AlgoManagerViewModel {
    /// An empty management view.
    pub fn empty() -> Self {
        Self { algos: Vec::new() }
    }

    /// Insert or replace an algo row (matched by name), keeping stable order.
    ///
    /// Returns `true` when an existing row was replaced, `false` when a new
    /// row was appended.
    pub fn upsert(&mut self, row: AlgoRow) -> bool {
        if let Some(existing) = self.algos.iter_mut().find(|a| a.name == row.name) {
            *existing = row;
            true
        } else {
            self.algos.push(row);
            false
        }
    }

    /// Set the status of the named algo.
    ///
    /// Returns `false` when no algo with that name exists.
    pub fn set_status(&mut self, name: &str, status: AlgoStatus) -> bool {
        let Some(row) = self.algos.iter_mut().find(|a| a.name == name) else {
            return false;
        };
        row.status = status;
        true
    }

    /// The number of algos currently live.
    pub fn live_count(&self) -> usize {
        self.algos.iter().filter(|a| a.status.is_live()).count()
    }

    /// Whether there are any algos to show.
    pub fn is_empty(&self) -> bool {
        self.algos.is_empty()
    }
}
