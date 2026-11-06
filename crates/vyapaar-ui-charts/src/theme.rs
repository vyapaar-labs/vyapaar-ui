//! Chart color palette (decimal-invariant, integer RGBA).

use serde::{Deserialize, Serialize};

/// An RGBA color used by chart renderers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChartPalette {
    /// Positive / bullish stroke.
    pub bullish: (u8, u8, u8),
    /// Negative / bearish stroke.
    pub bearish: (u8, u8, u8),
    /// Neutral line stroke.
    pub line: (u8, u8, u8),
    /// Grid stroke.
    pub grid: (u8, u8, u8),
}

impl ChartPalette {
    /// The default palette.
    pub const fn default_palette() -> Self {
        Self {
            bullish: (63, 201, 140),
            bearish: (239, 91, 109),
            line: (79, 195, 247),
            grid: (42, 53, 66),
        }
    }
}

impl Default for ChartPalette {
    fn default() -> Self {
        Self::default_palette()
    }
}
