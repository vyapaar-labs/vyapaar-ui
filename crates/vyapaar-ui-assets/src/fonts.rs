//! Typography & font asset definitions for Vyapaar UI.
//!
//! Two complementary halves live in this module:
//!
//! 1. **Embedded faces** — the [`FontFamily`] registry and [`FONTS_CSS`]
//!    stylesheet for the 7 self-hosted WOFF/WOFF2 families:
//!    Roboto Mono, IBM Plex Sans, IBM Plex Mono, Silkscreen, Inter,
//!    Source Sans 3, and Noto Sans.
//! 2. **Stack helpers** — [`FontStack`], [`sans_stack`], [`mono_stack`], and
//!    [`FontWeight`] for cross-platform CSS font stacks.

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

/// A cross-platform font stack (ordered fallback families).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FontStack {
    /// Stack name.
    pub name: String,
    /// Ordered font families, first choice first.
    pub families: Vec<String>,
}

impl FontStack {
    /// Create a new font stack.
    pub fn new(name: impl Into<String>, families: Vec<String>) -> Self {
        Self {
            name: name.into(),
            families,
        }
    }

    /// Join families into a CSS `font-family` value.
    pub fn css(&self) -> String {
        self.families
            .iter()
            .map(|f| format!("\"{f}\""))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// Default proportional font stack (UI).
pub fn sans_stack() -> FontStack {
    FontStack::new(
        "sans",
        vec![
            "Inter".into(),
            "Segoe UI".into(),
            "system-ui".into(),
            "-apple-system".into(),
            "sans-serif".into(),
        ],
    )
}

/// Default monospace font stack (terminal / code).
pub fn mono_stack() -> FontStack {
    FontStack::new(
        "mono",
        vec![
            "JetBrains Mono".into(),
            "SFMono-Regular".into(),
            "Menlo".into(),
            "Consolas".into(),
            "monospace".into(),
        ],
    )
}

/// Font weight.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FontWeight {
    /// 400.
    Regular,
    /// 500.
    Medium,
    /// 600.
    Semibold,
    /// 700.
    Bold,
}

impl FontWeight {
    /// Numeric weight value.
    pub const fn value(self) -> u16 {
        match self {
            Self::Regular => 400,
            Self::Medium => 500,
            Self::Semibold => 600,
            Self::Bold => 700,
        }
    }
}
