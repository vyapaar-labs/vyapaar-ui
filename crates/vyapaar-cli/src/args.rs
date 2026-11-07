//! CLI argument parsing (clap derive).

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Arguments for `vyapaar ask`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AskArgs {
    /// The prompt to send to the agent.
    pub prompt: String,
}

/// Arguments for `vyapaar chat`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ChatArgs {
    /// Optional system prompt instructions.
    pub system_prompt: Option<String>,
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
    /// Interactive conversational chat with the agent.
    Chat(ChatArgs),
    /// Run a screener scan.
    Scan(ScanArgs),
    /// Print a system/environment health report.
    Health,
}

/// The `vyapaar` CLI.
#[derive(Debug, Clone, PartialEq)]
pub struct CliArgs {
    /// The command to run.
    pub command: Command,
    /// Optional path to a local GGUF model file.
    pub model_path: Option<PathBuf>,
    /// Sampling temperature (0.0 to 1.0).
    pub temperature: Option<f64>,
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
    long_about = "vyapaar tui | vyapaar ask <prompt> | vyapaar chat | vyapaar scan <query> | vyapaar health"
)]
pub struct Cli {
    /// Optional path to a local GGUF model file (SmolLM2, etc.).
    #[arg(short = 'm', long = "model-path", global = true)]
    pub model_path: Option<PathBuf>,

    /// Sampling temperature (0.0 to 1.0).
    #[arg(short = 't', long = "temperature", global = true)]
    pub temperature: Option<f64>,

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
    /// Interactive conversational chat with the agent.
    Chat {
        /// Optional system prompt instructions.
        #[arg(short = 's', long = "system-prompt")]
        system_prompt: Option<String>,
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
            CliCommand::Chat { system_prompt } => Command::Chat(ChatArgs { system_prompt }),
            CliCommand::Scan { query } => Command::Scan(ScanArgs { query }),
            CliCommand::Health => Command::Health,
        };
        Self {
            command,
            model_path: cli.model_path,
            temperature: cli.temperature,
        }
    }
}
