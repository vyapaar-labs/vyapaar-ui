//! # व्यापार・face vyapaar-ui-assets
//!
//! Static presentational assets for the Vyapaar workstation: branding,
//! theme palettes, self-hosted fonts, demo fixtures, and audio profiles.
//!
//! - **Branding**: brand name, wordmark, tagline.
//! - **Themes**: named [`Theme`] palettes (dark / light / terminal).
//! - **Fonts**: self-hosted WOFF/WOFF2 faces ([`FontFamily`]) and
//!   cross-platform font stacks ([`FontStack`]).
//! - **Fixtures**: deterministic demo data for the Strategy Studio and TUI.
//! - **Audio**: closed set of sound profiles (descriptors only — no I/O;
//!   playback is the host renderer's job, so this stays WASM-safe).

pub mod audio;
pub mod branding;
pub mod fixtures;
pub mod fonts;
pub mod themes;

pub use audio::{SoundId, SoundProfile};
pub use branding::{BrandName, Branding};
pub use fonts::{
    FAMILY_IBM_PLEX_MONO, FAMILY_IBM_PLEX_SANS, FAMILY_INTER, FAMILY_NOTO_SANS, FAMILY_ROBOTO_MONO,
    FAMILY_SILKSCREEN, FAMILY_SOURCE_SANS_3, FONTS_CSS, FontFamily, FontStack, FontWeight,
    mono_stack, sans_stack,
};
pub use themes::{Color, Theme, ThemeId};
pub use vyapaar_core::rust_decimal;

/// CSS component stylesheet shipped with the workspace (`assets/style/`).
pub const COMPONENTS_CSS: &str = include_str!("../assets/style/components.css");
