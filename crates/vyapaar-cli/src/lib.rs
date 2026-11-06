//! # व्यापार・face vyapaar-cli
//!
//! The Vyapaar developer CLI (`vyapaar` binary) with four subcommands
//! (migration-guide.md §1): `tui`, `ask`, `scan`, `health`.
//!
//! Scope discipline (docs/plans/0001 §1): the surface is exactly the
//! four-command contract the migration guide asks for — no `plugin`/`jobs`/
//! `memory` backlog. `Tui` launches `vyapaar_tui::TuiApp`; `Ask`, `Scan`,
//! and `Health` are thin runners with clear "not yet wired" errors where the
//! backend (agent / market screener) is not available in-process yet.
//!
//! This crate is argument-parsing and dispatch only; it depends on
//! `vyapaar-tui` for the terminal and keeps no rendering logic of its own.

pub mod args;
pub mod runner;

pub use args::{AskArgs, CliArgs, Command, ScanArgs};
pub use runner::{CliError, CommandOutcome, run};
