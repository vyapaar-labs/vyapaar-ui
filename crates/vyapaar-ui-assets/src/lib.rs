//! # vyapaar-ui-assets
//!
//! Centralized static assets, self-hosted web fonts (WOFF & WOFF2), themes,
//! and typography tokens for Vyapaar UI.

pub mod fonts;

pub use fonts::{
    FAMILY_IBM_PLEX_MONO, FAMILY_IBM_PLEX_SANS, FAMILY_INTER, FAMILY_NOTO_SANS, FAMILY_ROBOTO_MONO,
    FAMILY_SILKSCREEN, FAMILY_SOURCE_SANS_3, FONTS_CSS, FontFamily,
};
