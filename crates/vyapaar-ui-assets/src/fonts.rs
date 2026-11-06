//! Typography & font asset definitions for Vyapaar UI.
//!
//! Provides family constants, weights, and embedded stylesheets for
//! the 7 primary font families:
//! - **Roboto Mono**: Primary financial monospace (order books, tapes, numbers).
//! - **IBM Plex Sans**: Technical UI and dense data table typography.
//! - **IBM Plex Mono**: Alternative code/terminal monospace.
//! - **Silkscreen**: Retro pixel/terminal accent typography.
//! - **Inter**: Clean modern dashboard and natural language typography.
//! - **Source Sans 3**: Editorial, documentation, and narrative analysis typography.
//! - **Noto Sans**: Broad unicode fallback and multilingual display.

use serde::{Deserialize, Serialize};

/// The complete embedded stylesheet registering all self-hosted WOFF and WOFF2 faces.
pub const FONTS_CSS: &str = include_str!("../assets/fonts/fonts.css");

/// Family name constant for Roboto Mono.
pub const FAMILY_ROBOTO_MONO: &str = "Roboto Mono";

/// Family name constant for IBM Plex Sans.
pub const FAMILY_IBM_PLEX_SANS: &str = "IBM Plex Sans";

/// Family name constant for IBM Plex Mono.
pub const FAMILY_IBM_PLEX_MONO: &str = "IBM Plex Mono";

/// Family name constant for Silkscreen.
pub const FAMILY_SILKSCREEN: &str = "Silkscreen";

/// Family name constant for Inter.
pub const FAMILY_INTER: &str = "Inter";

/// Family name constant for Source Sans 3.
pub const FAMILY_SOURCE_SANS_3: &str = "Source Sans 3";

/// Family name constant for Noto Sans.
pub const FAMILY_NOTO_SANS: &str = "Noto Sans";

/// Supported typography families in Vyapaar UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FontFamily {
    RobotoMono,
    IbmPlexSans,
    IbmPlexMono,
    Silkscreen,
    Inter,
    SourceSans3,
    NotoSans,
}

impl FontFamily {
    /// Returns all 7 supported font families in canonical order.
    pub const fn all() -> &'static [Self] {
        &[
            Self::RobotoMono,
            Self::IbmPlexSans,
            Self::IbmPlexMono,
            Self::Silkscreen,
            Self::Inter,
            Self::SourceSans3,
            Self::NotoSans,
        ]
    }

    /// Returns the standard CSS `font-family` name string.
    pub const fn css_name(&self) -> &'static str {
        match self {
            Self::RobotoMono => FAMILY_ROBOTO_MONO,
            Self::IbmPlexSans => FAMILY_IBM_PLEX_SANS,
            Self::IbmPlexMono => FAMILY_IBM_PLEX_MONO,
            Self::Silkscreen => FAMILY_SILKSCREEN,
            Self::Inter => FAMILY_INTER,
            Self::SourceSans3 => FAMILY_SOURCE_SANS_3,
            Self::NotoSans => FAMILY_NOTO_SANS,
        }
    }

    /// Returns the CSS font-family fallback stack.
    pub const fn css_fallback(&self) -> &'static str {
        match self {
            Self::RobotoMono | Self::IbmPlexMono => "monospace, monospace",
            Self::Silkscreen => "cursive, monospace",
            Self::IbmPlexSans | Self::Inter | Self::SourceSans3 | Self::NotoSans => {
                "system-ui, -apple-system, sans-serif"
            }
        }
    }

    /// Full CSS font-family value including quotes and fallback stack.
    pub fn css_font_stack(&self) -> String {
        format!("'{}', {}", self.css_name(), self.css_fallback())
    }

    /// Returns the supported font weights (e.g. 400, 500, 600, 700).
    pub const fn supported_weights(&self) -> &'static [u16] {
        match self {
            Self::Silkscreen => &[400, 700],
            _ => &[400, 500, 600, 700],
        }
    }

    /// Relative directory path under `assets/fonts/`.
    pub const fn relative_dir(&self) -> &'static str {
        match self {
            Self::RobotoMono => "roboto-mono",
            Self::IbmPlexSans => "ibm-plex-sans",
            Self::IbmPlexMono => "ibm-plex-mono",
            Self::Silkscreen => "silkscreen",
            Self::Inter => "inter",
            Self::SourceSans3 => "source-sans-3",
            Self::NotoSans => "noto-sans",
        }
    }
}
