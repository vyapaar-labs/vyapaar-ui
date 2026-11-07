//! # व्यापार・face vyapaar-cli
//!
//! The Vyapaar quantitative AI workstation and developer CLI (`vyapaar` binary)
//! implementing the full Quantcept command hierarchy:
//! - `tui`: interactive multi-tab Ratatui workstation.
//! - `ask`: single-turn headless quantitative question answering.
//! - `chat`: interactive multi-turn terminal conversation.
//! - `scan`: vectorized universe screener scanner.
//! - `jobs`: autonomous scheduled research jobs, cron evaluator, and approval gates.
//! - `verify`: static strategy verification for lookahead bias and data leakage.
//! - `memory`: semantic knowledge store, note indexing, and distillation.
//! - `plugin`: skills and plugin extensions.
//! - `doctor`: environment diagnostic health check and auto-fix.
//! - `purge`: cache and temporary artifact purge.

pub mod args;
pub mod runner;

pub use args::{
    AskArgs, ChatArgs, CliArgs, Command, DoctorArgs, JobsArgs, LogLevel, MemoryArgs, MissedPolicy,
    PluginArgs, PurgeArgs, ScanArgs, VerifyArgs,
};
pub use runner::{CliError, CommandOutcome, resolve_provider, run, run_with_provider};
