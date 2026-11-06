//! CLI runner: dispatch parsed commands to their implementations.

use std::fmt;

use crate::args::{CliArgs, Command};

/// Errors surfaced by the CLI runner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CliError {
    /// The command is not wired to an in-process backend yet.
    NotWired(String),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotWired(what) => write!(f, "`{what}` is not wired to a backend yet"),
        }
    }
}

impl std::error::Error for CliError {}

/// What a successful CLI run produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandOutcome {
    /// The TUI was launched (or, in tests, requested to launch).
    LaunchedTui,
    /// An `ask` turn produced an answer string.
    Answer(String),
    /// A `scan` produced a result count.
    ScanResults {
        /// Number of rows returned.
        rows: usize,
    },
    /// A `health` check produced a report string.
    HealthReport(String),
}

/// Run a parsed CLI command.
pub fn run(cli: &CliArgs) -> Result<CommandOutcome, CliError> {
    match &cli.command {
        Command::Tui => run_tui(),
        Command::Ask(args) => run_ask(&args.prompt),
        Command::Scan(args) => run_scan(&args.query),
        Command::Health => run_health(),
    }
}

/// Launch the terminal workstation.
fn run_tui() -> Result<CommandOutcome, CliError> {
    // The TUI event loop owns the terminal for the process lifetime. In
    // tests we only assert that this *would* launch.
    Ok(CommandOutcome::LaunchedTui)
}

/// Run one headless agent turn.
fn run_ask(prompt: &str) -> Result<CommandOutcome, CliError> {
    if prompt.trim().is_empty() {
        return Err(CliError::NotWired("vyapaar ask (empty prompt)".to_string()));
    }
    // TODO(tier-5 follow-up): wire to vyapaar-agent once its UI-facing API
    // lands. Until then fail loudly rather than fake success.
    Err(CliError::NotWired("vyapaar ask".to_string()))
}

/// Run a screener scan.
fn run_scan(query: &str) -> Result<CommandOutcome, CliError> {
    if query.trim().is_empty() {
        return Err(CliError::NotWired("vyapaar scan (empty query)".to_string()));
    }
    // TODO(tier-5 follow-up): wire to vyapaar-market screener once available.
    Err(CliError::NotWired("vyapaar scan".to_string()))
}

/// Produce a health report for the current environment.
fn run_health() -> Result<CommandOutcome, CliError> {
    let report = format!(
        "vyapaar {}\ncore schema v{}\napi {}\nworkstation: tui ready",
        vyapaar_core::API_VERSION,
        vyapaar_core::SCHEMA_VERSION,
        vyapaar_core::API_VERSION,
    );
    Ok(CommandOutcome::HealthReport(report))
}
