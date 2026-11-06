//! Audio profiles: descriptors only, no I/O.
//!
//! Playback is the host renderer's concern (Leptos/tone.js, Ratatui beeps,
//! system notifications). The UI layer only selects a [`SoundId`] and reads
//! its profile.

use serde::{Deserialize, Serialize};

/// The closed set of sounds the workstation can request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SoundId {
    /// Order acknowledged by broker.
    OrderAck,
    /// Fill received.
    Fill,
    /// Risk refusal.
    RiskReject,
    /// Kill-switch triggered.
    KillSwitch,
    /// Agent message delivered.
    AgentMessage,
    /// Generic notification.
    Notify,
}

/// A sound profile descriptor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SoundProfile {
    /// Sound id.
    pub id: SoundId,
    /// Approximate tone frequency in Hz (descriptor only).
    pub frequency_hz: u16,
    /// Approximate duration in ms (descriptor only).
    pub duration_ms: u16,
    /// Repeat count.
    pub repeats: u8,
}

impl SoundProfile {
    /// Create a new sound profile.
    pub fn new(id: SoundId, frequency_hz: u16, duration_ms: u16, repeats: u8) -> Self {
        Self {
            id,
            frequency_hz,
            duration_ms,
            repeats,
        }
    }
}

/// Look up the profile for a sound id.
pub fn profile(id: SoundId) -> SoundProfile {
    match id {
        SoundId::OrderAck => SoundProfile::new(id, 660, 90, 1),
        SoundId::Fill => SoundProfile::new(id, 880, 120, 2),
        SoundId::RiskReject => SoundProfile::new(id, 220, 180, 2),
        SoundId::KillSwitch => SoundProfile::new(id, 150, 400, 3),
        SoundId::AgentMessage => SoundProfile::new(id, 520, 100, 1),
        SoundId::Notify => SoundProfile::new(id, 440, 100, 1),
    }
}
