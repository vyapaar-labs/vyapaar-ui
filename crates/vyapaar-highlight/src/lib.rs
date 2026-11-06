//! # व्यापार・face vyapaar-highlight
//!
//! Zero-I/O, WASM-safe hand-rolled syntax highlighter for the Vyapaar
//! workstation. It returns byte-offset [`HighlightSpan`]s with a closed
//! [`HighlightClass`] enum that terminal (ratatui) and web (Leptos) renderers
//! turn into styled runs.
//!
//! Design constraints (migration-guide.md §1, `0001-agent-chat-terminal-from-artha.md`):
//! - **Zero-I/O**: no filesystem, no network, no clock. Purely a function of
//!   `(language, source)`.
//! - **WASM-safe**: no tree-sitter, no WASM grammar blobs, no native codegen.
//!   A small hand-rolled char lexer per language.
//! - **No runtime dependencies** beyond `std`.
//! - **Tests live in `tests/`** (parallel test directory rule — zero test
//!   modules inside `src/`).

use std::ops::Range;

/// Programming / markup languages the highlighter understands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    /// Rust source.
    Rust,
    /// Python source.
    Python,
    /// JSON documents.
    Json,
    /// Markdown text.
    Markdown,
    /// POSIX-style shell scripts.
    Shell,
    /// No highlighting at all (returns no spans).
    Plain,
}

impl Language {
    /// Parse a language from a common identifier / extension-ish name.
    ///
    /// Unknown names fall back to [`Language::Plain`].
    pub fn from_name(name: &str) -> Self {
        match name.to_ascii_lowercase().as_str() {
            "rust" | "rs" | "rust-lang" => Self::Rust,
            "python" | "py" => Self::Python,
            "json" => Self::Json,
            "markdown" | "md" | "mdx" => Self::Markdown,
            "shell" | "sh" | "bash" | "zsh" => Self::Shell,
            _ => Self::Plain,
        }
    }
}

/// The closed set of lexical classes a [`HighlightSpan`] can carry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HighlightClass {
    /// Reserved keyword (`fn`, `struct`, `def`, `class`, …).
    Keyword,
    /// String / character literal.
    String,
    /// Line or block comment.
    Comment,
    /// Numeric literal.
    Number,
    /// Function / method / heading call name.
    Function,
    /// Type / class name.
    Type,
    /// Punctuation (braces, commas, delimiters).
    Punctuation,
    /// Ordinary text (used mostly to mark non-highlighted runs explicitly).
    Plain,
}

/// A highlighted byte range in the source string.
///
/// Ranges are half-open `[start, end)` byte offsets into the original source
/// and are returned sorted by `start`. Gaps between spans are implicitly
/// [`HighlightClass::Plain`] — renderers treat the full source as the
/// background and paint spans on top.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HighlightSpan {
    /// Half-open byte range.
    pub range: Range<usize>,
    /// Lexical class for this run.
    pub class: HighlightClass,
}

impl HighlightSpan {
    /// Create a new span.
    pub fn new(range: Range<usize>, class: HighlightClass) -> Self {
        Self { range, class }
    }

    /// Length in bytes of the span.
    pub fn len(&self) -> usize {
        self.range.end.saturating_sub(self.range.start)
    }

    /// Whether the span is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Highlight `src` in the given `language`, returning sorted spans.
pub fn highlight_code(language: Language, src: &str) -> Vec<HighlightSpan> {
    let mut spans = match language {
        Language::Rust => highlight_keyworded(
            src,
            RUST_KEYWORDS,
            LexSyntax {
                line_comment: Some("//"),
                block_comment: Some(("/*", "*/")),
                hash_comment: false,
                triple_quote: false,
                single_quote_strings: true,
                string_quote: '"',
                shell_vars: false,
                json_punct: false,
            },
        ),
        Language::Python => highlight_keyworded(
            src,
            PYTHON_KEYWORDS,
            LexSyntax {
                line_comment: None,
                block_comment: None,
                hash_comment: true,
                triple_quote: true,
                single_quote_strings: true,
                string_quote: '"',
                shell_vars: false,
                json_punct: false,
            },
        ),
        Language::Json => highlight_keyworded(
            src,
            JSON_KEYWORDS,
            LexSyntax {
                line_comment: None,
                block_comment: None,
                hash_comment: false,
                triple_quote: false,
                single_quote_strings: false,
                string_quote: '"',
                shell_vars: false,
                json_punct: true,
            },
        ),
        Language::Markdown => highlight_markdown(src),
        Language::Shell => highlight_keyworded(
            src,
            SHELL_KEYWORDS,
            LexSyntax {
                line_comment: None,
                block_comment: None,
                hash_comment: true,
                triple_quote: false,
                single_quote_strings: true,
                string_quote: '"',
                shell_vars: true,
                json_punct: false,
            },
        ),
        Language::Plain => Vec::new(),
    };
    spans.sort_by_key(|s| s.range.start);
    spans
}

/// The syntax shape a keyworded lexer uses for a language.
struct LexSyntax {
    /// `//`-style line comments.
    line_comment: Option<&'static str>,
    /// `/* */`-style block comments.
    block_comment: Option<(&'static str, &'static str)>,
    /// `#`-style line comments.
    hash_comment: bool,
    /// Triple-quoted strings (`'''` / `"""`).
    triple_quote: bool,
    /// Single-quoted strings (`'…'`).
    single_quote_strings: bool,
    /// The double-quote string character.
    string_quote: char,
    /// `$NAME` / `${NAME}` shell variables.
    shell_vars: bool,
    /// JSON punctuation `{ } [ ] : ,`.
    json_punct: bool,
}

/// Keywords for the Rust lexer.
const RUST_KEYWORDS: &[&str] = &[
    "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern",
    "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub",
    "ref", "return", "self", "Self", "static", "struct", "super", "trait", "true", "type",
    "unsafe", "use", "where", "while", "yield",
];

/// Keywords for the Python lexer.
const PYTHON_KEYWORDS: &[&str] = &[
    "and", "as", "assert", "async", "await", "break", "class", "continue", "def", "del", "elif",
    "else", "except", "False", "finally", "for", "from", "global", "if", "import", "in", "is",
    "lambda", "None", "nonlocal", "not", "or", "pass", "raise", "return", "True", "try", "while",
    "with", "yield",
];

/// Keywords for the JSON lexer.
const JSON_KEYWORDS: &[&str] = &["true", "false", "null"];

/// Keywords for the shell lexer.
const SHELL_KEYWORDS: &[&str] = &[
    "case", "do", "done", "elif", "else", "esac", "fi", "for", "function", "if", "in", "select",
    "then", "until", "while",
];

/// The shared keyword/string/comment lexer.
fn highlight_keyworded(src: &str, keywords: &[&str], syntax: LexSyntax) -> Vec<HighlightSpan> {
    let bytes = src.as_bytes();
    let mut spans = Vec::new();
    let mut i = 0usize;
    let n = bytes.len();

    // Track keyword lookup as a set. The vocabularies are tiny so a linear
    // scan is both simple and fast enough for interactive rendering.
    let is_keyword = |word: &str| keywords.contains(&word);

    while i < n {
        let c = bytes[i] as char;

        // Line comment: `//` or `#`.
        if let Some(marker) = syntax.line_comment {
            if src[i..].starts_with(marker) {
                let start = i;
                i = line_comment_end(src, i);
                spans.push(HighlightSpan::new(start..i, HighlightClass::Comment));
                continue;
            }
        }
        if syntax.hash_comment && c == '#' {
            let start = i;
            i = line_comment_end(src, i);
            spans.push(HighlightSpan::new(start..i, HighlightClass::Comment));
            continue;
        }

        // Block comment: `/* … */`.
        if let Some((open, close)) = syntax.block_comment {
            if src[i..].starts_with(open) {
                let start = i;
                i = block_comment_end(src, i, close);
                spans.push(HighlightSpan::new(start..i, HighlightClass::Comment));
                continue;
            }
        }

        // Triple-quoted strings for Python.
        if syntax.triple_quote {
            let triple = "\"\"\"";
            if src[i..].starts_with(triple) {
                let start = i;
                i = delimited_string_end(src, i, triple);
                spans.push(HighlightSpan::new(start..i, HighlightClass::String));
                continue;
            }
            let triple = "'''";
            if src[i..].starts_with(triple) {
                let start = i;
                i = delimited_string_end(src, i, triple);
                spans.push(HighlightSpan::new(start..i, HighlightClass::String));
                continue;
            }
        }

        // Double-quoted string.
        if c == syntax.string_quote {
            let start = i;
            i = quoted_string_end(src, i, c);
            spans.push(HighlightSpan::new(start..i, HighlightClass::String));
            continue;
        }

        // Single-quoted string.
        if syntax.single_quote_strings && c == '\'' {
            let start = i;
            i = quoted_string_end(src, i, '\'');
            spans.push(HighlightSpan::new(start..i, HighlightClass::String));
            continue;
        }

        // Shell variable: `$NAME`, `${NAME}`, `$1`.
        if syntax.shell_vars && c == '$' {
            let start = i;
            i = shell_var_end(src, i);
            if i > start {
                spans.push(HighlightSpan::new(start..i, HighlightClass::Function));
                continue;
            }
            continue;
        }

        // Number literal.
        if c.is_ascii_digit() || (c == '.' && bytes.get(i + 1).is_some_and(u8::is_ascii_digit)) {
            let start = i;
            i = number_end(src, i);
            spans.push(HighlightSpan::new(start..i, HighlightClass::Number));
            continue;
        }

        // Identifier / keyword.
        if is_ident_start(c) {
            let start = i;
            while i < n && is_ident_continue(src.as_bytes()[i] as char) {
                i += 1;
            }
            let word = &src[start..i];
            let class = if is_keyword(word) {
                HighlightClass::Keyword
            } else if bytes.get(i).is_some_and(|b| *b == b'(') {
                HighlightClass::Function
            } else if word
                .chars()
                .next()
                .is_some_and(|ch| ch.is_ascii_uppercase())
            {
                HighlightClass::Type
            } else {
                HighlightClass::Plain
            };
            if class != HighlightClass::Plain {
                spans.push(HighlightSpan::new(start..i, class));
            }
            continue;
        }

        // JSON punctuation.
        if syntax.json_punct && matches!(c, '{' | '}' | '[' | ']' | ':' | ',') {
            spans.push(HighlightSpan::new(i..i + 1, HighlightClass::Punctuation));
        }

        i += 1;
    }

    spans
}

/// Highlight Markdown: headings, code fences, inline code, emphasis, links.
fn highlight_markdown(src: &str) -> Vec<HighlightSpan> {
    let bytes = src.as_bytes();
    let n = bytes.len();
    let mut spans = Vec::new();
    let mut i = 0usize;
    let mut in_fence = false;

    while i < n {
        // Code fence: ``` or ~~~ toggles block mode.
        if src[i..].starts_with("```") || src[i..].starts_with("~~~") {
            let start = i;
            i = consume_line(src, i);
            spans.push(HighlightSpan::new(start..i, HighlightClass::Function));
            in_fence = !in_fence;
            continue;
        }

        // Inside a code fence everything is a plain code run.
        if in_fence {
            let start = i;
            i = consume_line(src, i);
            spans.push(HighlightSpan::new(start..i, HighlightClass::String));
            continue;
        }

        let c = bytes[i] as char;

        // ATX heading: `# …` at the start of a line.
        if c == '#' && (i == 0 || bytes[i - 1] == b'\n') {
            let start = i;
            i = consume_line(src, i);
            spans.push(HighlightSpan::new(start..i, HighlightClass::Keyword));
            continue;
        }

        // Inline code: `` `code` ``.
        if c == '`' {
            let start = i;
            let mut j = i + 1;
            while j < n && bytes[j] != b'`' {
                j += 1;
            }
            let end = (j + 1).min(n);
            i = end;
            spans.push(HighlightSpan::new(start..end, HighlightClass::String));
            continue;
        }

        // Bold `**text**` / `__text__`.
        if (c == '*' || c == '_') && i + 1 < n && bytes[i + 1] == c as u8 {
            let start = i;
            let pair = if c == '*' { "**" } else { "__" };
            if let Some(rel) = src[i + 2..].find(pair) {
                let content_end = i + 2 + rel + 2;
                spans.push(HighlightSpan::new(
                    start..content_end,
                    HighlightClass::Keyword,
                ));
                i = content_end;
                continue;
            }
            i += 1;
            continue;
        }

        // Links `[label](target)`.
        if c == '[' {
            if let Some(close) = src[i..].find(']') {
                let label_end = i + close;
                if src[label_end..].starts_with("](") {
                    if let Some(rel) = src[label_end + 2..].find(')') {
                        let end = label_end + 2 + rel + 1;
                        spans.push(HighlightSpan::new(i..end, HighlightClass::Function));
                        i = end;
                        continue;
                    }
                }
            }
        }

        i += 1;
    }

    spans
}

/// Consume up to and including the trailing newline of the current line, so
/// the lexer always makes forward progress even when called at a `\n`.
fn consume_line(src: &str, from: usize) -> usize {
    match src[from..].find('\n') {
        Some(rel) => from + rel + 1,
        None => src.len(),
    }
}

fn line_comment_end(src: &str, from: usize) -> usize {
    match src[from..].find('\n') {
        Some(rel) => from + rel,
        None => src.len(),
    }
}

fn block_comment_end(src: &str, from: usize, close: &str) -> usize {
    match src[from..].find(close) {
        Some(rel) => from + rel + close.len(),
        None => src.len(),
    }
}

/// Consume a double- or single-quoted string, honouring `\` escapes.
fn quoted_string_end(src: &str, from: usize, quote: char) -> usize {
    let bytes = src.as_bytes();
    let n = bytes.len();
    let mut i = from + 1;
    while i < n {
        if bytes[i] == b'\\' {
            i += 2;
            continue;
        }
        if bytes[i] == quote as u8 {
            return i + 1;
        }
        i += 1;
    }
    n
}

/// Consume a delimited (triple-quote) string without escape handling.
fn delimited_string_end(src: &str, from: usize, delim: &str) -> usize {
    match src[from + delim.len()..].find(delim) {
        Some(rel) => from + delim.len() + rel + delim.len(),
        None => src.len(),
    }
}

/// Consume a shell variable token starting at `$`.
fn shell_var_end(src: &str, from: usize) -> usize {
    let bytes = src.as_bytes();
    let n = bytes.len();
    if from + 1 >= n {
        return from + 1;
    }
    // `${NAME}` braced form.
    if bytes[from + 1] == b'{' {
        let close = (from + 2..n).find(|&j| bytes[j] == b'}');
        return match close {
            Some(j) => j + 1,
            None => n,
        };
    }
    // `$NAME` or positional `$1`.
    let mut i = from + 1;
    while i < n && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
        i += 1;
    }
    i
}

/// Consume a numeric literal: digits, optional decimal point, `_`, and an
/// optional alphabetic suffix (`1_000`, `3.14`, `255u8`, `0xFF`).
fn number_end(src: &str, from: usize) -> usize {
    let bytes = src.as_bytes();
    let n = bytes.len();
    let mut i = from;
    let mut seen_dot = false;
    while i < n {
        let b = bytes[i];
        if b.is_ascii_digit() || b == b'_' {
            i += 1;
        } else if b == b'.' && !seen_dot && i + 1 < n && bytes[i + 1].is_ascii_digit() {
            seen_dot = true;
            i += 1;
        } else if b.is_ascii_alphabetic() && i > from {
            // suffix like `u8`, `f64`, `e3` (kept simple — consumes it whole)
            i += 1;
        } else {
            break;
        }
    }
    i
}

fn is_ident_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

fn is_ident_continue(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}
