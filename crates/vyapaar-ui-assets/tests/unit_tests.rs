//! Unit tests for `vyapaar-ui-assets`.

use std::fs;
use std::path::PathBuf;

use vyapaar_ui_assets::audio::{SoundId, profile};
use vyapaar_ui_assets::branding::{Branding, BRAND_NAME, TAGLINE};
use vyapaar_ui_assets::fonts::{FONTS_CSS, FontFamily, FontWeight, mono_stack, sans_stack};
use vyapaar_ui_assets::themes::{Color, ThemeId, all_themes, theme};

// --- branding ---

#[test]
fn default_branding() {
    let b = Branding::default_brand();
    assert_eq!(b.name, BRAND_NAME);
    assert_eq!(b.tagline, TAGLINE);
    assert!(!b.version.is_empty());
}

// --- themes ---

#[test]
fn all_themes_cover_three_ids() {
    let themes = all_themes();
    assert_eq!(themes.len(), 3);
    assert!(themes.iter().any(|t| t.id == ThemeId::Dark));
    assert!(themes.iter().any(|t| t.id == ThemeId::Light));
    assert!(themes.iter().any(|t| t.id == ThemeId::Amber));
}

#[test]
fn theme_colors_are_consistent() {
    let dark = theme(ThemeId::Dark);
    assert_eq!(dark.background, Color::rgb(0x0D, 0x11, 0x17));
    assert!(dark.foreground != dark.muted);
    // hex output is uppercase RRGGBB.
    assert_eq!(dark.brand.to_hex(), "#4FC3F7");
}

#[test]
fn color_css_rgba_normalizes_alpha() {
    let c = Color::rgba(255, 0, 0, 128);
    assert_eq!(c.to_css_rgba(), "rgba(255, 0, 0, 0.50)");
}

// --- fonts: stack helpers ---

#[test]
fn font_stacks_are_cross_platform() {
    let sans = sans_stack();
    let mono = mono_stack();
    assert!(sans.css().contains("\"Inter\""));
    assert!(mono.css().contains("\"JetBrains Mono\""));
    assert!(mono.css() != sans.css());
}

#[test]
fn font_weight_values() {
    assert_eq!(FontWeight::Regular.value(), 400);
    assert_eq!(FontWeight::Bold.value(), 700);
}

// --- fonts: embedded faces (self-hosted WOFF/WOFF2) ---

#[test]
fn test_all_seven_font_families_defined() {
    let families = FontFamily::all();
    assert_eq!(families.len(), 7);
    assert_eq!(FontFamily::RobotoMono.css_name(), "Roboto Mono");
    assert_eq!(FontFamily::IbmPlexSans.css_name(), "IBM Plex Sans");
    assert_eq!(FontFamily::IbmPlexMono.css_name(), "IBM Plex Mono");
    assert_eq!(FontFamily::Silkscreen.css_name(), "Silkscreen");
    assert_eq!(FontFamily::Inter.css_name(), "Inter");
    assert_eq!(FontFamily::SourceSans3.css_name(), "Source Sans 3");
    assert_eq!(FontFamily::NotoSans.css_name(), "Noto Sans");
}

#[test]
fn test_embedded_fonts_css_contains_all_families() {
    assert!(!FONTS_CSS.is_empty());
    assert!(FONTS_CSS.contains("font-family: 'Roboto Mono'"));
    assert!(FONTS_CSS.contains("font-family: 'IBM Plex Sans'"));
    assert!(FONTS_CSS.contains("font-family: 'IBM Plex Mono'"));
    assert!(FONTS_CSS.contains("font-family: 'Silkscreen'"));
    assert!(FONTS_CSS.contains("font-family: 'Inter'"));
    assert!(FONTS_CSS.contains("font-family: 'Source Sans 3'"));
    assert!(FONTS_CSS.contains("font-family: 'Noto Sans'"));
}

#[test]
fn test_all_font_files_exist_and_have_valid_magic_bytes() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let assets_dir = manifest_dir.join("assets/fonts");

    for family in FontFamily::all() {
        let fam_dir = assets_dir.join(family.relative_dir());
        assert!(fam_dir.exists(), "Directory does not exist: {:?}", fam_dir);

        let pkg = family.relative_dir();
        for &weight in family.supported_weights() {
            // Check woff2
            let woff2_path = fam_dir.join(format!("{pkg}-latin-{weight}-normal.woff2"));
            assert!(woff2_path.exists(), "Missing woff2 file: {:?}", woff2_path);
            let bytes2 = fs::read(&woff2_path).expect("read woff2");
            assert!(bytes2.len() > 1000);
            assert_eq!(&bytes2[0..4], b"wOF2", "Invalid woff2 magic header");

            // Check woff
            let woff_path = fam_dir.join(format!("{pkg}-latin-{weight}-normal.woff"));
            assert!(woff_path.exists(), "Missing woff file: {:?}", woff_path);
            let bytes = fs::read(&woff_path).expect("read woff");
            assert!(bytes.len() > 1000);
            assert_eq!(&bytes[0..4], b"wOFF", "Invalid woff magic header");
        }
    }
}

#[test]
fn test_font_family_serde() {
    for family in FontFamily::all() {
        let json = serde_json::to_string(family).expect("serialize");
        let de: FontFamily = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(*family, de);
    }
}

// --- audio ---

#[test]
fn sound_profiles_exist_for_all_ids() {
    for id in [
        SoundId::OrderAck,
        SoundId::Fill,
        SoundId::RiskReject,
        SoundId::KillSwitch,
        SoundId::AgentMessage,
        SoundId::Notify,
    ] {
        let p = profile(id);
        assert_eq!(p.id, id);
        assert!(p.duration_ms > 0);
    }
}

#[test]
fn kill_switch_is_loudest_and_longest() {
    let ks = profile(SoundId::KillSwitch);
    let n = profile(SoundId::Notify);
    assert!(ks.repeats > n.repeats);
    assert!(ks.duration_ms > n.duration_ms);
}
