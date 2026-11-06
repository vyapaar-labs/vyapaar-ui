//! System feature: subsystem health and status panel.

use serde::{Deserialize, Serialize};
use vyapaar_core::types::Exchange;

/// The state of a single subsystem.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubsystemState {
    /// Healthy.
    Healthy,
    /// Degraded.
    Degraded,
    /// Down.
    Down,
}

impl SubsystemState {
    /// Whether the subsystem is usable.
    pub fn is_ok(&self) -> bool {
        matches!(self, Self::Healthy)
    }
}

/// A single subsystem status row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubsystemStatus {
    /// Subsystem name (e.g. `"market"`, `"exec"`).
    pub name: String,
    /// Current state.
    pub state: SubsystemState,
    /// Human-readable detail.
    pub detail: String,
}

/// The system feature state: the set of subsystem status rows plus which
/// exchanges the workstation is watching.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemFeature {
    /// Subsystem statuses.
    pub subsystems: Vec<SubsystemStatus>,
    /// Exchanges tracked by the workstation.
    pub exchanges: Vec<Exchange>,
}

impl SystemFeature {
    /// Create a system feature with the standard subsystem set (all healthy).
    pub fn default_system() -> Self {
        Self {
            subsystems: vec![
                SubsystemStatus {
                    name: "market".into(),
                    state: SubsystemState::Healthy,
                    detail: "feed connected".into(),
                },
                SubsystemStatus {
                    name: "risk".into(),
                    state: SubsystemState::Healthy,
                    detail: "gate armed".into(),
                },
                SubsystemStatus {
                    name: "exec".into(),
                    state: SubsystemState::Healthy,
                    detail: "router ready".into(),
                },
                SubsystemStatus {
                    name: "ledger".into(),
                    state: SubsystemState::Healthy,
                    detail: "journal synced".into(),
                },
                SubsystemStatus {
                    name: "agent".into(),
                    state: SubsystemState::Healthy,
                    detail: "loop idle".into(),
                },
            ],
            exchanges: vec![Exchange::Nse, Exchange::Bse, Exchange::Nfo],
        }
    }

    /// Update a subsystem's state.
    ///
    /// Returns `false` when no subsystem with that name exists.
    pub fn set_subsystem(
        &mut self,
        name: &str,
        state: SubsystemState,
        detail: impl Into<String>,
    ) -> bool {
        let Some(row) = self.subsystems.iter_mut().find(|r| r.name == name) else {
            return false;
        };
        row.state = state;
        row.detail = detail.into();
        true
    }

    /// Whether every subsystem is healthy.
    pub fn all_healthy(&self) -> bool {
        self.subsystems.iter().all(|r| r.state.is_ok())
    }
}
