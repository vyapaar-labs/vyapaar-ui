//! Unit tests for the zero-I/O syntax highlighter (`vyapaar-highlight`).

use vyapaar_highlight::{highlight_code, HighlightClass, HighlightSpan, Language};

/// Helper: collect `(class, text)` pairs for a language so assertions read
/// naturally.
fn classes_and_text(lang: Language, src: &str) -> Vec<(HighlightClass, &str)> {
    highlight_code(lang, src)
        .into_iter()
        .map(|span| (span.class, &src[span.range]))
        .collect()
}

#[test]
fn rust_keywords_and_idents() {
    let src = "fn main() { let mut x = 42u32; }";
    let pairs = classes_and_text(Language::Rust, src);
    assert!(
        pairs.contains(&(HighlightClass::Keyword, "fn")),
        "expected keyword `fn`, got {pairs:?}"
    );
    assert!(
        pairs.contains(&(HighlightClass::Keyword, "let")),
        "expected keyword `let`, got {pairs:?}"
    );
    assert!(
        pairs.contains(&(HighlightClass::Keyword, "mut")),
        "expected keyword `mut`, got {pairs:?}"
    );
    assert!(
        pairs.contains(&(HighlightClass::Number, "42u32")),
        "expected number `42u32`, got {pairs:?}"
    );
    assert!(
        pairs.contains(&(HighlightClass::Function, "main")),
        "expected function `main`, got {pairs:?}"
    );
}

#[test]
fn rust_comments_and_strings() {
    let src = "// line comment\nlet s = \"hello\"; /* block */";
    let pairs = classes_and_text(Language::Rust, src);
    assert!(
        pairs.contains(&(HighlightClass::Comment, "// line comment")),
        "got {pairs:?}"
    );
    assert!(
        pairs.contains(&(HighlightClass::Comment, "/* block */")),
        "got {pairs:?}"
    );
    assert!(
        pairs.contains(&(HighlightClass::String, "\"hello\"")),
        "got {pairs:?}"
    );
}

#[test]
fn python_strings_comments_keywords() {
    let src = "def f(x):\n    # comment\n    return \"yes\" if x else 'no'";
    let pairs = classes_and_text(Language::Python, src);
    assert!(pairs.contains(&(HighlightClass::Keyword, "def")));
    assert!(pairs.contains(&(HighlightClass::Keyword, "return")));
    assert!(pairs.contains(&(HighlightClass::Keyword, "if")));
    assert!(pairs.contains(&(HighlightClass::Keyword, "else")));
    assert!(pairs.contains(&(HighlightClass::Comment, "# comment")));
    assert!(pairs.contains(&(HighlightClass::String, "\"yes\"")));
    assert!(pairs.contains(&(HighlightClass::String, "'no'")));
    assert!(pairs.contains(&(HighlightClass::Function, "f")));
}

#[test]
fn python_triple_quoted_strings() {
    let src = "doc = \"\"\"hello\nworld\"\"\"\n";
    let pairs = classes_and_text(Language::Python, src);
    assert!(
        pairs.contains(&(HighlightClass::String, "\"\"\"hello\nworld\"\"\"")),
        "got {pairs:?}"
    );
}

#[test]
fn json_strings_numbers_punct() {
    let src = r#"{"symbol": "RELIANCE", "price": 2450.75, "ok": true}"#;
    let pairs = classes_and_text(Language::Json, src);
    assert!(pairs.contains(&(HighlightClass::String, "\"symbol\"")));
    assert!(pairs.contains(&(HighlightClass::String, "\"RELIANCE\"")));
    assert!(pairs.contains(&(HighlightClass::String, "\"price\"")));
    assert!(pairs.contains(&(HighlightClass::Number, "2450.75")));
    assert!(pairs.contains(&(HighlightClass::Keyword, "true")));
    assert!(pairs.contains(&(HighlightClass::Punctuation, "{")));
    assert!(pairs.contains(&(HighlightClass::Punctuation, "}")));
    assert!(pairs.contains(&(HighlightClass::Punctuation, ":")));
}

#[test]
fn markdown_headings_code_and_bold() {
    let src = "# Header\n\nSome **bold** and `code`.\n\n```rust\nfn main() {}\n```\n";
    let pairs = classes_and_text(Language::Markdown, src);
    assert!(pairs.contains(&(HighlightClass::Keyword, "# Header\n")));
    assert!(pairs.contains(&(HighlightClass::Keyword, "**bold**")));
    assert!(pairs.contains(&(HighlightClass::String, "`code`")));
    // The fence line itself is highlighted as a Function-class run.
    assert!(pairs.iter().any(|(c, t)| *c == HighlightClass::Function && t.contains("```")));
}

#[test]
fn shell_comments_vars_strings() {
    let src = "#!/bin/sh\nNAME=\"world\"\necho hi $NAME  # greeting\n";
    let pairs = classes_and_text(Language::Shell, src);
    assert!(pairs.contains(&(HighlightClass::Comment, "# greeting")));
    assert!(pairs.contains(&(HighlightClass::String, "\"world\"")));
    assert!(pairs.contains(&(HighlightClass::Function, "$NAME")));
}

#[test]
fn plain_language_returns_no_spans() {
    assert!(highlight_code(Language::Plain, "fn main() { let x = 1; }").is_empty());
}

#[test]
fn spans_are_sorted_and_within_bounds() {
    let src = "let a = \"str\"; // comment\nlet b = 12;\n";
    let spans = highlight_code(Language::Rust, src);
    let mut prev_end = 0usize;
    for span in &spans {
        assert!(span.range.start >= prev_end, "span out of order: {span:?}");
        assert!(span.range.start < span.range.end, "empty span: {span:?}");
        assert!(span.range.end <= src.len(), "span out of bounds: {span:?}");
        prev_end = span.range.end;
    }
    // At least the string, comment, and two keywords/numbers present.
    assert!(spans.len() >= 4, "too few spans: {spans:?}");
}

#[test]
fn language_from_name_fallbacks() {
    assert_eq!(Language::from_name("rs"), Language::Rust);
    assert_eq!(Language::from_name("PY"), Language::Python);
    assert_eq!(Language::from_name("json"), Language::Json);
    assert_eq!(Language::from_name("md"), Language::Markdown);
    assert_eq!(Language::from_name("sh"), Language::Shell);
    assert_eq!(Language::from_name("weird-unknown"), Language::Plain);
}

#[test]
fn span_helpers() {
    let span = HighlightSpan::new(0..4, HighlightClass::Keyword);
    assert_eq!(span.len(), 4);
    assert!(!span.is_empty());
    assert!(HighlightSpan::new(2..2, HighlightClass::Plain).is_empty());
}

#[test]
fn unclosed_string_goes_to_end() {
    let src = "let s = \"unterminated";
    let pairs = classes_and_text(Language::Rust, src);
    assert!(
        pairs.contains(&(HighlightClass::String, "\"unterminated")),
        "got {pairs:?}"
    );
}

#[test]
fn number_leading_dot() {
    let src = "let x = .5;";
    let pairs = classes_and_text(Language::Rust, src);
    assert!(pairs.contains(&(HighlightClass::Number, ".5")), "got {pairs:?}");
}
