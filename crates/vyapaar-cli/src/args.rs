//! CLI argument parsing (clap derive).

use clap::{Parser, Subcommand};

/// Arguments for `vyapaar ask`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AskArgs {
    /// The prompt to send to the agent.
    pub prompt: String,
}

/// Arguments for `vyapaar scan`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanArgs {
    /// The screener query (e.g. `"roe>15 and pe between 10 30"`).
    pub query: String,
}

/// The CLI subcommands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Launch the Ratatui terminal workstation.
    Tui,
    /// Ask the agent one headless question.
    Ask(AskArgs),
    /// Run a screener scan.
    Scan(ScanArgs),
    /// Print a system/environment health report.
    Health,
}

/// The `vyapaar` CLI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliArgs {
    /// The command to run.
    pub command: Command,
}

/// Parse CLI arguments using the clap derive API (used by the binary).
pub fn parse() -> CliArgs {
    <Cli as clap::Parser>::parse().into()
}

/// The clap-defined CLI structure.
#[derive(Debug, Parser)]
#[command(
    name = "vyapaar",
    version,
    about = "Vyapaar Indian equities algo workstation CLI",
    long_about = "vyapaar tui | vyapaar ask <prompt> | vyapaar scan <query> | vyapaar health"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: CliCommand,
}

/// The clap-defined subcommand enum.
#[derive(Debug, Subcommand)]
pub enum CliCommand {
    /// Launch the terminal workstation.
    Tui,
    /// Ask the agent one headless question.
    Ask {
        /// The prompt to send to the agent.
        prompt: String,
    },
    /// Run a screener scan.
    Scan {
        /// The screener query.
        query: String,
    },
    /// Print a system/environment health report.
    Health,
}

impl From<Cli> for CliArgs {
    fn from(cli: Cli) -> Self {
        let command = match cli.command {
            CliCommand::Tui => Command::Tui,
            CliCommand::Ask { prompt } => Command::Ask(AskArgs { prompt }),
            CliCommand::Scan { query } => Command::Scan(ScanArgs { query }),
            CliCommand::Health => Command::Health,
        };
        Self { command }
    }
}
