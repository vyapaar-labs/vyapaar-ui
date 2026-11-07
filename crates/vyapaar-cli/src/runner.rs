//! CLI runner: dispatch parsed commands to their implementations.

use std::fmt;
use std::io::{self, Write};
use std::path::Path;
use std::sync::Arc;

use vyapaar_agent_llm::{
    CandleGgufConfig, CandleGgufProvider, ChatMessage, ChatRequest, ChatStreamEvent, LlmProvider,
    ENV_GGUF_PATH,
};

use crate::args::{AskArgs, ChatArgs, CliArgs, Command};

/// Errors surfaced by the CLI runner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CliError {
    /// The command is not wired to an in-process backend yet.
    NotWired(String),
    /// Execution error.
    Execution(String),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotWired(what) => write!(f, "`{what}` is not wired to a backend yet"),
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
    /// A `health` check produced a report string.
    HealthReport(String),
}

fn find_existing_path(path: &Path) -> std::path::PathBuf {
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
    let default_candidates = [
        "models/SmolLM2-360M-Instruct-Q4_K_M.gguf",
        "models/smollm2-360m-instruct-q4_k_m.gguf",
        "../models/SmolLM2-360M-Instruct-Q4_K_M.gguf",
        "../models/smollm2-360m-instruct-q4_k_m.gguf",
        "../../models/SmolLM2-360M-Instruct-Q4_K_M.gguf",
    ];
    for candidate in default_candidates {
        let p = Path::new(candidate);
        if p.exists() {
            let mut config = CandleGgufConfig::new(p);
            if let Some(t) = temperature {
                config = config.with_temperature(t);
            }
            return Ok(Some(Arc::new(CandleGgufProvider::new(config))));
        }
    }

    Ok(None)
}

/// Run a parsed CLI command.
pub fn run(cli: &CliArgs) -> Result<CommandOutcome, CliError> {
    run_with_provider(cli, None)
}

/// Run a parsed CLI command with an explicitly injected provider.
pub fn run_with_provider(
    cli: &CliArgs,
    custom_provider: Option<Arc<dyn LlmProvider>>,
) -> Result<CommandOutcome, CliError> {
    match &cli.command {
        Command::Tui => run_tui(),
        Command::Ask(args) => run_ask(args, cli, custom_provider),
        Command::Chat(args) => run_chat(args, cli, custom_provider),
        Command::Scan(args) => run_scan(&args.query),
        Command::Health => run_health(),
    }
}

/// Launch the terminal workstation.
fn run_tui() -> Result<CommandOutcome, CliError> {
    Ok(CommandOutcome::LaunchedTui)
}

/// Run one headless agent turn.
fn run_ask(
    args: &AskArgs,
    cli: &CliArgs,
    custom_provider: Option<Arc<dyn LlmProvider>>,
) -> Result<CommandOutcome, CliError> {
    let prompt = args.prompt.trim();
    if prompt.is_empty() {
        return Err(CliError::NotWired("vyapaar ask (empty prompt)".to_string()));
    }

    let provider = match custom_provider {
        Some(p) => p,
        None => match resolve_provider(cli.model_path.as_deref(), cli.temperature)? {
            Some(p) => p,
            None => return Err(CliError::NotWired("vyapaar ask".to_string())),
        },
    };

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| CliError::Execution(e.to_string()))?;

    rt.block_on(async move {
        let req = ChatRequest {
            messages: vec![ChatMessage::user(prompt)],
            system_prompt: Some("You are Vyapaar, an intelligent quantitative finance and algorithmic trading assistant for Indian equities (NSE/BSE).".to_string()),
            tools: Vec::new(),
            temperature: cli.temperature.and_then(rust_decimal::Decimal::from_f64_retain),
            max_tokens: Some(1024),
        };

        let mut stream = provider
            .chat_stream(&req)
            .await
            .map_err(|e| CliError::Execution(e.to_string()))?;

        use futures::StreamExt;
        let mut full_answer = String::new();

        while let Some(event_res) = stream.next().await {
            match event_res.map_err(|e| CliError::Execution(e.to_string()))? {
                ChatStreamEvent::TextDelta { text } => {
                    print!("{text}");
                    let _ = io::stdout().flush();
                    full_answer.push_str(&text);
                }
                ChatStreamEvent::MessageEnd { message, .. } => {
                    if full_answer.is_empty() {
                        full_answer = message.content;
                        println!("{full_answer}");
                    } else {
                        println!();
                    }
                }
                _ => {}
            }
        }

        Ok(CommandOutcome::Answer(full_answer))
    })
}

/// Interactive conversational chat session with the agent.
fn run_chat(
    args: &ChatArgs,
    cli: &CliArgs,
    custom_provider: Option<Arc<dyn LlmProvider>>,
) -> Result<CommandOutcome, CliError> {
    let provider = match custom_provider {
        Some(p) => p,
        None => match resolve_provider(cli.model_path.as_deref(), cli.temperature)? {
            Some(p) => p,
            None => return Err(CliError::NotWired("vyapaar chat".to_string())),
        },
    };

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| CliError::Execution(e.to_string()))?;

    rt.block_on(async move {
        println!("═══════════════════════════════════════════════════════════════════");
        println!(" व्यापार ・ Vyapaar Financial AI Agent (SmolLM2 / Candle GGUF)");
        println!(" Model: {}", provider.model_name());
        println!(" Type your question, or 'exit' / '/exit' to quit.");
        println!("═══════════════════════════════════════════════════════════════════\n");

        let mut conversation: Vec<ChatMessage> = Vec::new();
        let sys_prompt = args.system_prompt.clone().unwrap_or_else(|| {
            "You are Vyapaar, an intelligent quantitative finance and algorithmic trading assistant for Indian equities (NSE/BSE).".to_string()
        });

        loop {
            print!("vyapaar > ");
            let _ = io::stdout().flush();

            let mut input = String::new();
            if io::stdin().read_line(&mut input).is_err() {
                break;
            }

            let prompt = input.trim();
            if prompt.is_empty() {
                continue;
            }
            if prompt.eq_ignore_ascii_case("exit")
                || prompt.eq_ignore_ascii_case("/exit")
                || prompt.eq_ignore_ascii_case("quit")
            {
                println!("Goodbye!");
                break;
            }

            conversation.push(ChatMessage::user(prompt));

            let req = ChatRequest {
                messages: conversation.clone(),
                system_prompt: Some(sys_prompt.clone()),
                tools: Vec::new(),
                temperature: cli.temperature.and_then(rust_decimal::Decimal::from_f64_retain),
                max_tokens: Some(1024),
            };

            print!("assistant: ");
            let _ = io::stdout().flush();

            let mut full_resp = String::new();
            match provider.chat_stream(&req).await {
                Ok(mut stream) => {
                    use futures::StreamExt;
                    while let Some(event_res) = stream.next().await {
                        match event_res {
                            Ok(ChatStreamEvent::TextDelta { text }) => {
                                print!("{text}");
                                let _ = io::stdout().flush();
                                full_resp.push_str(&text);
                            }
                            Ok(ChatStreamEvent::MessageEnd { message, .. }) => {
                                if full_resp.is_empty() {
                                    print!("{}", message.content);
                                    let _ = io::stdout().flush();
                                    full_resp = message.content;
                                }
                            }
                            Err(e) => {
                                eprintln!("\n[Error: {e}]");
                                break;
                            }
                            _ => {}
                        }
                    }
                    println!("\n");
                    conversation.push(ChatMessage::assistant(full_resp));
                }
                Err(e) => {
                    eprintln!("[Error: {e}]\n");
                }
            }
        }

        Ok(CommandOutcome::ChatCompleted)
    })
}

/// Run a screener scan.
fn run_scan(query: &str) -> Result<CommandOutcome, CliError> {
    if query.trim().is_empty() {
        return Err(CliError::NotWired("vyapaar scan (empty query)".to_string()));
    }
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
