//! Theme palettes for the Vyapaar workstation.

use serde::{Deserialize, Serialize};

/// An RGB(A) color.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Color {
    /// Red channel (0–255).
    pub r: u8,
    /// Green channel (0–255).
    pub g: u8,
    /// Blue channel (0–255).
    pub b: u8,
    /// Alpha channel (0–255); 255 is opaque.
    pub a: u8,
}

impl Color {
    /// Create an opaque color.
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    /// Create a color with an explicit alpha.
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// CSS `#rrggbb` hex string.
    pub fn to_hex(&self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }

    /// CSS `rgba(r, g, b, a)` string with alpha normalized to `0..=1`.
    pub fn to_css_rgba(&self) -> String {
        format!(
            "rgba({}, {}, {}, {:.2})",
            self.r,
            self.g,
            self.b,
            f64::from(self.a) / 255.0
        )
    }
}

/// Named workstation themes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThemeId {
    /// Dark terminal theme (default).
    Dark,
    /// Light theme.
    Light,
    /// Amber terminal theme.
    Amber,
}

/// A full color palette for a theme.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Theme {
    /// Theme identifier.
    pub id: ThemeId,
    /// Background.
    pub background: Color,
    /// Surface / card background.
    pub surface: Color,
    /// Border color.
    pub border: Color,
    /// Foreground / primary text.
    pub foreground: Color,
    /// Muted secondary text.
    pub muted: Color,
    /// Brand accent.
    pub brand: Color,
    /// Positive / up.
    pub positive: Color,
    /// Negative / down.
    pub negative: Color,
    /// Warning.
    pub warning: Color,
    /// Informational.
    pub info: Color,
}

/// All themes in a stable order.
pub fn all_themes() -> [Theme; 3] {
    [theme(ThemeId::Dark), theme(ThemeId::Light), theme(ThemeId::Amber)]
}

/// Look up a theme by id.
pub fn theme(id: ThemeId) -> Theme {
    match id {
        ThemeId::Dark => Theme {
            id: ThemeId::Dark,
            background: Color::rgb(0x0D, 0x11, 0x17),
            surface: Color::rgb(0x16, 0x1D, 0x26),
            border: Color::rgb(0x2A, 0x35, 0x42),
            foreground: Color::rgb(0xD6, 0xDE, 0xE6),
            muted: Color::rgb(0x7C, 0x8B, 0x9A),
            brand: Color::rgb(0x4F, 0xC3, 0xF7),
            positive: Color::rgb(0x3F, 0xC9, 0x8C),
            negative: Color::rgb(0xEF, 0x5B, 0x6D),
            warning: Color::rgb(0xF2, 0xA9, 0x3C),
            info: Color::rgb(0x64, 0xB5, 0xF6),
        },
        ThemeId::Light => Theme {
            id: ThemeId::Light,
            background: Color::rgb(0xF6, 0xF7, 0xF9),
            surface: Color::rgb(0xFF, 0xFF, 0xFF),
            border: Color::rgb(0xE1, 0xE6, 0xEC),
            foreground: Color::rgb(0x1A, 0x22, 0x2E),
            muted: Color::rgb(0x5A, 0x6A, 0x7A),
            brand: Color::rgb(0x0E, 0x7A, 0xB8),
            positive: Color::rgb(0x16, 0xA3, 0x4A),
            negative: Color::rgb(0xD6, 0x3B, 0x4C),
            warning: Color::rgb(0xC7, 0x7A, 0x0A),
            info: Color::rgb(0x1B, 0x7A, 0xC8),
        },
        ThemeId::Amber => Theme {
            id: ThemeId::Amber,
            background: Color::rgb(0x1C, 0x13, 0x0A),
            surface: Color::rgb(0x2A, 0x1D, 0x10),
            border: Color::rgb(0x46, 0x33, 0x1C),
            foreground: Color::rgb(0xF5, 0xE3, 0xC6),
            muted: Color::rgb(0xA8, 0x8B, 0x66),
            brand: Color::rgb(0xFF, 0xB3, 0x4D),
            positive: Color::rgb(0x7E, 0xD3, 0x6C),
            negative: Color::rgb(0xE8, 0x6A, 0x6A),
            warning: Color::rgb(0xFF, 0xD5, 0x4D),
            info: Color::rgb(0x8A, 0xC6, 0xFF),
        },
    }
}
