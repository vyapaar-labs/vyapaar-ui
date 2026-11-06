//! Zero-I/O syntax highlighting for terminal and web code blocks.

use serde::{Deserialize, Serialize};

/// Supported code languages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    /// Rust source code.
    Rust,
    /// Python source code.
    Python,
    /// JSON data.
    Json,
    /// Markdown documentation.
    Markdown,
    /// Shell script / bash commands.
    Shell,
    /// TOML configuration.
    Toml,
    /// SQL queries.
    Sql,
    /// Unstyled plain text.
    Plain,
}

impl Language {
    /// Detect language from code fence tag or file extension.
    pub fn from_tag(tag: &str) -> Self {
        match tag.trim().to_ascii_lowercase().as_str() {
            "rs" | "rust" => Self::Rust,
            "py" | "python" => Self::Python,
            "json" => Self::Json,
            "md" | "markdown" => Self::Markdown,
            "sh" | "bash" | "shell" | "zsh" => Self::Shell,
            "toml" => Self::Toml,
            "sql" => Self::Sql,
            _ => Self::Plain,
        }
    }
}

/// Stylistic token classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HighlightClass {
    /// Language keywords (e.g. `fn`, `let`, `if`, `def`, `SELECT`).
    Keyword,
    /// String literals.
    String,
    /// Line or block comments.
    Comment,
    /// Numeric literals (integers, floats, decimals).
    Number,
    /// Function names or calls.
    Function,
    /// Type or struct names.
    Type,
    /// Punctuation or operators.
    Punctuation,
    /// Unformatted plain text.
    Plain,
}

/// Highlighted span with byte offsets into original source.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HighlightSpan {
    /// Byte start offset (inclusive).
    pub start: usize,
    /// Byte end offset (exclusive).
    pub end: usize,
    /// Token classification.
    pub class: HighlightClass,
}

/// Produce syntax highlighting spans for a source snippet in a given language.
pub fn highlight_code(lang: Language, src: &str) -> Vec<HighlightSpan> {
    if src.is_empty() {
        return Vec::new();
    }
    match lang {
        Language::Rust => highlight_rust(src),
        Language::Python => highlight_python(src),
        Language::Json => highlight_json(src),
        Language::Sql => highlight_sql(src),
        Language::Plain | Language::Markdown | Language::Shell | Language::Toml => {
            highlight_generic(src, lang)
        }
    }
}

fn highlight_rust(src: &str) -> Vec<HighlightSpan> {
    const RUST_KEYWORDS: &[&str] = &[
        "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum",
        "extern", "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move",
        "mut", "pub", "ref", "return", "self", "Self", "static", "struct", "super", "trait",
        "true", "type", "unsafe", "use", "where", "while",
    ];

    tokenize_c_like(src, RUST_KEYWORDS)
}

fn highlight_python(src: &str) -> Vec<HighlightSpan> {
    const PY_KEYWORDS: &[&str] = &[
        "and", "as", "assert", "async", "await", "break", "class", "continue", "def", "del",
        "elif", "else", "except", "False", "finally", "for", "from", "global", "if", "import",
        "in", "is", "lambda", "None", "nonlocal", "not", "or", "pass", "raise", "return", "True",
        "try", "while", "with", "yield",
    ];

    let mut spans = Vec::new();
    let bytes = src.as_bytes();
    let mut i = 0;
    let n = bytes.len();

    while i < n {
        let b = bytes[i];
        if b == b'#' {
            let start = i;
            while i < n && bytes[i] != b'\n' {
                i += 1;
            }
            spans.push(HighlightSpan {
                start,
                end: i,
                class: HighlightClass::Comment,
            });
        } else if b == b'"' || b == b'\'' {
            let quote = b;
            let start = i;
            i += 1;
            let mut escaped = false;
            while i < n {
                if escaped {
                    escaped = false;
                } else if bytes[i] == b'\\' {
                    escaped = true;
                } else if bytes[i] == quote {
                    i += 1;
                    break;
                }
                i += 1;
            }
            spans.push(HighlightSpan {
                start,
                end: i,
                class: HighlightClass::String,
            });
        } else if b.is_ascii_digit() {
            let start = i;
            while i < n && (bytes[i].is_ascii_digit() || bytes[i] == b'.' || bytes[i] == b'_') {
                i += 1;
            }
            spans.push(HighlightSpan {
                start,
                end: i,
                class: HighlightClass::Number,
            });
        } else if b.is_ascii_alphabetic() || b == b'_' {
            let start = i;
            while i < n && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                i += 1;
            }
            let word = &src[start..i];
            let class = if PY_KEYWORDS.contains(&word) {
                HighlightClass::Keyword
            } else if word.chars().next().is_some_and(|c| c.is_ascii_uppercase()) {
                HighlightClass::Type
            } else if i < n && bytes[i] == b'(' {
                HighlightClass::Function
            } else {
                HighlightClass::Plain
            };
            spans.push(HighlightSpan {
                start,
                end: i,
                class,
            });
        } else if b.is_ascii_punctuation() {
            spans.push(HighlightSpan {
                start: i,
                end: i + 1,
                class: HighlightClass::Punctuation,
            });
            i += 1;
        } else {
            i += 1;
        }
    }

    spans
}

fn highlight_json(src: &str) -> Vec<HighlightSpan> {
    let mut spans = Vec::new();
    let bytes = src.as_bytes();
    let mut i = 0;
    let n = bytes.len();

    while i < n {
        let b = bytes[i];
        if b == b'"' {
            let start = i;
            i += 1;
            let mut escaped = false;
            while i < n {
                if escaped {
                    escaped = false;
                } else if bytes[i] == b'\\' {
                    escaped = true;
                } else if bytes[i] == b'"' {
                    i += 1;
                    break;
                }
                i += 1;
            }
            // Check if it's a key or a value
            let mut j = i;
            while j < n && bytes[j].is_ascii_whitespace() {
                j += 1;
            }
            let class = if j < n && bytes[j] == b':' {
                HighlightClass::Keyword
            } else {
                HighlightClass::String
            };
            spans.push(HighlightSpan {
                start,
                end: i,
                class,
            });
        } else if b.is_ascii_digit() || b == b'-' {
            let start = i;
            i += 1;
            while i < n
                && (bytes[i].is_ascii_digit()
                    || bytes[i] == b'.'
                    || bytes[i] == b'e'
                    || bytes[i] == b'E'
                    || bytes[i] == b'+'
                    || bytes[i] == b'-')
            {
                i += 1;
            }
            spans.push(HighlightSpan {
                start,
                end: i,
                class: HighlightClass::Number,
            });
        } else if b.is_ascii_alphabetic() {
            let start = i;
            while i < n && bytes[i].is_ascii_alphabetic() {
                i += 1;
            }
            let word = &src[start..i];
            let class = if word == "true" || word == "false" || word == "null" {
                HighlightClass::Keyword
            } else {
                HighlightClass::Plain
            };
            spans.push(HighlightSpan {
                start,
                end: i,
                class,
            });
        } else if b == b'{' || b == b'}' || b == b'[' || b == b']' || b == b':' || b == b',' {
            spans.push(HighlightSpan {
                start: i,
                end: i + 1,
                class: HighlightClass::Punctuation,
            });
            i += 1;
        } else {
            i += 1;
        }
    }

    spans
}

fn highlight_sql(src: &str) -> Vec<HighlightSpan> {
    const SQL_KEYWORDS: &[&str] = &[
        "select", "from", "where", "insert", "into", "values", "update", "set", "delete", "create",
        "table", "drop", "index", "on", "primary", "key", "not", "null", "check", "default", "and",
        "or", "order", "by", "desc", "asc", "limit", "as", "join", "left", "inner", "outer",
        "group", "having", "count", "sum", "avg", "min", "max", "in",
    ];

    let mut spans = Vec::new();
    let bytes = src.as_bytes();
    let mut i = 0;
    let n = bytes.len();

    while i < n {
        let b = bytes[i];
        if b == b'-' && i + 1 < n && bytes[i + 1] == b'-' {
            let start = i;
            while i < n && bytes[i] != b'\n' {
                i += 1;
            }
            spans.push(HighlightSpan {
                start,
                end: i,
                class: HighlightClass::Comment,
            });
        } else if b == b'\'' || b == b'"' {
            let quote = b;
            let start = i;
            i += 1;
            while i < n && bytes[i] != quote {
                i += 1;
            }
            if i < n {
                i += 1;
            }
            spans.push(HighlightSpan {
                start,
                end: i,
                class: HighlightClass::String,
            });
        } else if b.is_ascii_digit() {
            let start = i;
            while i < n && (bytes[i].is_ascii_digit() || bytes[i] == b'.') {
                i += 1;
            }
            spans.push(HighlightSpan {
                start,
                end: i,
                class: HighlightClass::Number,
            });
        } else if b.is_ascii_alphabetic() || b == b'_' {
            let start = i;
            while i < n && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                i += 1;
            }
            let word = &src[start..i];
            let lower = word.to_ascii_lowercase();
            let class = if SQL_KEYWORDS.contains(&lower.as_str()) {
                HighlightClass::Keyword
            } else {
                HighlightClass::Plain
            };
            spans.push(HighlightSpan {
                start,
                end: i,
                class,
            });
        } else if b.is_ascii_punctuation() {
            spans.push(HighlightSpan {
                start: i,
                end: i + 1,
                class: HighlightClass::Punctuation,
            });
            i += 1;
        } else {
            i += 1;
        }
    }

    spans
}

fn highlight_generic(src: &str, _lang: Language) -> Vec<HighlightSpan> {
    let mut spans = Vec::new();
    let bytes = src.as_bytes();
    let mut i = 0;
    let n = bytes.len();

    while i < n {
        let b = bytes[i];
        if b == b'#' {
            let start = i;
            while i < n && bytes[i] != b'\n' {
                i += 1;
            }
            spans.push(HighlightSpan {
                start,
                end: i,
                class: HighlightClass::Comment,
            });
        } else if b == b'"' || b == b'\'' {
            let quote = b;
            let start = i;
            i += 1;
            while i < n && bytes[i] != quote {
                i += 1;
            }
            if i < n {
                i += 1;
            }
            spans.push(HighlightSpan {
                start,
                end: i,
                class: HighlightClass::String,
            });
        } else if b.is_ascii_digit() {
            let start = i;
            while i < n && (bytes[i].is_ascii_digit() || bytes[i] == b'.') {
                i += 1;
            }
            spans.push(HighlightSpan {
                start,
                end: i,
                class: HighlightClass::Number,
            });
        } else {
            i += 1;
        }
    }

    spans
}

fn tokenize_c_like(src: &str, keywords: &[&str]) -> Vec<HighlightSpan> {
    let mut spans = Vec::new();
    let bytes = src.as_bytes();
    let mut i = 0;
    let n = bytes.len();

    while i < n {
        let b = bytes[i];
        if b == b'/' && i + 1 < n && bytes[i + 1] == b'/' {
            let start = i;
            while i < n && bytes[i] != b'\n' {
                i += 1;
            }
            spans.push(HighlightSpan {
                start,
                end: i,
                class: HighlightClass::Comment,
            });
        } else if b == b'"' {
            let start = i;
            i += 1;
            let mut escaped = false;
            while i < n {
                if escaped {
                    escaped = false;
                } else if bytes[i] == b'\\' {
                    escaped = true;
                } else if bytes[i] == b'"' {
                    i += 1;
                    break;
                }
                i += 1;
            }
            spans.push(HighlightSpan {
                start,
                end: i,
                class: HighlightClass::String,
            });
        } else if b.is_ascii_digit() {
            let start = i;
            while i < n && (bytes[i].is_ascii_digit() || bytes[i] == b'.' || bytes[i] == b'_') {
                i += 1;
            }
            spans.push(HighlightSpan {
                start,
                end: i,
                class: HighlightClass::Number,
            });
        } else if b.is_ascii_alphabetic() || b == b'_' {
            let start = i;
            while i < n && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                i += 1;
            }
            let word = &src[start..i];
            let class = if keywords.contains(&word) {
                HighlightClass::Keyword
            } else if word.chars().next().is_some_and(|c| c.is_ascii_uppercase()) {
                HighlightClass::Type
            } else if i < n && bytes[i] == b'(' {
                HighlightClass::Function
            } else {
                HighlightClass::Plain
            };
            spans.push(HighlightSpan {
                start,
                end: i,
                class,
            });
        } else if b.is_ascii_punctuation() {
            spans.push(HighlightSpan {
                start: i,
                end: i + 1,
                class: HighlightClass::Punctuation,
            });
            i += 1;
        } else {
            i += 1;
        }
    }

    spans
}
