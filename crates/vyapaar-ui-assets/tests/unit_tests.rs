use std::fs;
use std::path::PathBuf;
use vyapaar_ui_assets::fonts::{FONTS_CSS, FontFamily};

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
