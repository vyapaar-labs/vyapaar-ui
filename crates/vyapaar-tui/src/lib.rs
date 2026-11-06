//! Terminal User Interface (TUI) workstation model for Vyapaar.
//!
//! Provides layout, keybinding management, tab routing (`ActiveTab`),
//! and the `AgentChat` tab state machine wired to [`vyapaar_agent_chat`].

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use vyapaar_agent_chat::{
    Notice, PermissionDecision, PermissionPrompt, SessionEvent, SessionHandle, ToolOutcome,
    TurnOutcome,
};

/// Navigation tabs in the workstation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActiveTab {
    /// Market screening and scanner catalog.
    Screener,
    /// Account portfolio, holdings, and risk metrics.
    Portfolio,
    /// Active, filled, and pending order book.
    Orders,
    /// Conversational AI agent terminal.
    #[default]
    AgentChat,
}

/// Key actions handled by the TUI shell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeyAction {
    /// Switch to the next tab.
    NextTab,
    /// Switch to the previous tab.
    PrevTab,
    /// Select a specific tab directly.
    SelectTab(ActiveTab),
    /// Submit current input buffer.
    Submit,
    /// Cancel current ongoing turn or operation.
    Cancel,
    /// Accept a pending permission prompt.
    AcceptPermission,
    /// Reject a pending permission prompt.
    RejectPermission,
    /// Quit application.
    Quit,
}

/// Rendered message representation in the chat tab.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChatDisplayItem {
    /// User message.
    User(String),
    /// Assistant streaming or final text.
    Assistant(String),
    /// Tool call status with summary.
    ToolCall {
        name: String,
        input: String,
        outcome: Option<String>,
    },
    /// Status notice from system.
    SystemNotice(String),
}

/// Streaming status of the active agent chat session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatStatus {
    /// Idle, ready for next user query.
    Idle,
    /// Model response is streaming deltas.
    Streaming,
    /// Waiting for user response to a permission prompt.
    AwaitingPermission,
    /// Turn ended with error or cancellation.
    Failed,
}

/// State machine for the AgentChat tab.
#[derive(Debug)]
pub struct AgentChatTab {
    /// Linear history of display items in the transcript.
    pub items: Vec<ChatDisplayItem>,
    /// Active streaming buffer for the current turn.
    pub current_streaming: String,
    /// Current input buffer being typed by user.
    pub input_buffer: String,
    /// Active state of the chat engine.
    pub status: ChatStatus,
    /// Active permission prompt if awaiting confirmation.
    pub pending_permission: Option<PermissionPrompt>,
    /// Connected session handle, if open.
    pub session_handle: Option<SessionHandle>,
    /// Queue of notices received.
    pub notices: VecDeque<Notice>,
}

impl Default for AgentChatTab {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentChatTab {
    /// Create a new, empty AgentChat tab.
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            current_streaming: String::new(),
            input_buffer: String::new(),
            status: ChatStatus::Idle,
            pending_permission: None,
            session_handle: None,
            notices: VecDeque::new(),
        }
    }

    /// Attach a running session handle.
    pub fn attach_session(&mut self, handle: SessionHandle) {
        self.session_handle = Some(handle);
    }

    /// Append typed user input to chat history and prepare for turn.
    pub fn submit_input(&mut self) -> Option<String> {
        let text = self.input_buffer.trim().to_string();
        if text.is_empty() {
            return None;
        }
        self.input_buffer.clear();
        self.items.push(ChatDisplayItem::User(text.clone()));
        self.current_streaming.clear();
        self.status = ChatStatus::Streaming;
        Some(text)
    }

    /// Ingest a typed event from [`vyapaar_agent_chat::SessionEvent`].
    pub fn handle_event(&mut self, event: SessionEvent) {
        match event {
            SessionEvent::TurnStarted { .. } => {
                self.status = ChatStatus::Streaming;
                self.current_streaming.clear();
            }
            SessionEvent::TextDelta { text, .. } => {
                self.current_streaming.push_str(&text);
            }
            SessionEvent::ToolCallReady { call, .. } => {
                self.items.push(ChatDisplayItem::ToolCall {
                    name: call.name,
                    input: call.arguments.to_string(),
                    outcome: None,
                });
            }
            SessionEvent::ToolFinished {
                name: _, outcome, ..
            } => {
                let outcome_text = match outcome {
                    ToolOutcome::Success => "Success".to_string(),
                    ToolOutcome::Error(error) => format!("Error: {error}"),
                    ToolOutcome::Denied(reason) => format!("Denied: {reason}"),
                };
                if let Some(ChatDisplayItem::ToolCall { outcome: o, .. }) = self
                    .items
                    .iter_mut()
                    .rev()
                    .find(|item| matches!(item, ChatDisplayItem::ToolCall { .. }))
                {
                    *o = Some(outcome_text);
                }
            }
            SessionEvent::PermissionRequested(prompt) => {
                self.status = ChatStatus::AwaitingPermission;
                self.pending_permission = Some(prompt);
            }
            SessionEvent::Notice(notice) => {
                let msg = format!("{notice:?}");
                self.notices.push_back(notice);
                self.items.push(ChatDisplayItem::SystemNotice(msg));
            }
            SessionEvent::TurnEnded { outcome, .. } => {
                if !self.current_streaming.is_empty() {
                    let text = std::mem::take(&mut self.current_streaming);
                    self.items.push(ChatDisplayItem::Assistant(text));
                }
                self.status = match outcome {
                    TurnOutcome::Completed => ChatStatus::Idle,
                    TurnOutcome::Cancelled => {
                        self.items
                            .push(ChatDisplayItem::SystemNotice("Turn cancelled.".to_string()));
                        ChatStatus::Idle
                    }
                    TurnOutcome::RoundLimit { limit } => {
                        self.items.push(ChatDisplayItem::SystemNotice(format!(
                            "Round limit reached: {limit}"
                        )));
                        ChatStatus::Idle
                    }
                    TurnOutcome::MaxTokens => {
                        self.items.push(ChatDisplayItem::SystemNotice(
                            "Max tokens reached.".to_string(),
                        ));
                        ChatStatus::Idle
                    }
                    TurnOutcome::Refused { reason } => {
                        self.items
                            .push(ChatDisplayItem::SystemNotice(format!("Refused: {reason}")));
                        ChatStatus::Failed
                    }
                    TurnOutcome::Stopped => {
                        self.items
                            .push(ChatDisplayItem::SystemNotice("Turn stopped.".to_string()));
                        ChatStatus::Idle
                    }
                    TurnOutcome::Failed { error } => {
                        self.items.push(ChatDisplayItem::SystemNotice(format!(
                            "Failed: {}",
                            error.message
                        )));
                        ChatStatus::Failed
                    }
                };
            }
            _ => {}
        }
    }

    /// Respond to a pending permission prompt.
    pub async fn respond_permission(&mut self, decision: PermissionDecision) -> Result<(), String> {
        if let Some(prompt) = self.pending_permission.take() {
            if let Some(responder) = prompt.responder {
                responder
                    .respond(decision)
                    .map_err(|e| format!("failed to deliver permission decision: {e}"))?;
                self.status = ChatStatus::Streaming;
                Ok(())
            } else {
                Err("prompt has no interactive responder".to_string())
            }
        } else {
            Err("no pending permission prompt".to_string())
        }
    }
}

/// Workstation screen layout specification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TuiLayout {
    /// Screen width in characters.
    pub width: u16,
    /// Screen height in characters.
    pub height: u16,
    /// Tab bar height.
    pub tab_bar_height: u16,
    /// Status bar height.
    pub status_bar_height: u16,
}

impl Default for TuiLayout {
    fn default() -> Self {
        Self {
            width: 120,
            height: 40,
            tab_bar_height: 3,
            status_bar_height: 1,
        }
    }
}

impl TuiLayout {
    /// Calculate main content viewport height.
    pub fn content_height(&self) -> u16 {
        self.height
            .saturating_sub(self.tab_bar_height + self.status_bar_height)
    }
}

/// Primary TUI application coordinator.
#[derive(Debug)]
pub struct TuiApp {
    /// Currently active workspace tab.
    pub active_tab: ActiveTab,
    /// Terminal layout geometry.
    pub layout: TuiLayout,
    /// State of the agent chat interface.
    pub chat_tab: AgentChatTab,
    /// Whether the user requested shutdown.
    pub should_quit: bool,
}

impl Default for TuiApp {
    fn default() -> Self {
        Self::new()
    }
}

impl TuiApp {
    /// Create a new TUI application instance.
    pub fn new() -> Self {
        Self {
            active_tab: ActiveTab::AgentChat,
            layout: TuiLayout::default(),
            chat_tab: AgentChatTab::new(),
            should_quit: false,
        }
    }

    /// Switch active tab.
    pub fn switch_tab(&mut self, tab: ActiveTab) {
        self.active_tab = tab;
    }

    /// Cycle to next tab.
    pub fn next_tab(&mut self) {
        self.active_tab = match self.active_tab {
            ActiveTab::Screener => ActiveTab::Portfolio,
            ActiveTab::Portfolio => ActiveTab::Orders,
            ActiveTab::Orders => ActiveTab::AgentChat,
            ActiveTab::AgentChat => ActiveTab::Screener,
        };
    }

    /// Cycle to previous tab.
    pub fn prev_tab(&mut self) {
        self.active_tab = match self.active_tab {
            ActiveTab::Screener => ActiveTab::AgentChat,
            ActiveTab::Portfolio => ActiveTab::Screener,
            ActiveTab::Orders => ActiveTab::Portfolio,
            ActiveTab::AgentChat => ActiveTab::Orders,
        };
    }

    /// Dispatch a high-level key action.
    pub fn handle_action(&mut self, action: KeyAction) {
        match action {
            KeyAction::NextTab => self.next_tab(),
            KeyAction::PrevTab => self.prev_tab(),
            KeyAction::SelectTab(t) => self.switch_tab(t),
            KeyAction::Quit => self.should_quit = true,
            KeyAction::Submit => {
                if self.active_tab == ActiveTab::AgentChat {
                    self.chat_tab.submit_input();
                }
            }
            KeyAction::Cancel => {
                if let Some(ref handle) = self.chat_tab.session_handle {
                    let h = handle.clone();
                    tokio::spawn(async move {
                        let _ = h.cancel().await;
                    });
                }
            }
            _ => {}
        }
    }
}
