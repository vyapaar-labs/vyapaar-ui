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

impl LogLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Debug => "debug",
            Self::Info => "info",
            Self::Warn => "warn",
            Self::Error => "error",
            Self::Silent => "silent",
        }
    }
}

/// Output format for ask and other machine-readable commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Default)]
pub enum OutputFormat {
    #[default]
    Text,
    Json,
}

/// Missed schedule policy for jobs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Default)]
pub enum MissedPolicy {
    #[default]
    Skip,
    CatchUp,
}

/// Approval gating for job runs (`jobs add --approval auto|manual`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Default)]
pub enum ApprovalPolicyArg {
    /// Runs within budget automatically; only budget breaches need approval.
    #[default]
    Auto,
    /// Hold every run behind a human approval gate.
    Manual,
}

/// Arguments for `vyapaar ask`.
#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct AskArgs {
    /// The prompt to send to the agent.
    pub prompt: String,
    /// Maximum timeout in minutes (default 20).
    #[arg(long, default_value = "20")]
    pub timeout_min: u64,
    /// Specialized financial persona (quant, analyst, trader, risk-manager, etc.).
    #[arg(long)]
    pub persona: Option<String>,
    /// Runtime execution mode (paper, backtest).
    #[arg(long, default_value = "paper")]
    pub mode: String,
    /// Configuration profile.
    #[arg(long, default_value = "default")]
    pub profile: String,
    /// Output format (text or json).
    #[arg(long, value_enum, default_value = "text")]
    pub format: OutputFormat,
}

impl AskArgs {
    /// Construct a new AskArgs with default 20 min timeout.
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            timeout_min: 20,
            persona: None,
            mode: "paper".to_string(),
            profile: "default".to_string(),
            format: OutputFormat::Text,
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
// Clap command structs are built once per dispatch; boxing the Add field-set
// would ripple through `to_command`/tests without a real gain.
#[allow(clippy::large_enum_variant)]
pub enum JobsArgs {
    Add {
        goal: String,
        schedule: Option<String>,
        max_turns: usize,
        max_inr: Option<rust_decimal::Decimal>,
        max_usd: Option<rust_decimal::Decimal>,
        max_tokens: Option<usize>,
        max_tool_calls: Option<usize>,
        max_data_calls: Option<usize>,
        missed_policy: MissedPolicy,
        approval: ApprovalPolicyArg,
        strategy: Option<String>,
        symbol: Option<String>,
        capital: Option<rust_decimal::Decimal>,
        params: Option<String>,
        success: Option<String>,
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
#[derive(Debug, Clone, PartialEq, Eq, Default, Args)]
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
    Import { path: Option<PathBuf> },
    Install,
}

/// Marketplace subcommands for plugin management.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MarketplaceArgs {
    Add { source: String },
    List,
    Remove { name: String },
    Update { name: Option<String> },
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
    Marketplace(MarketplaceArgs),
}

/// Arguments for `vyapaar skills`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkillsArgs {
    List,
    Show { name: String },
}

/// Arguments for `vyapaar mcp`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum McpArgs {
    List,
    Connect { name: String, command: String },
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

/// Arguments for `vyapaar strategy run`.
#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct StrategyRunArgs {
    /// Strategy name or manifest id.
    pub name: String,
    /// Target trading symbol.
    #[arg(short, long, default_value = "NSE:NIFTY50-INDEX")]
    pub symbol: String,
    /// Parameter overrides as JSON (e.g. '{"fast_period":12}').
    #[arg(long)]
    pub params: Option<String>,
    /// Capital allocated for backtest (INR).
    #[arg(long, default_value = "1000000")]
    pub capital: rust_decimal::Decimal,
    /// Output format (text or json).
    #[arg(long, value_enum, default_value = "text")]
    pub format: OutputFormat,
}

/// Arguments for `vyapaar strategy tune`.
#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct StrategyTuneArgs {
    /// Strategy name or manifest id to tune.
    pub name: String,
    /// Target trading symbol.
    #[arg(short, long, default_value = "NSE:NIFTY50-INDEX")]
    pub symbol: String,
    /// Optimization objective metric (sharpe or pnl).
    #[arg(long, default_value = "sharpe")]
    pub metric: String,
    /// Capital allocated for evaluation (INR).
    #[arg(long, default_value = "1000000")]
    pub capital: rust_decimal::Decimal,
    /// Output format (text or json).
    #[arg(long, value_enum, default_value = "text")]
    pub format: OutputFormat,
}

/// Arguments for `vyapaar strategy suggest`.
#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct StrategySuggestArgs {
    /// Strategy name or manifest id.
    pub name: String,
    /// Target trading symbol.
    #[arg(short, long, default_value = "NSE:NIFTY50-INDEX")]
    pub symbol: String,
    /// Account capital (INR).
    #[arg(long, default_value = "1000000")]
    pub capital: rust_decimal::Decimal,
    /// Percentage of account risked on trade (e.g. 1.0 for 1%).
    #[arg(long, default_value = "1.0")]
    pub risk_pct: rust_decimal::Decimal,
    /// Output format (text or json).
    #[arg(long, value_enum, default_value = "text")]
    pub format: OutputFormat,
}

/// Arguments for `vyapaar strategy`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StrategyArgs {
    List,
    Run(StrategyRunArgs),
    Tune(StrategyTuneArgs),
    Suggest(StrategySuggestArgs),
}

/// The CLI subcommands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Launch the Ratatui terminal workstation.
    Tui {
        /// Optional initial message or query.
        message: Option<String>,
    },
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
    /// Manage financial skills and tool sets.
    Skills(SkillsArgs),
    /// Manage Model Context Protocol (MCP) servers and tools.
    Mcp(McpArgs),
    /// Diagnostic environment check and automatic repair.
    Doctor(DoctorArgs),
    /// Purge temporary caches and session traces.
    Purge(PurgeArgs),
    /// Print a system/environment health report.
    Health,
    /// Manage, run, tune, and get trade suggestions for algorithmic strategies.
    Strategy(StrategyArgs),
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
    /// Effective resolved log level.
    pub log_level: LogLevel,
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
    /// Optional initial message for terminal launch.
    pub message: Option<String>,
}

impl Default for CliArgs {
    fn default() -> Self {
        Self {
            command: Command::Tui { message: None },
            model_path: None,
            temperature: None,
            log_level: LogLevel::Info,
            verbose: false,
            quiet: false,
            skip_permissions: false,
            r#continue: false,
            resume: None,
            message: None,
        }
    }
}

/// Parse CLI arguments using the clap derive API (used by the binary).
pub fn parse() -> CliArgs {
    <Cli as clap::Parser>::parse().into()
}

/// Resolve effective log level honoring Quantcept precedence:
/// 1. VYAPAAR_LOG_LEVEL or QUANTCEPT_LOG_LEVEL env var
/// 2. Explicit flag `--log-level`
/// 3. `--verbose` (debug) / `--quiet` (silent)
/// 4. settings.json (if present)
/// 5. default: info
pub fn resolve_log_level(flag_level: Option<LogLevel>, verbose: bool, quiet: bool) -> LogLevel {
    if quiet {
        return LogLevel::Silent;
    }
    if verbose {
        return LogLevel::Debug;
    }
    if let Ok(env_lvl) =
        std::env::var("VYAPAAR_LOG_LEVEL").or_else(|_| std::env::var("QUANTCEPT_LOG_LEVEL"))
    {
        match env_lvl.trim().to_lowercase().as_str() {
            "debug" => return LogLevel::Debug,
            "info" => return LogLevel::Info,
            "warn" | "warning" => return LogLevel::Warn,
            "error" => return LogLevel::Error,
            "silent" | "off" => return LogLevel::Silent,
            _ => {}
        }
    }
    if let Some(lvl) = flag_level {
        return lvl;
    }
    LogLevel::Info
}

/// The clap-defined CLI structure.
#[derive(Debug, Parser, Clone, PartialEq)]
#[command(
    name = "vyapaar",
    version,
    about = "Vyapaar Indian equities quantitative AI workstation & CLI",
    long_about = "vyapaar [message] | vyapaar ask <prompt> | vyapaar jobs | vyapaar verify | vyapaar doctor"
)]
pub struct Cli {
    /// Optional initial prompt or message to launch the workstation with.
    #[arg(value_name = "MESSAGE")]
    pub message: Option<String>,

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

#[allow(clippy::derivable_impls)]
impl Default for Cli {
    fn default() -> Self {
        Self {
            message: None,
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
        /// Specialized financial persona (quant, analyst, trader, etc.).
        #[arg(long)]
        persona: Option<String>,
        /// Runtime execution mode (paper, backtest).
        #[arg(long, default_value = "paper")]
        mode: String,
        /// Configuration profile.
        #[arg(long, default_value = "default")]
        profile: String,
        /// Output format (text or json).
        #[arg(long, value_enum, default_value = "text")]
        format: OutputFormat,
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
        subcommand: Option<JobsSubcommand>,
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
        subcommand: Option<MemorySubcommand>,
    },
    /// Manage plugins, skills, and extensions.
    Plugin {
        #[command(subcommand)]
        subcommand: Option<PluginSubcommand>,
    },
    /// Manage installed financial skills and tool sets.
    Skills {
        #[command(subcommand)]
        subcommand: Option<SkillsSubcommand>,
    },
    /// Manage Model Context Protocol (MCP) servers and tools.
    Mcp {
        #[command(subcommand)]
        subcommand: Option<McpSubcommand>,
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
    /// Manage, run, tune, and get trade suggestions for algorithmic strategies.
    Strategy {
        #[command(subcommand)]
        subcommand: Option<StrategySubcommand>,
    },
}

/// Jobs subcommands.
#[derive(Debug, Subcommand, Clone, PartialEq, Eq)]
#[allow(clippy::large_enum_variant)]
pub enum JobsSubcommand {
    /// Add a new autonomous scheduled research job.
    Add {
        /// Natural language goal or task.
        #[arg(required = true, num_args = 1..)]
        goal: Vec<String>,
        /// Cron schedule expression (e.g. "30 7 * * 1-5") or schedule JSON.
        #[arg(long)]
        schedule: Option<String>,
        /// Max conversation turns per run (default: 20).
        #[arg(long, default_value = "20")]
        max_turns: usize,
        /// Max cost budget in INR (statutory/data/compute).
        #[arg(long)]
        max_inr: Option<rust_decimal::Decimal>,
        /// Max cost budget in USD.
        #[arg(long)]
        max_usd: Option<rust_decimal::Decimal>,
        /// Max token consumption cap.
        #[arg(long)]
        max_tokens: Option<usize>,
        /// Max tool calls permitted.
        #[arg(long)]
        max_tool_calls: Option<usize>,
        /// Max data calls permitted.
        #[arg(long)]
        max_data_calls: Option<usize>,
        /// Policy when a scheduled run is missed.
        #[arg(long, value_enum, default_value = "skip")]
        missed_policy: MissedPolicy,
        /// Approval gating before a run may execute.
        #[arg(long, value_enum, default_value = "auto")]
        approval: ApprovalPolicyArg,
        /// Strategy id the job executes (e.g. "sma_crossover"); jobs without a
        /// strategy are refused by `jobs run` because they have no payload.
        #[arg(long)]
        strategy: Option<String>,
        /// Canonical instrument the strategy targets (e.g. "NSE:RELIANCE-EQ").
        #[arg(long)]
        symbol: Option<String>,
        /// Backtest capital in INR (default 1_000_000).
        #[arg(long)]
        capital: Option<rust_decimal::Decimal>,
        /// Strategy parameter overrides as a JSON object (e.g. '{"fast":10}').
        #[arg(long)]
        params: Option<String>,
        /// Optional success criteria JSON.
        #[arg(long)]
        success: Option<String>,
        /// Enforce read-only advisory mode.
        #[arg(long, default_value = "true", action = clap::ArgAction::Set)]
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
        path: Option<PathBuf>,
    },
    /// Print systemd/cron timer snippet for memory auto-distillation.
    Install,
}

/// Plugin marketplace subcommands.
#[derive(Debug, Subcommand, Clone, PartialEq, Eq)]
pub enum MarketplaceSubcommand {
    /// Add an external marketplace source.
    Add {
        /// Marketplace source URL or path.
        source: String,
    },
    /// List marketplace sources and available packages.
    List,
    /// Remove an external marketplace source.
    Remove {
        /// Source identifier or name.
        name: String,
    },
    /// Update marketplace sources.
    Update {
        /// Optional marketplace name to update.
        name: Option<String>,
    },
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
    /// Manage plugin marketplace sources.
    Marketplace {
        #[command(subcommand)]
        action: MarketplaceSubcommand,
    },
}

/// Skills subcommands.
#[derive(Debug, Subcommand, Clone, PartialEq, Eq)]
pub enum SkillsSubcommand {
    /// List all installed financial skills.
    List,
    /// Display details for a financial skill.
    Show {
        /// Skill name (e.g. quant, analyst, trader).
        name: String,
    },
}

/// MCP subcommands.
#[derive(Debug, Subcommand, Clone, PartialEq, Eq)]
pub enum McpSubcommand {
    /// List connected MCP servers.
    List,
    /// Connect a new MCP server.
    Connect {
        /// Server name.
        name: String,
        /// Command or endpoint to connect.
        command: String,
    },
}

/// Strategy subcommands.
#[derive(Debug, Subcommand, Clone, PartialEq, Eq)]
pub enum StrategySubcommand {
    /// List all discovered built-in and manifest strategies.
    List,
    /// Run a strategy backtest.
    Run {
        /// Strategy name or id.
        name: String,
        /// Target instrument symbol.
        #[arg(short, long, default_value = "NSE:NIFTY50-INDEX")]
        symbol: String,
        /// Parameter overrides as JSON (e.g. '{"fast_period":12}').
        #[arg(long)]
        params: Option<String>,
        /// Allocated capital.
        #[arg(long, default_value = "1000000")]
        capital: rust_decimal::Decimal,
        /// Output format.
        #[arg(long, value_enum, default_value = "text")]
        format: OutputFormat,
    },
    /// Tune strategy parameters using grid search with overfitting guardrails.
    Tune {
        /// Strategy name or id.
        name: String,
        /// Target instrument symbol.
        #[arg(short, long, default_value = "NSE:NIFTY50-INDEX")]
        symbol: String,
        /// Target optimization metric (sharpe or pnl).
        #[arg(long, default_value = "sharpe")]
        metric: String,
        /// Allocated capital.
        #[arg(long, default_value = "1000000")]
        capital: rust_decimal::Decimal,
        /// Output format.
        #[arg(long, value_enum, default_value = "text")]
        format: OutputFormat,
    },
    /// Tune strategy parameters and generate an optimal trade recommendation.
    Suggest {
        /// Strategy name or id.
        name: String,
        /// Target instrument symbol.
        #[arg(short, long, default_value = "NSE:NIFTY50-INDEX")]
        symbol: String,
        /// Account capital.
        #[arg(long, default_value = "1000000")]
        capital: rust_decimal::Decimal,
        /// Percentage of account to risk (e.g. 1.0 for 1%).
        #[arg(long, default_value = "1.0")]
        risk_pct: rust_decimal::Decimal,
        /// Output format.
        #[arg(long, value_enum, default_value = "text")]
        format: OutputFormat,
    },
}

impl From<Cli> for CliArgs {
    fn from(cli: Cli) -> Self {
        let log_level = resolve_log_level(cli.log_level, cli.verbose, cli.quiet);
        let message = cli.message.clone();

        let command = match cli.command {
            None | Some(CliCommand::Tui) => Command::Tui {
                message: message.clone(),
            },
            Some(CliCommand::Ask {
                prompt,
                timeout_min,
                persona,
                mode,
                profile,
                format,
            }) => Command::Ask(AskArgs {
                prompt,
                timeout_min,
                persona,
                mode,
                profile,
                format,
            }),
            Some(CliCommand::Chat { system_prompt }) => Command::Chat(ChatArgs { system_prompt }),
            Some(CliCommand::Scan { query }) => Command::Scan(ScanArgs { query }),
            Some(CliCommand::Jobs { subcommand }) => Command::Jobs(match subcommand {
                None | Some(JobsSubcommand::List) => JobsArgs::List,
                Some(JobsSubcommand::Add {
                    goal,
                    schedule,
                    max_turns,
                    max_inr,
                    max_usd,
                    max_tokens,
                    max_tool_calls,
                    max_data_calls,
                    missed_policy,
                    approval,
                    strategy,
                    symbol,
                    capital,
                    params,
                    success,
                    read_only,
                    once,
                }) => JobsArgs::Add {
                    goal: goal.join(" "),
                    schedule,
                    max_turns,
                    max_inr,
                    max_usd,
                    max_tokens,
                    max_tool_calls,
                    max_data_calls,
                    missed_policy,
                    approval,
                    strategy,
                    symbol,
                    capital,
                    params,
                    success,
                    read_only,
                    once,
                },
                Some(JobsSubcommand::Run { id }) => JobsArgs::Run { id },
                Some(JobsSubcommand::Tick { all }) => JobsArgs::Tick { all },
                Some(JobsSubcommand::Install) => JobsArgs::Install,
                Some(JobsSubcommand::Logs { id }) => JobsArgs::Logs { id },
                Some(JobsSubcommand::Pause { id }) => JobsArgs::Pause { id },
                Some(JobsSubcommand::Resume { id }) => JobsArgs::Resume { id },
                Some(JobsSubcommand::Approvals) => JobsArgs::Approvals,
                Some(JobsSubcommand::Approve { id }) => JobsArgs::Approve { id },
                Some(JobsSubcommand::Deny { id }) => JobsArgs::Deny { id },
            }),
            Some(CliCommand::Verify { file, strict }) => {
                Command::Verify(VerifyArgs { file, strict })
            }
            Some(CliCommand::Memory { subcommand }) => Command::Memory(match subcommand {
                None | Some(MemorySubcommand::Stats) => MemoryArgs::Stats,
                Some(MemorySubcommand::Tick) => MemoryArgs::Tick,
                Some(MemorySubcommand::Model { name }) => MemoryArgs::Model { name },
                Some(MemorySubcommand::Import { path }) => MemoryArgs::Import { path },
                Some(MemorySubcommand::Install) => MemoryArgs::Install,
            }),
            Some(CliCommand::Plugin { subcommand }) => Command::Plugin(match subcommand {
                None | Some(PluginSubcommand::List) => PluginArgs::List,
                Some(PluginSubcommand::Install { name }) => PluginArgs::Install { name },
                Some(PluginSubcommand::Uninstall { name }) => PluginArgs::Uninstall { name },
                Some(PluginSubcommand::Enable { name }) => PluginArgs::Enable { name },
                Some(PluginSubcommand::Disable { name }) => PluginArgs::Disable { name },
                Some(PluginSubcommand::Validate { path }) => PluginArgs::Validate { path },
                Some(PluginSubcommand::Marketplace { action }) => {
                    PluginArgs::Marketplace(match action {
                        MarketplaceSubcommand::Add { source } => MarketplaceArgs::Add { source },
                        MarketplaceSubcommand::List => MarketplaceArgs::List,
                        MarketplaceSubcommand::Remove { name } => MarketplaceArgs::Remove { name },
                        MarketplaceSubcommand::Update { name } => MarketplaceArgs::Update { name },
                    })
                }
            }),
            Some(CliCommand::Skills { subcommand }) => Command::Skills(match subcommand {
                None | Some(SkillsSubcommand::List) => SkillsArgs::List,
                Some(SkillsSubcommand::Show { name }) => SkillsArgs::Show { name },
            }),
            Some(CliCommand::Mcp { subcommand }) => Command::Mcp(match subcommand {
                None | Some(McpSubcommand::List) => McpArgs::List,
                Some(McpSubcommand::Connect { name, command }) => {
                    McpArgs::Connect { name, command }
                }
            }),
            Some(CliCommand::Doctor { fix, json }) => Command::Doctor(DoctorArgs { fix, json }),
            Some(CliCommand::Purge { dry_run, yes }) => Command::Purge(PurgeArgs { dry_run, yes }),
            Some(CliCommand::Health) => Command::Health,
            Some(CliCommand::Strategy { subcommand }) => Command::Strategy(match subcommand {
                None | Some(StrategySubcommand::List) => StrategyArgs::List,
                Some(StrategySubcommand::Run {
                    name,
                    symbol,
                    params,
                    capital,
                    format,
                }) => StrategyArgs::Run(StrategyRunArgs {
                    name,
                    symbol,
                    params,
                    capital,
                    format,
                }),
                Some(StrategySubcommand::Tune {
                    name,
                    symbol,
                    metric,
                    capital,
                    format,
                }) => StrategyArgs::Tune(StrategyTuneArgs {
                    name,
                    symbol,
                    metric,
                    capital,
                    format,
                }),
                Some(StrategySubcommand::Suggest {
                    name,
                    symbol,
                    capital,
                    risk_pct,
                    format,
                }) => StrategyArgs::Suggest(StrategySuggestArgs {
                    name,
                    symbol,
                    capital,
                    risk_pct,
                    format,
                }),
            }),
        };

        Self {
            command,
            model_path: cli.model_path,
            temperature: cli.temperature,
            log_level,
            verbose: cli.verbose,
            quiet: cli.quiet,
            skip_permissions: cli.skip_permissions,
            r#continue: cli.r#continue,
            resume: cli.resume,
            message,
        }
    }
}
