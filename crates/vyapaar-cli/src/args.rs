//! CLI argument parsing (clap derive) implementing the Quantcept command hierarchy.

use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

/// Log levels for CLI output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
    Silent,
}

/// Missed schedule policy for jobs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Default)]
pub enum MissedPolicy {
    #[default]
    Skip,
    CatchUp,
}

/// Arguments for `vyapaar ask`.
#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct AskArgs {
    /// The prompt to send to the agent.
    pub prompt: String,
    /// Maximum timeout in minutes (default 20).
    #[arg(long, default_value = "20")]
    pub timeout_min: u64,
}

impl AskArgs {
    /// Construct a new AskArgs with default 20 min timeout.
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            timeout_min: 20,
        }
    }
}

/// Arguments for `vyapaar chat`.
#[derive(Debug, Clone, PartialEq, Eq, Default, Args)]
pub struct ChatArgs {
    /// Optional system prompt instructions.
    #[arg(short = 's', long)]
    pub system_prompt: Option<String>,
}

/// Arguments for `vyapaar scan`.
#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct ScanArgs {
    /// The screener query (e.g. `"roe>15 and pe between 10 30"`).
    pub query: String,
}

/// Arguments for `vyapaar jobs`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JobsArgs {
    Add {
        goal: String,
        schedule: Option<String>,
        max_turns: usize,
        max_inr: Option<rust_decimal::Decimal>,
        max_tokens: Option<usize>,
        max_tool_calls: Option<usize>,
        missed_policy: MissedPolicy,
        read_only: bool,
        once: bool,
    },
    List,
    Run {
        id: String,
    },
    Tick {
        all: bool,
    },
    Install,
    Logs {
        id: String,
    },
    Pause {
        id: String,
    },
    Resume {
        id: String,
    },
    Approvals,
    Approve {
        id: String,
    },
    Deny {
        id: String,
    },
}

/// Arguments for `vyapaar verify`.
#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct VerifyArgs {
    /// Path to the strategy file or algorithm script to verify.
    pub file: Option<PathBuf>,
    /// Treat warnings (e.g. fit-before-split) as fatal errors.
    #[arg(long)]
    pub strict: bool,
}

/// Arguments for `vyapaar memory`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemoryArgs {
    Tick,
    Stats,
    Model { name: Option<String> },
    Import { path: PathBuf },
    Install,
}

/// Arguments for `vyapaar plugin`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginArgs {
    List,
    Install { name: String },
    Uninstall { name: String },
    Enable { name: String },
    Disable { name: String },
    Validate { path: Option<PathBuf> },
}

/// Arguments for `vyapaar doctor`.
#[derive(Debug, Clone, PartialEq, Eq, Default, Args)]
pub struct DoctorArgs {
    /// Automatically provision missing directories, tables, and sidecars.
    #[arg(long)]
    pub fix: bool,
    /// Emit health diagnostic report as JSON.
    #[arg(long)]
    pub json: bool,
}

/// Arguments for `vyapaar purge`.
#[derive(Debug, Clone, PartialEq, Eq, Default, Args)]
pub struct PurgeArgs {
    /// Display which files and caches would be removed without deleting.
    #[arg(long)]
    pub dry_run: bool,
    /// Confirm non-interactive deletion.
    #[arg(long)]
    pub yes: bool,
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
    /// Manage autonomous scheduled jobs and approvals.
    Jobs(JobsArgs),
    /// Verify a strategy file for lookahead bias and data leakage.
    Verify(VerifyArgs),
    /// Manage local semantic financial memory and distillation.
    Memory(MemoryArgs),
    /// Manage plugins, skills, and extensions.
    Plugin(PluginArgs),
    /// Diagnostic environment check and automatic repair.
    Doctor(DoctorArgs),
    /// Purge temporary caches and session traces.
    Purge(PurgeArgs),
    /// Print a system/environment health report.
    Health,
}

/// The parsed CLI arguments.
#[derive(Debug, Clone, PartialEq)]
pub struct CliArgs {
    /// The command to run.
    pub command: Command,
    /// Optional path to a local GGUF model file.
    pub model_path: Option<PathBuf>,
    /// Sampling temperature (0.0 to 1.0).
    pub temperature: Option<f64>,
    /// Log level.
    pub log_level: Option<LogLevel>,
    /// Verbose output flag.
    pub verbose: bool,
    /// Quiet output flag.
    pub quiet: bool,
    /// Skip tool permissions (auto-accept).
    pub skip_permissions: bool,
    /// Continue latest session.
    pub r#continue: bool,
    /// Resume session by ID.
    pub resume: Option<String>,
}

impl Default for CliArgs {
    fn default() -> Self {
        Self {
            command: Command::Tui,
            model_path: None,
            temperature: None,
            log_level: None,
            verbose: false,
            quiet: false,
            skip_permissions: false,
            r#continue: false,
            resume: None,
        }
    }
}

/// Parse CLI arguments using the clap derive API (used by the binary).
pub fn parse() -> CliArgs {
    <Cli as clap::Parser>::parse().into()
}

/// The clap-defined CLI structure.
#[derive(Debug, Parser, Clone, PartialEq)]
#[command(
    name = "vyapaar",
    version,
    about = "Vyapaar Indian equities quantitative AI workstation & CLI",
    long_about = "vyapaar [message] | vyapaar ask <prompt> | vyapaar chat | vyapaar jobs | vyapaar verify | vyapaar doctor"
)]
pub struct Cli {
    /// Optional path to a local GGUF model file (SmolLM2, etc.).
    #[arg(short = 'm', long = "model-path", global = true)]
    pub model_path: Option<PathBuf>,

    /// Sampling temperature (0.0 to 1.0).
    #[arg(short = 't', long = "temperature", global = true)]
    pub temperature: Option<f64>,

    /// Log level (debug, info, warn, error, silent).
    #[arg(long = "log-level", global = true)]
    pub log_level: Option<LogLevel>,

    /// Verbose output (shortcut for --log-level debug).
    #[arg(short = 'v', long = "verbose", global = true)]
    pub verbose: bool,

    /// Quiet output (shortcut for --log-level silent).
    #[arg(short = 'q', long = "quiet", global = true)]
    pub quiet: bool,

    /// Auto-accept tool confirmation prompts.
    #[arg(
        long = "skip-permissions",
        aliases = ["dangerously-skip-permissions", "yolo"],
        global = true
    )]
    pub skip_permissions: bool,

    /// Resume the latest session in the current directory.
    #[arg(short = 'c', long = "continue", global = true)]
    pub r#continue: bool,

    /// Resume specific session by ID.
    #[arg(short = 'r', long = "resume", global = true)]
    pub resume: Option<String>,

    #[command(subcommand)]
    pub command: Option<CliCommand>,
}

impl Default for Cli {
    fn default() -> Self {
        Self {
            model_path: None,
            temperature: None,
            log_level: None,
            verbose: false,
            quiet: false,
            skip_permissions: false,
            r#continue: false,
            resume: None,
            command: None,
        }
    }
}

/// The clap-defined subcommand enum.
#[derive(Debug, Subcommand, Clone, PartialEq, Eq)]
pub enum CliCommand {
    /// Launch the terminal workstation.
    Tui,
    /// Ask the agent one headless question.
    Ask {
        /// The prompt to send to the agent.
        prompt: String,
        /// Maximum timeout in minutes (default 20).
        #[arg(long, default_value = "20")]
        timeout_min: u64,
    },
    /// Interactive conversational chat with the agent.
    Chat {
        /// Optional system prompt instructions.
        #[arg(short = 's', long)]
        system_prompt: Option<String>,
    },
    /// Run a screener scan.
    Scan {
        /// The screener query.
        query: String,
    },
    /// Manage autonomous scheduled jobs and approvals.
    Jobs {
        #[command(subcommand)]
        subcommand: JobsSubcommand,
    },
    /// Verify a strategy file for lookahead bias and data leakage.
    Verify {
        /// Path to the strategy file or algorithm script to verify.
        file: Option<PathBuf>,
        /// Treat warnings (e.g. fit-before-split) as fatal errors.
        #[arg(long)]
        strict: bool,
    },
    /// Manage local semantic financial memory and distillation.
    Memory {
        #[command(subcommand)]
        subcommand: MemorySubcommand,
    },
    /// Manage plugins, skills, and extensions.
    Plugin {
        #[command(subcommand)]
        subcommand: PluginSubcommand,
    },
    /// Diagnostic environment check and automatic repair.
    Doctor {
        /// Automatically provision missing directories, tables, and sidecars.
        #[arg(long)]
        fix: bool,
        /// Emit health diagnostic report as JSON.
        #[arg(long)]
        json: bool,
    },
    /// Purge temporary caches and session traces.
    Purge {
        /// Display which files and caches would be removed without deleting.
        #[arg(long)]
        dry_run: bool,
        /// Confirm non-interactive deletion.
        #[arg(long)]
        yes: bool,
    },
    /// Print a system/environment health report.
    Health,
}

/// Jobs subcommands.
#[derive(Debug, Subcommand, Clone, PartialEq, Eq)]
pub enum JobsSubcommand {
    /// Add a new autonomous scheduled research job.
    Add {
        /// Natural language goal or task.
        goal: Vec<String>,
        /// Cron schedule expression (e.g. "30 7 * * 1-5").
        #[arg(long)]
        schedule: Option<String>,
        /// Max conversation turns per run (default: 20).
        #[arg(long, default_value = "20")]
        max_turns: usize,
        /// Max cost budget in INR (statutory/data/compute).
        #[arg(long)]
        max_inr: Option<rust_decimal::Decimal>,
        /// Max token consumption cap.
        #[arg(long)]
        max_tokens: Option<usize>,
        /// Max tool calls permitted.
        #[arg(long)]
        max_tool_calls: Option<usize>,
        /// Policy when a scheduled run is missed.
        #[arg(long, value_enum, default_value = "skip")]
        missed_policy: MissedPolicy,
        /// Enforce read-only advisory mode.
        #[arg(long, default_value = "true")]
        read_only: bool,
        /// Run immediately once and exit.
        #[arg(long)]
        once: bool,
    },
    /// List all scheduled and active jobs.
    List,
    /// Immediately run a specific job by ID.
    Run {
        /// The 8-character job ID.
        id: String,
    },
    /// Evaluate cron schedules against system time and run due jobs.
    Tick {
        /// Tick all workspaces rather than just the current directory.
        #[arg(long)]
        all: bool,
    },
    /// Print scheduler integration snippet (systemd timer or crontab).
    Install,
    /// Display logs and execution traces for a job.
    Logs {
        /// Job ID.
        id: String,
    },
    /// Pause a job.
    Pause {
        /// Job ID.
        id: String,
    },
    /// Resume a paused job.
    Resume {
        /// Job ID.
        id: String,
    },
    /// List pending actions requiring human approval.
    Approvals,
    /// Approve a pending job action.
    Approve {
        /// Action or Job ID.
        id: String,
    },
    /// Deny a pending job action.
    Deny {
        /// Action or Job ID.
        id: String,
    },
}

/// Memory subcommands.
#[derive(Debug, Subcommand, Clone, PartialEq, Eq)]
pub enum MemorySubcommand {
    /// Condense and distill recent conversation transcripts into semantic memory facts.
    Tick,
    /// Display statistics on semantic notes, vectors, and storage.
    Stats,
    /// Set or display the active embedding model name.
    Model {
        /// Embedding model identifier.
        name: Option<String>,
    },
    /// Import research notes, PDFs, or strategy markdown into local memory.
    Import {
        /// Path to directory or file to ingest.
        path: PathBuf,
    },
    /// Print systemd/cron timer snippet for memory auto-distillation.
    Install,
}

/// Plugin subcommands.
#[derive(Debug, Subcommand, Clone, PartialEq, Eq)]
pub enum PluginSubcommand {
    /// List installed plugins and tools.
    List,
    /// Install a plugin by package or source.
    Install {
        /// Plugin name or package specification.
        name: String,
    },
    /// Uninstall a plugin.
    Uninstall {
        /// Plugin name.
        name: String,
    },
    /// Enable an installed plugin.
    Enable {
        /// Plugin name.
        name: String,
    },
    /// Disable an installed plugin.
    Disable {
        /// Plugin name.
        name: String,
    },
    /// Validate a plugin manifest at a given path.
    Validate {
        /// Path to plugin manifest directory.
        path: Option<PathBuf>,
    },
}

impl From<Cli> for CliArgs {
    fn from(cli: Cli) -> Self {
        let command = match cli.command {
            None | Some(CliCommand::Tui) => Command::Tui,
            Some(CliCommand::Ask {
                prompt,
                timeout_min,
            }) => Command::Ask(AskArgs {
                prompt,
                timeout_min,
            }),
            Some(CliCommand::Chat { system_prompt }) => Command::Chat(ChatArgs { system_prompt }),
            Some(CliCommand::Scan { query }) => Command::Scan(ScanArgs { query }),
            Some(CliCommand::Jobs { subcommand }) => Command::Jobs(match subcommand {
                JobsSubcommand::Add {
                    goal,
                    schedule,
                    max_turns,
                    max_inr,
                    max_tokens,
                    max_tool_calls,
                    missed_policy,
                    read_only,
                    once,
                } => JobsArgs::Add {
                    goal: goal.join(" "),
                    schedule,
                    max_turns,
                    max_inr,
                    max_tokens,
                    max_tool_calls,
                    missed_policy,
                    read_only,
                    once,
                },
                JobsSubcommand::List => JobsArgs::List,
                JobsSubcommand::Run { id } => JobsArgs::Run { id },
                JobsSubcommand::Tick { all } => JobsArgs::Tick { all },
                JobsSubcommand::Install => JobsArgs::Install,
                JobsSubcommand::Logs { id } => JobsArgs::Logs { id },
                JobsSubcommand::Pause { id } => JobsArgs::Pause { id },
                JobsSubcommand::Resume { id } => JobsArgs::Resume { id },
                JobsSubcommand::Approvals => JobsArgs::Approvals,
                JobsSubcommand::Approve { id } => JobsArgs::Approve { id },
                JobsSubcommand::Deny { id } => JobsArgs::Deny { id },
            }),
            Some(CliCommand::Verify { file, strict }) => {
                Command::Verify(VerifyArgs { file, strict })
            }
            Some(CliCommand::Memory { subcommand }) => Command::Memory(match subcommand {
                MemorySubcommand::Tick => MemoryArgs::Tick,
                MemorySubcommand::Stats => MemoryArgs::Stats,
                MemorySubcommand::Model { name } => MemoryArgs::Model { name },
                MemorySubcommand::Import { path } => MemoryArgs::Import { path },
                MemorySubcommand::Install => MemoryArgs::Install,
            }),
            Some(CliCommand::Plugin { subcommand }) => Command::Plugin(match subcommand {
                PluginSubcommand::List => PluginArgs::List,
                PluginSubcommand::Install { name } => PluginArgs::Install { name },
                PluginSubcommand::Uninstall { name } => PluginArgs::Uninstall { name },
                PluginSubcommand::Enable { name } => PluginArgs::Enable { name },
                PluginSubcommand::Disable { name } => PluginArgs::Disable { name },
                PluginSubcommand::Validate { path } => PluginArgs::Validate { path },
            }),
            Some(CliCommand::Doctor { fix, json }) => Command::Doctor(DoctorArgs { fix, json }),
            Some(CliCommand::Purge { dry_run, yes }) => Command::Purge(PurgeArgs { dry_run, yes }),
            Some(CliCommand::Health) => Command::Health,
        };

        Self {
            command,
            model_path: cli.model_path,
            temperature: cli.temperature,
            log_level: cli.log_level,
            verbose: cli.verbose,
            quiet: cli.quiet,
            skip_permissions: cli.skip_permissions,
            r#continue: cli.r#continue,
            resume: cli.resume,
        }
    }
}
