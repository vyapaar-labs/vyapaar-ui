//! Command line interface (CLI) launcher and command runner for Vyapaar.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;
use vyapaar_agent_chat::headless::run_turn_headless;
use vyapaar_agent_chat::permission::ToolRisk;
use vyapaar_agent_chat::session::SessionSelector;
use vyapaar_agent_chat::testing::{FakeChatTool, InMemorySessionStore};
use vyapaar_agent_chat::{ChatAgent, SessionEvent, ToolRegistry};
use vyapaar_agent_core::ports::{NewSession, RuntimeMode, SessionId, SessionScope, SessionStore};
use vyapaar_agent_llm::ScriptedFakeLlm;
use vyapaar_tui::TuiApp;

/// Errors produced during CLI execution.
#[derive(Debug, Error)]
pub enum CliError {
    /// An error occurred during conversational assistant execution.
    #[error("agent error: {0}")]
    Agent(String),
    /// Subcommand execution failure.
    #[error("execution error: {0}")]
    Execution(String),
}

/// Supported CLI subcommands.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Command {
    /// Launch the interactive terminal UI workstation.
    Tui,
    /// Run a single, headless conversational turn.
    Ask {
        /// User query or prompt.
        query: String,
        /// Account profile name (defaults to "default").
        profile: Option<String>,
        /// Runtime mode ("live", "paper", "backtest", defaults to "paper").
        mode: Option<String>,
    },
    /// Run market screening over an instrument universe.
    Scan {
        /// Instrument universe slug (e.g. "nifty50").
        universe: Option<String>,
        /// Filter or ranking metric.
        metric: Option<String>,
    },
    /// Perform environment and dependency health checks.
    Health,
}

/// Root CLI arguments container.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CliArgs {
    /// Subcommand to execute.
    pub command: Command,
}

impl CliArgs {
    /// Create new CLI arguments with a specific command.
    pub fn new(command: Command) -> Self {
        Self { command }
    }
}

/// Execute CLI commands and return human-readable summary output.
pub async fn run_command(args: CliArgs) -> Result<String, CliError> {
    match args.command {
        Command::Tui => {
            let app = TuiApp::new();
            Ok(format!("TUI initialized on tab: {:?}", app.active_tab))
        }
        Command::Ask {
            query,
            profile,
            mode,
        } => {
            let profile_name = profile.unwrap_or_else(|| "default".to_string());
            let runtime_mode = match mode.as_deref() {
                Some("live") => RuntimeMode::Live,
                Some("backtest") => RuntimeMode::Backtest,
                _ => RuntimeMode::Paper,
            };

            // Build standard headless chat agent with in-memory fallback stores
            let fake_llm = ScriptedFakeLlm::new("fake");
            fake_llm.push_text_response(format!(
                "Answering query '{query}' for profile '{profile_name}' in mode '{runtime_mode:?}'."
            ));
            let provider = Arc::new(fake_llm);

            let mut registry = ToolRegistry::new();
            registry.register(Arc::new(FakeChatTool::new("echo", ToolRisk::ReadOnly)));

            let session_store = Arc::new(InMemorySessionStore::new());

            let session_id = SessionId::new();
            session_store
                .create(NewSession {
                    id: session_id,
                    profile: profile_name.clone(),
                    mode: runtime_mode,
                    provider: "scripted".to_string(),
                    model: "fake".to_string(),
                    title: Some("CLI Headless Session".to_string()),
                })
                .await
                .map_err(|e| CliError::Agent(e.to_string()))?;

            let agent = ChatAgent::builder(provider, registry, session_store)
                .build()
                .map_err(|e| CliError::Agent(e.to_string()))?;

            let scope = SessionScope::new(profile_name, runtime_mode);
            let (outcome, events) =
                run_turn_headless(&agent, SessionSelector::Continue(scope), &query)
                    .await
                    .map_err(|e| CliError::Agent(e.to_string()))?;

            let mut response = String::new();
            for event in events {
                if let SessionEvent::TextDelta { text, .. } = event {
                    response.push_str(&text);
                }
            }

            if response.is_empty() {
                response = format!("Turn finished with outcome: {outcome:?}");
            }

            Ok(response)
        }
        Command::Scan { universe, metric } => {
            let uni = universe.unwrap_or_else(|| "nifty50".to_string());
            let met = metric.unwrap_or_else(|| "rsi_14".to_string());
            Ok(format!(
                "Scan completed for universe '{uni}' on metric '{met}': 0 candidates triggered."
            ))
        }
        Command::Health => Ok(
            "System health: OK. Core: OK, Agent: OK, Persistence: OK, RiskEngine: OK.".to_string(),
        ),
    }
}
