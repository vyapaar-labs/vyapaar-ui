//! Atomic and molecular presentational primitives.
//!
//! These are plain data structs describing *what* to render (label, tone,
//! alignment, spacing) — never *how* to render it. Renderers in
//! `vyapaar-ui-app` (Leptos) and `vyapaar-tui` (Ratatui) translate them into
//! concrete widgets.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Horizontal alignment of text within a cell/column.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextAlign {
    /// Left-aligned.
    Left,
    /// Right-aligned (good for numeric columns).
    Right,
    /// Centered.
    Center,
}

/// Semantic accent used to tint a primitive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Accent {
    /// Neutral / default ink.
    Neutral,
    /// Brand accent (Vyapaar primary).
    Brand,
    /// Positive / up (green in most themes).
    Positive,
    /// Negative / down (red in most themes).
    Negative,
    /// Warning (amber).
    Warning,
    /// Informational (blue).
    Info,
}

/// Tone used for badges and pills.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BadgeTone {
    /// Subtle neutral chip.
    Neutral,
    /// Positive outcome chip.
    Positive,
    /// Negative outcome chip.
    Negative,
    /// Warning chip.
    Warning,
    /// Info chip.
    Info,
}

/// A compact labelled chip.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Badge {
    /// Short label text.
    pub label: String,
    /// Tone.
    pub tone: BadgeTone,
}

impl Badge {
    /// Create a new badge.
    pub fn new(label: impl Into<String>, tone: BadgeTone) -> Self {
        Self {
            label: label.into(),
            tone,
        }
    }
}

/// A status pill with an optional "live pulse" hint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusPill {
    /// Status label (e.g. `"LIVE"`, `"PAPER"`, `"FILLED"`).
    pub label: String,
    /// Tone.
    pub tone: BadgeTone,
    /// Whether renderers should draw a pulsing dot.
    pub pulse: bool,
}

impl StatusPill {
    /// Create a new status pill.
    pub fn new(label: impl Into<String>, tone: BadgeTone) -> Self {
        Self {
            label: label.into(),
            tone,
            pulse: false,
        }
    }

    /// Set the pulse flag.
    pub fn with_pulse(mut self, pulse: bool) -> Self {
        self.pulse = pulse;
        self
    }
}

/// A free-form tag (used for instrument/sector tags).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tag {
    /// Tag text.
    pub text: String,
}

impl Tag {
    /// Create a new tag.
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }
}

/// Direction of a trend indicator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrendDirection {
    /// Up / bullish.
    Up,
    /// Down / bearish.
    Down,
    /// Flat.
    Flat,
}

impl TrendDirection {
    /// Flip an up/down trend.
    pub fn opposite(self) -> Self {
        match self {
            Self::Up => Self::Down,
            Self::Down => Self::Up,
            Self::Flat => Self::Flat,
        }
    }
}

/// A signed price delta expressed as both an absolute value and a percentage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Delta {
    /// Absolute change (signed; `rust_decimal::Decimal`).
    pub absolute: Decimal,
    /// Percentage change (signed).
    pub pct: Decimal,
}

impl Delta {
    /// Create a new delta.
    pub fn new(absolute: Decimal, pct: Decimal) -> Self {
        Self { absolute, pct }
    }

    /// The implied trend direction for this delta.
    pub fn direction(&self) -> TrendDirection {
        if self.absolute.is_zero() {
            TrendDirection::Flat
        } else if self.absolute.is_sign_positive() {
            TrendDirection::Up
        } else {
            TrendDirection::Down
        }
    }
}

/// A trend pill combining a direction arrow with a delta.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrendPill {
    /// Direction.
    pub direction: TrendDirection,
    /// The numeric delta.
    pub delta: Delta,
}

impl TrendPill {
    /// Create a new trend pill from a delta.
    pub fn from_delta(delta: Delta) -> Self {
        Self {
            direction: delta.direction(),
            delta,
        }
    }
}

/// A labelled metric value (key → display label + value).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetricValue {
    /// Stable key (used by renderers for column mapping).
    pub key: String,
    /// Human-readable label.
    pub label: String,
    /// The value.
    pub value: Decimal,
    /// Optional unit suffix (e.g. `"%"`, `"INR"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
}

impl MetricValue {
    /// Create a new metric value.
    pub fn new(key: impl Into<String>, label: impl Into<String>, value: Decimal) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            value,
            unit: None,
        }
    }

    /// Attach a unit.
    pub fn with_unit(mut self, unit: impl Into<String>) -> Self {
        self.unit = Some(unit.into());
        self
    }
}

/// Whitespace/density hint for a container.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Spacing {
    /// Dense layout (terminals, tables).
    Compact,
    /// Standard layout.
    Regular,
    /// Airy layout (cards, home).
    Relaxed,
}

/// A single point on an equity curve: timestamp-ordering key + value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct EquityPoint {
    /// Monotonic index ordering the curve.
    pub index: u32,
    /// Mark-to-market equity value at that point.
    pub value: Decimal,
}

impl EquityPoint {
    /// Create a new equity point.
    pub fn new(index: u32, value: Decimal) -> Self {
        Self { index, value }
    }
}
