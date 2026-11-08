//! # व्यापार・face vyapaar-cli
//!
//! The Vyapaar quantitative AI workstation and developer CLI (`vyapaar` binary)
//! implementing the full Quantcept command hierarchy:
//! - `tui`: interactive multi-tab Ratatui workstation (`vyapaar [message]`).
//! - `ask`: single-turn headless quantitative question answering.
//! - `chat`: interactive multi-turn terminal conversation.
//! - `strategy`: algorithmic strategy discovery, backtesting, hyperparameter tuning, and trade suggestions.
//! - `scan`: vectorized universe screener scanner.
//! - `jobs`: autonomous scheduled research jobs, cron evaluator, and approval gates.
//! - `verify`: static strategy verification for lookahead bias and data leakage.
//! - `memory`: semantic knowledge store, note indexing, and distillation.
//! - `plugin`: skills, plugin extensions, and marketplace.
//! - `skills`: financial skill suites and allowed tool sets.
//! - `mcp`: Model Context Protocol server connectors.
//! - `doctor`: environment diagnostic health check and auto-fix.
//! - `purge`: cache and temporary artifact purge.

pub mod args;
pub mod backtest_gen;
pub mod runner;
pub mod strategy_bridge;

pub use args::{
    AskArgs, ChatArgs, CliArgs, Command, DoctorArgs, JobsArgs, LogLevel, MarketplaceArgs, McpArgs,
    MemoryArgs, MissedPolicy, OutputFormat, PluginArgs, PurgeArgs, ScanArgs, SkillsArgs,
    StrategyArgs, StrategyRunArgs, StrategySubcommand, StrategySuggestArgs, StrategyTuneArgs,
    VerifyArgs, resolve_log_level,
};
pub use backtest_gen::{BacktestQuerySpec, handle_backtest_query, parse_backtest_query};
pub use runner::{CliError, CommandOutcome, resolve_provider, run, run_with_provider};
pub use strategy_bridge::StrategyBridge;
