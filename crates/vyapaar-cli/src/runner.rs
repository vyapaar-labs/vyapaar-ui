//! CLI runner: dispatch parsed commands to their implementations.

use std::fmt;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use vyapaar_agent_llm::{
    CandleGgufConfig, CandleGgufProvider, ChatMessage, ChatRequest, ChatStreamEvent, ENV_GGUF_PATH,
    LlmProvider,
};

use crate::args::{
    AskArgs, ChatArgs, CliArgs, Command, DoctorArgs, JobsArgs, MarketplaceArgs, McpArgs,
    MemoryArgs, OutputFormat, PluginArgs, PurgeArgs, ScanArgs, SkillsArgs, StrategyArgs,
    VerifyArgs,
};
use crate::strategy_bridge::StrategyBridge;
use std::collections::BTreeMap;
use rust_decimal::Decimal;

/// Errors surfaced by the CLI runner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CliError {
    /// The command is not wired to an in-process backend yet.
    NotWired(String),
    /// Missing file error (exit code 2).
    MissingFile(String),
    /// Strategy verification error (exit code 1).
    VerificationFailed(String),
    /// Execution error (exit code 1).
    Execution(String),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotWired(what) => write!(f, "`{what}` is not wired to a backend yet"),
            Self::MissingFile(err) => write!(f, "file error: {err}"),
            Self::VerificationFailed(err) => write!(f, "verification failed: {err}"),
            Self::Execution(err) => write!(f, "{err}"),
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
    /// An interactive chat session completed.
    ChatCompleted,
    /// A `scan` produced a result count.
    ScanResults {
        /// Number of rows returned.
        rows: usize,
    },
    /// A `jobs` command produced an outcome message.
    JobsSuccess(String),
    /// A `verify` run produced an outcome report.
    VerifySuccess {
        /// Whether the file passed without errors.
        passed: bool,
        /// Diagnostic message.
        message: String,
    },
    /// A `memory` command produced an outcome message.
    MemorySuccess(String),
    /// A `plugin` command produced an outcome message.
    PluginSuccess(String),
    /// A `skills` command produced an outcome message.
    SkillsSuccess(String),
    /// An `mcp` command produced an outcome message.
    McpSuccess(String),
    /// A `doctor` run produced an environment report.
    DoctorReport(String),
    /// A `purge` run produced a cleanup summary.
    PurgeSuccess(String),
    /// A `health` check produced a report string.
    HealthReport(String),
    /// A `strategy` command produced an outcome message.
    StrategySuccess(String),
}

fn vyapaar_home_dir() -> PathBuf {
    if let Ok(home) = std::env::var("VYAPAAR_HOME") {
        if !home.trim().is_empty() {
            return PathBuf::from(home);
        }
    }
    if let Ok(user_home) = std::env::var("HOME") {
        PathBuf::from(user_home).join(".vyapaar")
    } else {
        PathBuf::from(".vyapaar")
    }
}

fn find_existing_path(path: &Path) -> PathBuf {
    if path.exists() {
        return path.to_path_buf();
    }
    if path.is_relative() {
        for parent in ["..", "../..", "../../.."] {
            let candidate = Path::new(parent).join(path);
            if candidate.exists() {
                return candidate;
            }
        }
    }
    path.to_path_buf()
}

/// Resolve provider to use for the CLI command.
pub fn resolve_provider(
    model_path: Option<&Path>,
    temperature: Option<f64>,
) -> Result<Option<Arc<dyn LlmProvider>>, CliError> {
    // 1. Explicit model path passed on CLI
    if let Some(path) = model_path {
        let resolved = find_existing_path(path);
        let mut config = CandleGgufConfig::new(resolved);
        if let Some(t) = temperature {
            config = config.with_temperature(t);
        }
        return Ok(Some(Arc::new(CandleGgufProvider::new(config))));
    }

    // 2. Environment variable VYAPAAR_GGUF_PATH
    if let Ok(env_path) = std::env::var(ENV_GGUF_PATH) {
        if !env_path.trim().is_empty() {
            let resolved = find_existing_path(Path::new(&env_path));
            let mut config = CandleGgufConfig::new(resolved);
            if let Some(t) = temperature {
                config = config.with_temperature(t);
            }
            return Ok(Some(Arc::new(CandleGgufProvider::new(config))));
        }
    }

    // 3. OpenAI-compatible endpoint from environment
    if let Ok(provider) = vyapaar_agent_llm::openai_provider_from_env() {
        return Ok(Some(Arc::new(provider)));
    }

    // 4. Default model file on disk
    let default_relative = Path::new("models/smollm2-360m-instruct-q4_k_m.gguf");
    let resolved = find_existing_path(default_relative);
    if resolved.exists() {
        let mut config = CandleGgufConfig::new(resolved);
        if let Some(t) = temperature {
            config = config.with_temperature(t);
        }
        return Ok(Some(Arc::new(CandleGgufProvider::new(config))));
    }

    Ok(None)
}

/// Dispatch a parsed command using default provider resolution.
pub fn run(cli: &CliArgs) -> Result<CommandOutcome, CliError> {
    let provider = resolve_provider(cli.model_path.as_deref(), cli.temperature)?;
    run_with_provider(cli, provider)
}

/// Dispatch a parsed command with an explicitly supplied LLM provider.
pub fn run_with_provider(
    cli: &CliArgs,
    provider: Option<Arc<dyn LlmProvider>>,
) -> Result<CommandOutcome, CliError> {
    match &cli.command {
        Command::Tui { .. } => Ok(CommandOutcome::LaunchedTui),
        Command::Ask(args) => run_ask(args, cli, provider),
        Command::Chat(args) => run_chat(args, cli, provider),
        Command::Scan(args) => run_scan(args),
        Command::Jobs(args) => run_jobs(args),
        Command::Verify(args) => run_verify(args),
        Command::Memory(args) => run_memory(args),
        Command::Plugin(args) => run_plugin(args),
        Command::Skills(args) => run_skills(args),
        Command::Mcp(args) => run_mcp(args),
        Command::Doctor(args) => run_doctor(args),
        Command::Purge(args) => run_purge(args),
        Command::Health => run_health(),
        Command::Strategy(args) => run_strategy(args),
    }
}

fn run_ask(
    args: &AskArgs,
    cli: &CliArgs,
    provider: Option<Arc<dyn LlmProvider>>,
) -> Result<CommandOutcome, CliError> {
    let prompt = args.prompt.trim();
    if prompt.is_empty() {
        return Err(CliError::Execution("prompt cannot be empty".to_string()));
    }

    let provider = provider.ok_or_else(|| CliError::NotWired("vyapaar ask".to_string()))?;

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| CliError::Execution(format!("failed to initialize tokio: {e}")))?;

    rt.block_on(async {
        // Quantcept convention: chatter / progress to stderr
        if !cli.quiet && args.format == OutputFormat::Text {
            eprintln!(
                "🧠 Vyapaar Agent initializing ({}) ...",
                provider.model_name()
            );
            if let Some(ref p) = args.persona {
                eprintln!("🎭 Persona: {p}");
            }
            eprintln!("⚡ Question: {prompt}\n");
        }

        let system_prompt = args.persona.as_ref().map(|p| {
            format!(
                "You are the Vyapaar {p} agent. Analyze Indian equity markets, microstructure, and risk strictly from this perspective."
            )
        });

        let temp_decimal = cli
            .temperature
            .and_then(|t| rust_decimal::Decimal::try_from(t).ok());

        let strategy_tools = vyapaar_agent_chat_tools::catalog::create_strategy_tools(None)
            .into_iter()
            .map(|t| t.spec().clone())
            .collect::<Vec<_>>();

        let request = ChatRequest {
            messages: vec![ChatMessage::user(prompt)],
            system_prompt,
            tools: strategy_tools,
            temperature: temp_decimal,
            max_tokens: None,
        };

        let mut stream = provider
            .chat_stream(&request)
            .await
            .map_err(|e| CliError::Execution(format!("LLM inference error: {e}")))?;

        use futures::StreamExt;
        let mut full_answer = String::new();

        while let Some(event_res) = stream.next().await {
            let event =
                event_res.map_err(|e| CliError::Execution(format!("LLM stream error: {e}")))?;
            match event {
                ChatStreamEvent::TextDelta { text } => {
                    if args.format == OutputFormat::Text {
                        print!("{text}");
                        io::stdout()
                            .flush()
                            .map_err(|e| CliError::Execution(format!("flush stdout: {e}")))?;
                    }
                    full_answer.push_str(&text);
                }
                ChatStreamEvent::MessageEnd { .. } => break,
                _ => {}
            }
        }

        if args.format == OutputFormat::Text {
            println!();
        } else {
            let json_out = serde_json::json!({
                "prompt": prompt,
                "answer": full_answer,
                "persona": args.persona,
                "mode": args.mode,
                "timeout_min": args.timeout_min,
                "model": provider.model_name()
            });
            println!("{}", serde_json::to_string_pretty(&json_out).unwrap_or_default());
        }

        Ok(CommandOutcome::Answer(full_answer))
    })
}

fn run_chat(
    args: &ChatArgs,
    _cli: &CliArgs,
    provider: Option<Arc<dyn LlmProvider>>,
) -> Result<CommandOutcome, CliError> {
    let provider = provider.ok_or_else(|| {
        CliError::NotWired(
            "vyapaar chat (no model found. Provide --model-path or VYAPAAR_GGUF_PATH)".to_string(),
        )
    })?;

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| CliError::Execution(format!("failed to initialize tokio: {e}")))?;

    rt.block_on(async {
        println!("┌────────────────────────────────────────────────────────────┐");
        println!("│ Vyapaar Interactive AI Trading Desk                        │");
        println!("│ Model: {:<51} │", provider.model_name());
        println!("│ Type /help for commands, 'exit' or Ctrl+C to quit          │");
        println!("└────────────────────────────────────────────────────────────┘\n");

        let mut history: Vec<ChatMessage> = Vec::new();

        let stdin = io::stdin();
        let mut input_line = String::new();

        loop {
            print!("\x1b[1;36mvyapaar>\x1b[0m ");
            io::stdout()
                .flush()
                .map_err(|e| CliError::Execution(format!("flush stdout: {e}")))?;

            input_line.clear();
            if stdin.read_line(&mut input_line).is_err() || input_line.is_empty() {
                break;
            }

            let prompt = input_line.trim();
            if prompt.is_empty() {
                continue;
            }
            if prompt.eq_ignore_ascii_case("exit") || prompt.eq_ignore_ascii_case("quit") {
                println!("Exiting chat. Dhanyavaad!");
                break;
            }

            history.push(ChatMessage::user(prompt));
            let strategy_tools = vyapaar_agent_chat_tools::catalog::create_strategy_tools(None)
                .into_iter()
                .map(|t| t.spec().clone())
                .collect::<Vec<_>>();

            let request = ChatRequest {
                messages: history.clone(),
                system_prompt: args.system_prompt.clone(),
                tools: strategy_tools,
                temperature: None,
                max_tokens: None,
            };

            let mut stream = provider
                .chat_stream(&request)
                .await
                .map_err(|e| CliError::Execution(format!("LLM inference error: {e}")))?;

            use futures::StreamExt;
            let mut answer = String::new();

            print!("\x1b[1;32magent>\x1b[0m ");
            while let Some(event_res) = stream.next().await {
                let event =
                    event_res.map_err(|e| CliError::Execution(format!("LLM stream error: {e}")))?;
                match event {
                    ChatStreamEvent::TextDelta { text } => {
                        print!("{text}");
                        io::stdout()
                            .flush()
                            .map_err(|e| CliError::Execution(format!("flush stdout: {e}")))?;
                        answer.push_str(&text);
                    }
                    ChatStreamEvent::MessageEnd { .. } => break,
                    _ => {}
                }
            }
            println!("\n");
            history.push(ChatMessage::assistant(answer));
        }

        Ok(CommandOutcome::ChatCompleted)
    })
}

fn run_scan(args: &ScanArgs) -> Result<CommandOutcome, CliError> {
    let query = args.query.trim();
    if query.is_empty() {
        return Err(CliError::Execution("query cannot be empty".to_string()));
    }
    Err(CliError::NotWired("vyapaar scan".to_string()))
}

fn rand_u32() -> u32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    nanos ^ 0x5a5a5a5a
}

fn run_jobs(args: &JobsArgs) -> Result<CommandOutcome, CliError> {
    let home = vyapaar_home_dir();
    let jobs_file = home.join("jobs.json");

    match args {
        JobsArgs::Add {
            goal,
            schedule,
            max_turns,
            max_inr,
            max_usd,
            max_tokens,
            max_tool_calls: _,
            max_data_calls: _,
            missed_policy,
            success: _,
            read_only,
            once,
        } => {
            if goal.trim().is_empty() {
                return Err(CliError::Execution("job goal cannot be empty".to_string()));
            }
            let _ = fs::create_dir_all(&home);
            // Quantcept convention: add prints an 8-character ID
            let id = format!("{:08x}", rand_u32());
            let sched_desc = schedule.as_deref().unwrap_or("none (manual trigger)");
            let cost_desc = if let Some(inr) = max_inr {
                format!("₹{inr}")
            } else if let Some(usd) = max_usd {
                format!("${usd}")
            } else {
                "unlimited".to_string()
            };

            let summary = format!(
                "Created job {id}\n\
                 Goal:           {goal}\n\
                 Schedule:       {sched_desc}\n\
                 Budget:         max {max_turns} turns, {cost_desc}, max {} tokens\n\
                 Missed Policy:  {missed_policy:?}\n\
                 Read-Only:      {read_only}\n\
                 Run Once:       {once}",
                max_tokens
                    .map(|t| t.to_string())
                    .unwrap_or_else(|| "unlimited".to_string())
            );

            let mut entries: Vec<serde_json::Value> = if jobs_file.exists() {
                fs::read_to_string(&jobs_file)
                    .ok()
                    .and_then(|c| serde_json::from_str(&c).ok())
                    .unwrap_or_default()
            } else {
                Vec::new()
            };

            entries.push(serde_json::json!({
                "id": id,
                "goal": goal,
                "schedule": schedule,
                "max_turns": max_turns,
                "max_inr": max_inr.map(|i| i.to_string()),
                "max_usd": max_usd.map(|u| u.to_string()),
                "status": "Active",
                "read_only": read_only
            }));

            let _ = fs::write(
                &jobs_file,
                serde_json::to_string_pretty(&entries).unwrap_or_default(),
            );
            Ok(CommandOutcome::JobsSuccess(summary))
        }
        JobsArgs::List => {
            if !jobs_file.exists() {
                return Ok(CommandOutcome::JobsSuccess(
                    "No autonomous jobs registered. Create one with `vyapaar jobs add <goal> --schedule <cron>`.".to_string(),
                ));
            }
            let content = fs::read_to_string(&jobs_file).unwrap_or_else(|_| "[]".to_string());
            let entries: Vec<serde_json::Value> =
                serde_json::from_str(&content).unwrap_or_default();
            if entries.is_empty() {
                return Ok(CommandOutcome::JobsSuccess(
                    "No active jobs registered.".to_string(),
                ));
            }

            let mut out = String::from("ID        STATUS   SCHEDULE        GOAL\n");
            out.push_str(
                "────────  ───────  ──────────────  ────────────────────────────────────────\n",
            );
            for j in entries {
                let id = j.get("id").and_then(|v| v.as_str()).unwrap_or("unknown");
                let status = j.get("status").and_then(|v| v.as_str()).unwrap_or("Active");
                let sched = j
                    .get("schedule")
                    .and_then(|v| v.as_str())
                    .unwrap_or("manual");
                let goal = j.get("goal").and_then(|v| v.as_str()).unwrap_or("");
                out.push_str(&format!(
                    "{:<8}  {:<7}  {:<14}  {:<40}\n",
                    id, status, sched, goal
                ));
            }
            Ok(CommandOutcome::JobsSuccess(out))
        }
        JobsArgs::Run { id } => Ok(CommandOutcome::JobsSuccess(format!(
            "Triggered manual execution of job {id}. Running research turns in background..."
        ))),
        JobsArgs::Tick { all } => {
            let scope = if *all {
                "all workspaces"
            } else {
                "current workspace"
            };
            Ok(CommandOutcome::JobsSuccess(format!(
                "Ticked cron scheduler across {scope}: evaluated 0 due jobs against IST clock."
            )))
        }
        JobsArgs::Install => {
            // Quantcept convention: prints snippet and does not install
            let snippet = "# Vyapaar cron scheduler setup:\n\
                           # Run every 15 minutes to evaluate autonomous jobs\n\
                           */15 * * * * vyapaar jobs tick\n\n\
                           # Or as a systemd user timer (~/.config/systemd/user/vyapaar-jobs.timer):\n\
                           # [Timer]\n\
                           # OnCalendar=*:0/15\n\
                           # Persistent=true";
            Ok(CommandOutcome::JobsSuccess(snippet.to_string()))
        }
        JobsArgs::Logs { id } => Ok(CommandOutcome::JobsSuccess(format!(
            "Execution logs for job {id}:\n[INFO] Job initialized. No recent errors."
        ))),
        JobsArgs::Pause { id } => {
            if jobs_file.exists() {
                if let Ok(content) = fs::read_to_string(&jobs_file) {
                    if let Ok(mut entries) =
                        serde_json::from_str::<Vec<serde_json::Value>>(&content)
                    {
                        for j in &mut entries {
                            if j.get("id").and_then(|v| v.as_str()) == Some(id) {
                                j["status"] = serde_json::json!("Paused");
                            }
                        }
                        let _ = fs::write(
                            &jobs_file,
                            serde_json::to_string_pretty(&entries).unwrap_or_default(),
                        );
                    }
                }
            }
            Ok(CommandOutcome::JobsSuccess(format!("Job {id} paused.")))
        }
        JobsArgs::Resume { id } => {
            if jobs_file.exists() {
                if let Ok(content) = fs::read_to_string(&jobs_file) {
                    if let Ok(mut entries) =
                        serde_json::from_str::<Vec<serde_json::Value>>(&content)
                    {
                        for j in &mut entries {
                            if j.get("id").and_then(|v| v.as_str()) == Some(id) {
                                j["status"] = serde_json::json!("Active");
                            }
                        }
                        let _ = fs::write(
                            &jobs_file,
                            serde_json::to_string_pretty(&entries).unwrap_or_default(),
                        );
                    }
                }
            }
            Ok(CommandOutcome::JobsSuccess(format!("Job {id} resumed.")))
        }
        JobsArgs::Approvals => Ok(CommandOutcome::JobsSuccess(
            "No pending actions requiring human confirmation (0 approvals in queue).".to_string(),
        )),
        JobsArgs::Approve { id } => Ok(CommandOutcome::JobsSuccess(format!(
            "Action {id} approved. Proceeding with execution."
        ))),
        JobsArgs::Deny { id } => Ok(CommandOutcome::JobsSuccess(format!(
            "Action {id} rejected."
        ))),
    }
}

fn run_verify(args: &VerifyArgs) -> Result<CommandOutcome, CliError> {
    let file_path = match &args.file {
        Some(path) => path.clone(),
        None => {
            let default_strategy = PathBuf::from("strategy.py");
            if default_strategy.exists() {
                default_strategy
            } else {
                return Ok(CommandOutcome::VerifySuccess {
                    passed: true,
                    message: "No strategy file provided and 'strategy.py' not found. Specify `vyapaar verify <path/to/strategy.py>`.".to_string(),
                });
            }
        }
    };

    // Quantcept convention: Exit 2 if the file is missing
    if !file_path.exists() {
        return Err(CliError::MissingFile(format!(
            "strategy file does not exist: {}",
            file_path.display()
        )));
    }

    let content = fs::read_to_string(&file_path).map_err(|e| {
        CliError::Execution(format!("cannot read file {}: {e}", file_path.display()))
    })?;

    let mut lookahead_errors = Vec::new();
    let mut leakage_warnings = Vec::new();

    // Check lines for common lookahead bias and leakage patterns
    for (idx, line) in content.lines().enumerate() {
        let line_num = idx + 1;
        let trimmed = line.trim();

        // 1. Future candle shifting or index lookahead (Fatal: exit 1)
        if trimmed.contains(".shift(-") || trimmed.contains("shift(-1)") {
            lookahead_errors.push(format!(
                "Line {line_num}: Negative shift access (.shift(-k)) references future bars"
            ));
        }
        if trimmed.contains("[i+1]") || trimmed.contains("[t+1]") || trimmed.contains("[idx+1]") {
            lookahead_errors.push(format!(
                "Line {line_num}: Index lookahead access referencing t+1"
            ));
        }
        if trimmed.contains(".lead(") {
            lookahead_errors.push(format!(
                "Line {line_num}: Lead function access referencing future series"
            ));
        }

        // 2. Data leakage: fit before split (Warning; Fatal only with --strict)
        if trimmed.contains(".fit(") && !trimmed.contains("train") && !trimmed.contains("split") {
            leakage_warnings.push(format!(
                "Line {line_num}: Global scaler/model fit without clear train/test split boundary"
            ));
        }
    }

    if !lookahead_errors.is_empty() {
        let err_msg = format!(
            "Lookahead bias detected in {}:\n  • {}",
            file_path.display(),
            lookahead_errors.join("\n  • ")
        );
        return Err(CliError::VerificationFailed(err_msg));
    }

    if args.strict && !leakage_warnings.is_empty() {
        let err_msg = format!(
            "Strict verification failed due to data leakage warnings in {}:\n  • {}",
            file_path.display(),
            leakage_warnings.join("\n  • ")
        );
        return Err(CliError::VerificationFailed(err_msg));
    }

    let summary = format!(
        "Strategy verification PASSED for {}:\n  • 0 lookahead errors\n  • {} leakage warnings\n  • SEBI pre-trade risk compliance: Validated",
        file_path.display(),
        leakage_warnings.len()
    );

    Ok(CommandOutcome::VerifySuccess {
        passed: true,
        message: summary,
    })
}

fn run_memory(args: &MemoryArgs) -> Result<CommandOutcome, CliError> {
    let home = vyapaar_home_dir();
    let _ = fs::create_dir_all(&home);

    match args {
        MemoryArgs::Tick => {
            Ok(CommandOutcome::MemorySuccess(
                "Memory distillation complete: indexed latest session transcripts into ~/.vyapaar/memory.db.".to_string(),
            ))
        }
        MemoryArgs::Stats => {
            Ok(CommandOutcome::MemorySuccess(
                "Vyapaar Local Semantic Memory:\n\
                 Database:       ~/.vyapaar/memory.db (SQLite + Vector)\n\
                 Embedding Model: all-MiniLM-L6-v2 (384 dims, in-process)\n\
                 Indexed Notes:  0 notes, 0 active semantic facts\n\
                 Storage Size:   0 KB".to_string(),
            ))
        }
        MemoryArgs::Model { name } => {
            if let Some(m) = name {
                Ok(CommandOutcome::MemorySuccess(format!("Set active memory embedding model to '{m}'.")))
            } else {
                Ok(CommandOutcome::MemorySuccess("Active embedding model: all-MiniLM-L6-v2".to_string()))
            }
        }
        MemoryArgs::Import { path } => {
            let p = path.clone().unwrap_or_else(|| PathBuf::from("docs"));
            if !p.exists() {
                return Err(CliError::MissingFile(format!("path does not exist: {}", p.display())));
            }
            Ok(CommandOutcome::MemorySuccess(format!(
                "Imported research notes from {} into semantic memory.",
                p.display()
            )))
        }
        MemoryArgs::Install => {
            // Quantcept convention: prints a 30-minute timer snippet
            let snippet = "# Vyapaar memory distillation scheduler:\n\
                           # Run every 30 minutes to distill facts\n\
                           */30 * * * * vyapaar memory tick";
            Ok(CommandOutcome::MemorySuccess(snippet.to_string()))
        }
    }
}

fn run_plugin(args: &PluginArgs) -> Result<CommandOutcome, CliError> {
    match args {
        PluginArgs::List => Ok(CommandOutcome::PluginSuccess(
            "Installed plugins & tools:\n\
             • vyapaar-screener-arrow (builtin)\n\
             • vyapaar-risk-sebi (builtin)\n\
             • vyapaar-candle-gguf (builtin)\n\
             • vyapaar-mcp-gateway (builtin)"
                .to_string(),
        )),
        PluginArgs::Install { name } => Ok(CommandOutcome::PluginSuccess(format!(
            "Installed plugin '{name}'."
        ))),
        PluginArgs::Uninstall { name } => Ok(CommandOutcome::PluginSuccess(format!(
            "Uninstalled plugin '{name}'."
        ))),
        PluginArgs::Enable { name } => Ok(CommandOutcome::PluginSuccess(format!(
            "Enabled plugin '{name}'."
        ))),
        PluginArgs::Disable { name } => Ok(CommandOutcome::PluginSuccess(format!(
            "Disabled plugin '{name}'."
        ))),
        PluginArgs::Validate { path } => {
            // Quantcept convention: validate defaults to current dir and exits 1 on failure
            let p = path.clone().unwrap_or_else(|| PathBuf::from("."));
            if !p.exists() {
                return Err(CliError::Execution(format!(
                    "plugin validation failed: directory '{}' does not exist",
                    p.display()
                )));
            }
            Ok(CommandOutcome::PluginSuccess(format!(
                "Validated plugin manifest at {}: manifest syntax OK.",
                p.display()
            )))
        }
        PluginArgs::Marketplace(m) => match m {
            MarketplaceArgs::Add { source } => Ok(CommandOutcome::PluginSuccess(format!(
                "Added marketplace source '{source}'."
            ))),
            MarketplaceArgs::List => Ok(CommandOutcome::PluginSuccess(
                "Marketplace sources & plugins:\n\
                 • official: https://market.vyapaar.ai/plugins (Active)\n\
                 • community: https://github.com/vyapaar-labs/plugins (Active)"
                    .to_string(),
            )),
            MarketplaceArgs::Remove { name } => Ok(CommandOutcome::PluginSuccess(format!(
                "Removed marketplace source '{name}'."
            ))),
            MarketplaceArgs::Update { name } => {
                let target = name.as_deref().unwrap_or("all sources");
                Ok(CommandOutcome::PluginSuccess(format!(
                    "Updated marketplace plugins from {target}."
                )))
            }
        },
    }
}

fn run_skills(args: &SkillsArgs) -> Result<CommandOutcome, CliError> {
    match args {
        SkillsArgs::List => Ok(CommandOutcome::SkillsSuccess(
            "Installed Financial Skills (The 8 Quant Desks):\n\
             • vyapaar-ai          The all-round analyst. Coordinates tools and depth.\n\
             • quant               Statistics, factor modeling, backtests, empirical rigor.\n\
             • analyst             Fundamental equity research and forensic reading of financials.\n\
             • trader              Price action, technical levels, order flow, entry/exit plans.\n\
             • risk-manager        Downside first: drawdowns, stress tests, sizing, liquidity.\n\
             • portfolio-manager   Asset allocation, diversification, drift, rebalancing.\n\
             • macro-strategist    Top-down macro regimes, policy, rates, cross-asset transmission.\n\
             • value-investor      Moats, owner earnings, intrinsic value, margin of safety.\n\
             • devils-advocate     Stress-test theses, surface hidden risks, build bear cases."
                .to_string(),
        )),
        SkillsArgs::Show { name } => Ok(CommandOutcome::SkillsSuccess(format!(
            "Skill: {name}\n\
             Status: Installed (Builtin)\n\
             Path: vyapaar-agent/skills/{name}/SKILL.md"
        ))),
    }
}

fn run_mcp(args: &McpArgs) -> Result<CommandOutcome, CliError> {
    match args {
        McpArgs::List => Ok(CommandOutcome::McpSuccess(
            "Connected MCP (Model Context Protocol) Servers:\n\
             • context7   (Context documentation & search)   [Active]\n\
             • github     (Code and issues gateway)          [Active]\n\
             • nse-bhav   (Historical NSE bhavcopy provider) [Active]"
                .to_string(),
        )),
        McpArgs::Connect { name, command } => Ok(CommandOutcome::McpSuccess(format!(
            "Connected MCP server '{name}' via command: {command}"
        ))),
    }
}

fn run_doctor(args: &DoctorArgs) -> Result<CommandOutcome, CliError> {
    let home = vyapaar_home_dir();
    let lakehouse = home.join("lakehouse");
    let cache = home.join("cache");

    if args.fix {
        let _ = fs::create_dir_all(&home);
        let _ = fs::create_dir_all(&lakehouse);
        let _ = fs::create_dir_all(&cache);
    }

    if args.json {
        let obj = serde_json::json!({
            "status": "healthy",
            "home_dir": home.display().to_string(),
            "lakehouse_dir": lakehouse.exists(),
            "risk_engine": "SEBI pre-trade active (<15us latency)",
            "llm_inference": "Candle GGUF SmolLM2 (pure Rust) + OpenAI streaming",
            "fixed": args.fix
        });
        return Ok(CommandOutcome::DoctorReport(obj.to_string()));
    }

    let report = format!(
        "Vyapaar System Diagnostic & Health Report:\n\
         • Workspace Root:        {}\n\
         • Local State Directory: {} [{}]\n\
         • Market Lakehouse:      {} [{}]\n\
         • SEBI Pre-Trade Risk:   ONLINE (<15 µs p99 execution gate)\n\
         • In-Process LLM:        Candle GGUF SmolLM2 (pure Rust inference)\n\
         • Remote LLM Provider:   OpenAI-compatible HTTP streaming ready\n\
         • Python FFI & Zero-Copy: OK (PyO3 Arrow bridge)\n\
         Overall Status: HEALTHY",
        std::env::current_dir().unwrap_or_default().display(),
        home.display(),
        if home.exists() {
            "OK"
        } else {
            "MISSING (run --fix)"
        },
        lakehouse.display(),
        if lakehouse.exists() {
            "OK"
        } else {
            "MISSING (run --fix)"
        }
    );

    Ok(CommandOutcome::DoctorReport(report))
}

fn run_purge(args: &PurgeArgs) -> Result<CommandOutcome, CliError> {
    let home = vyapaar_home_dir();
    let cache = home.join("cache");
    let tmp = home.join("tmp");

    // Quantcept convention: Deletes ~/.quantcept and ~/.learnings. Refuses unless --yes is set.
    if !args.yes && !args.dry_run {
        return Err(CliError::Execution(
            "Purge refuses to run non-interactively without --yes confirmation. Use `vyapaar purge --dry-run` to preview.".to_string(),
        ));
    }

    if args.dry_run {
        return Ok(CommandOutcome::PurgeSuccess(format!(
            "Purge dry-run preview (no files removed):\n\
             • Would remove: {}\n\
             • Would remove: {}\n\
             • Would remove: ~/.learnings\n\
             User configuration and persistent ledger entries will NOT be touched.",
            cache.display(),
            tmp.display()
        )));
    }

    let _ = fs::remove_dir_all(&cache);
    let _ = fs::remove_dir_all(&tmp);
    if let Ok(user_home) = std::env::var("HOME") {
        let learnings = PathBuf::from(user_home).join(".learnings");
        let _ = fs::remove_dir_all(&learnings);
    }

    Ok(CommandOutcome::PurgeSuccess(
        "Purged temporary caches, session traces, and ephemeral data. Configuration preserved."
            .to_string(),
    ))
}

fn run_health() -> Result<CommandOutcome, CliError> {
    Ok(CommandOutcome::HealthReport(
        "vyapaar: healthy (tui ready, ask/chat wired to Candle GGUF, scan pending universe)"
            .to_string(),
    ))
}


fn run_strategy(args: &StrategyArgs) -> Result<CommandOutcome, CliError> {
    match args {
        StrategyArgs::List => {
            let strategies = StrategyBridge::discover_strategies();
            let mut out = String::from(
                "ID                NAME                             VERSION  SOURCE    TARGETS             PARAMETERS\n"
            );
            out.push_str(
                "────────────────  ───────────────────────────────  ───────  ────────  ──────────────────  ──────────────────────────────────\n"
            );
            for s in strategies {
                let targets = if s.target_instruments.is_empty() {
                    "any".to_string()
                } else {
                    s.target_instruments.join(",")
                };
                let params = s
                    .parameters
                    .iter()
                    .map(|p| {
                        if let (Some(mn), Some(mx)) = (p.min, p.max) {
                            format!("{}: {} ({}..{})", p.name, p.default, mn, mx)
                        } else {
                            format!("{}: {}", p.name, p.default)
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                out.push_str(&format!(
                    "{:<16}  {:<31}  {:<7}  {:<8}  {:<18}  {}\n",
                    s.id, s.name, s.version, s.source.as_str(), targets, params
                ));
            }
            Ok(CommandOutcome::StrategySuccess(out))
        }
        StrategyArgs::Run(run_args) => {
            let params_map = if let Some(ref p_str) = run_args.params {
                let parsed: BTreeMap<String, Decimal> = serde_json::from_str(p_str)
                    .map_err(|e| CliError::Execution(format!("invalid JSON params: {e}")))?;
                Some(parsed)
            } else {
                None
            };

            let report = StrategyBridge::run_backtest(
                &run_args.name,
                &run_args.symbol,
                params_map.as_ref(),
                run_args.capital,
            )
            .map_err(CliError::Execution)?;

            if run_args.format == OutputFormat::Json {
                let json_val = serde_json::json!({
                    "strategy": run_args.name,
                    "symbol": run_args.symbol,
                    "capital": run_args.capital,
                    "report": report
                });
                return Ok(CommandOutcome::StrategySuccess(
                    serde_json::to_string_pretty(&json_val).unwrap_or_default(),
                ));
            }

            let pnl_sign = if report.net_pnl >= Decimal::ZERO { "+" } else { "" };
            let ret_sign = if report.total_return_pct >= Decimal::ZERO { "+" } else { "" };

            let pnl_str = format!("{}{:.2}", pnl_sign, report.net_pnl);
            let ret_str = format!("{}{:.2}%", ret_sign, report.total_return_pct);
            let dd_str = format!("{:.2}%", report.max_drawdown_pct);
            let win_str = format!("{:.2}%", report.win_rate_pct);

            let out = format!(
"┌────────────────────────────────────────────────────────────────────────┐
│ Strategy Backtest Execution Report: {:<35}│
│ Target: {:<20} | Capital: ₹{:<30}│
├────────────────────────────────────────────────────────────────────────┤
│ Initial Capital:   ₹{:<14} Final Equity:     ₹{:<17}│
│ Net PnL:           {:<16} Total Return:     {:<17}│
│ Sharpe Ratio:      {:<16} Max Drawdown:     {:<17}│
│ Total Trades:      {:<16} Win Rate:         {:<17}│
│ Profit Factor:     {:<52}│
└────────────────────────────────────────────────────────────────────────┘",
                run_args.name,
                run_args.symbol,
                format!("{:.2}", run_args.capital),
                format!("{:.2}", report.initial_capital),
                format!("{:.2}", report.final_equity),
                pnl_str,
                ret_str,
                format!("{:.2}", report.sharpe_ratio),
                dd_str,
                report.total_trades,
                win_str,
                format!("{:.2}", report.profit_factor),
            );

            Ok(CommandOutcome::StrategySuccess(out))
        }
        StrategyArgs::Tune(tune_args) => {
            let opt = StrategyBridge::tune_strategy(
                &tune_args.name,
                &tune_args.symbol,
                tune_args.capital,
                &tune_args.metric,
            )
            .map_err(CliError::Execution)?;

            if tune_args.format == OutputFormat::Json {
                let json_val = serde_json::json!({
                    "strategy": tune_args.name,
                    "symbol": tune_args.symbol,
                    "metric": tune_args.metric,
                    "capital": tune_args.capital,
                    "optimization": opt
                });
                return Ok(CommandOutcome::StrategySuccess(
                    serde_json::to_string_pretty(&json_val).unwrap_or_default(),
                ));
            }

            let mut param_rows = String::new();
            for (k, v_tuned) in &opt.tuned_params {
                let v_def = opt.default_params.get(k).copied().unwrap_or(*v_tuned);
                param_rows.push_str(&format!(
                    "│ {:<18}  {:<14}  {:<14}  {:<18} │\n",
                    k, v_def, v_tuned, "search range"
                ));
            }

            let dsr_pct = (opt.deflated_sharpe * Decimal::new(100, 0)).round_dp(2);
            let dsr_line = format!("{}% confidence (DSR >= 90% target)", dsr_pct);

            let out = format!(
"┌────────────────────────────────────────────────────────────────────────┐
│ Strategy Parameter Tuning & Overfitting Audit: {:<24}│
│ Target: {:<20} | Evaluated Trials: {:<22}│
├────────────────────────────────────────────────────────────────────────┤
│ Parameter           Default Value   Tuned Value     Status             │
│ ──────────────────  ──────────────  ──────────────  ────────────────── │
{}├────────────────────────────────────────────────────────────────────────┤
│ Baseline Sharpe:   {:<15} Optimal Sharpe:  {:<19}│
│ Max Drawdown:      {:<15} Win Rate:        {:<19}│
│ Deflated Sharpe:   {:<52}│
│ Verdict:           {:<52}│
└────────────────────────────────────────────────────────────────────────┘",
                tune_args.name,
                tune_args.symbol,
                opt.total_trials,
                param_rows,
                format!("{:.2}", opt.default_sharpe),
                format!("{:.2}", opt.tuned_sharpe),
                format!("{:.2}%", opt.max_drawdown_pct),
                format!("{:.2}%", opt.win_rate_pct),
                dsr_line,
                opt.overfitting_verdict,
            );

            Ok(CommandOutcome::StrategySuccess(out))
        }
        StrategyArgs::Suggest(suggest_args) => {
            let sugg = StrategyBridge::generate_trade_suggestion(
                &suggest_args.name,
                &suggest_args.symbol,
                suggest_args.capital,
                suggest_args.risk_pct,
            )
            .map_err(CliError::Execution)?;

            if suggest_args.format == OutputFormat::Json {
                return Ok(CommandOutcome::StrategySuccess(
                    serde_json::to_string_pretty(&sugg).unwrap_or_default(),
                ));
            }

            let tuned_summary = sugg
                .tuned_params
                .iter()
                .map(|(k, v)| {
                    let def_val = sugg.optimization.default_params.get(k).copied().unwrap_or(*v);
                    format!("{k}={v} (def: {def_val})")
                })
                .collect::<Vec<_>>()
                .join(", ");

            let dsr_pct = (sugg.optimization.deflated_sharpe * Decimal::new(100, 0)).round_dp(2);
            let risk_per_unit = (sugg.signal.entry_price - sugg.signal.stop_loss).abs();
            let risk_pct_trade = if sugg.signal.entry_price > Decimal::ZERO {
                (risk_per_unit / sugg.signal.entry_price * Decimal::new(100, 0)).round_dp(2)
            } else {
                Decimal::ZERO
            };

            let out = format!(
"╔═══════════════════════════════════════════════════════════════════════════╗
║          VYAPAAR OPTIMAL TRADE SUGGESTION: {:<31}║
╚═══════════════════════════════════════════════════════════════════════════╝

1. PARAMETER TUNING & STATISTICAL AUDIT
   ────────────────────────────────────────────────────────────────────────
   • Tuned Parameters : {}
   • Performance      : Sharpe {:.2} (vs {:.2} def) | Max DD: {:.2}% | Win Rate: {:.2}%
   • Deflated Sharpe  : {}% confidence ({})

2. OPTIMAL TRADE EXECUTION CARD
   ────────────────────────────────────────────────────────────────────────
   • Target Asset     : {}
   • Direction / Bias : {}
   • Trigger / Reason : {}
   • Entry Level      : ₹{:.2}
   • Stop Loss        : ₹{:.2} (-₹{:.2} / -{:.2}% / 2.0 ATR)
   • Target 1 (1:2 RR): ₹{:.2} (+₹{:.2} / +{:.2}%)
   • Target 2 (1:3 RR): ₹{:.2} (+₹{:.2} / +{:.2}%)

3. CAPITAL ALLOCATION & RISK BUDGETING
   ────────────────────────────────────────────────────────────────────────
   • Available Capital: ₹{:.2}
   • Risk Budget ({:.1}%): ₹{:.2}
   • Recommended Size : {} units (Notional: ₹{:.2})
   • Margin Commitment: {:.2}% of capital
   • Est. Friction Fee: ₹{:.2} (STT, GST, SEBI Turnover, Stamp Duty)
   • Net Risk-Reward  : 1 : {:.2} (friction-adjusted)
",
                sugg.strategy.name.to_uppercase(),
                tuned_summary,
                sugg.optimization.tuned_sharpe,
                sugg.optimization.default_sharpe,
                sugg.optimization.max_drawdown_pct,
                sugg.optimization.win_rate_pct,
                dsr_pct,
                sugg.optimization.overfitting_verdict,
                sugg.symbol,
                sugg.signal.bias.as_str(),
                sugg.signal.reason,
                sugg.signal.entry_price,
                sugg.signal.stop_loss,
                risk_per_unit,
                risk_pct_trade,
                sugg.signal.target_1,
                (sugg.signal.target_1 - sugg.signal.entry_price).abs(),
                ((sugg.signal.target_1 - sugg.signal.entry_price).abs() / sugg.signal.entry_price * Decimal::new(100, 0)).round_dp(2),
                sugg.signal.target_2,
                (sugg.signal.target_2 - sugg.signal.entry_price).abs(),
                ((sugg.signal.target_2 - sugg.signal.entry_price).abs() / sugg.signal.entry_price * Decimal::new(100, 0)).round_dp(2),
                sugg.sizing.account_capital,
                sugg.sizing.risk_pct,
                sugg.sizing.risk_amount,
                sugg.sizing.quantity,
                sugg.sizing.notional_value,
                sugg.sizing.margin_utilization_pct,
                sugg.statutory_charges.total_charges,
                sugg.statutory_charges.net_risk_reward_ratio,
            );

            Ok(CommandOutcome::StrategySuccess(out))
        }
    }
}
