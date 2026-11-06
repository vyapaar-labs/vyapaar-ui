use vyapaar_highlight::{HighlightClass, Language, highlight_code};

#[test]
fn test_highlight_rust_tokens() {
    let code = "fn main() {\n    let x = 42;\n    // comment\n}";
    let spans = highlight_code(Language::Rust, code);
    assert!(!spans.is_empty());
    assert!(spans.iter().any(|s| s.class == HighlightClass::Keyword));
    assert!(spans.iter().any(|s| s.class == HighlightClass::Number));
    assert!(spans.iter().any(|s| s.class == HighlightClass::Comment));
}

#[test]
fn test_highlight_json_tokens() {
    let json = r#"{"name": "test", "count": 10, "active": true}"#;
    let spans = highlight_code(Language::Json, json);
    assert!(!spans.is_empty());
    assert!(spans.iter().any(|s| s.class == HighlightClass::Keyword));
    assert!(spans.iter().any(|s| s.class == HighlightClass::String));
    assert!(spans.iter().any(|s| s.class == HighlightClass::Number));
}

#[test]
fn test_from_tag() {
    assert_eq!(Language::from_tag("rs"), Language::Rust);
    assert_eq!(Language::from_tag("python"), Language::Python);
    assert_eq!(Language::from_tag("JSON"), Language::Json);
    assert_eq!(Language::from_tag("unknown"), Language::Plain);
}
