//! The seven-tab workstation model.

use serde::{Deserialize, Serialize};

use crate::agent_chat::{AgentMessage, AgentRole, ChatTranscript};
use crate::keybindings::{Action, handle_key};
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind};

/// The workstation tabs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActiveTab {
    /// Watchlist / dashboard.
    Watchlist,
    /// Neural AST screener.
    Screener,
    /// Portfolio snapshot.
    Portfolio,
    /// Live orders.
    Orders,
    /// Backtest results viewer.
    Backtest,
    /// Algo / strategy management.
    Algos,
    /// Agent chat.
    AgentChat,
}

impl ActiveTab {
    /// All tabs in tab-order.
    pub const ALL: [ActiveTab; 7] = [
        ActiveTab::Watchlist,
        ActiveTab::Screener,
        ActiveTab::Portfolio,
        ActiveTab::Orders,
        ActiveTab::Backtest,
        ActiveTab::Algos,
        ActiveTab::AgentChat,
    ];

    /// The display title of the tab.
    pub fn title(&self) -> &'static str {
        match self {
            Self::Watchlist => "Watchlist",
            Self::Screener => "Screener",
            Self::Portfolio => "Portfolio",
            Self::Orders => "Orders",
            Self::Backtest => "Backtest",
            Self::Algos => "Algos",
            Self::AgentChat => "Agent",
        }
    }

    /// The next tab in tab-order (wrapping).
    pub fn next(self) -> Self {
        match self {
            Self::Watchlist => Self::Screener,
            Self::Screener => Self::Portfolio,
            Self::Portfolio => Self::Orders,
            Self::Orders => Self::Backtest,
            Self::Backtest => Self::Algos,
            Self::Algos => Self::AgentChat,
            Self::AgentChat => Self::Watchlist,
        }
    }

    /// The previous tab in tab-order (wrapping).
    pub fn previous(self) -> Self {
        match self {
            Self::Watchlist => Self::AgentChat,
            Self::Screener => Self::Watchlist,
            Self::Portfolio => Self::Screener,
            Self::Orders => Self::Portfolio,
            Self::Backtest => Self::Orders,
            Self::Algos => Self::Backtest,
            Self::AgentChat => Self::Algos,
        }
    }
}

/// The terminal application state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TuiApp {
    /// The active tab.
    pub active_tab: ActiveTab,
    /// Whether the user requested to quit.
    pub should_quit: bool,
    /// The agent chat transcript.
    pub chat: ChatTranscript,
    /// Index of the selected row in the active data tab.
    pub selected_index: usize,
    /// Chat input buffer for prompt typing.
    pub input_buffer: String,
    /// Whether user is currently in input editing mode.
    pub is_input_mode: bool,
    /// Whether the agent is actively generating a completion.
    pub is_loading: bool,
    /// Active model name.
    pub model_name: String,
    /// Prompt pending submission to agent.
    pub pending_prompt: Option<String>,
}

impl TuiApp {
    /// Create a new TUI app starting on the watchlist dashboard.
    pub fn new() -> Self {
        let mut chat = ChatTranscript::new();
        chat.push(AgentMessage::new(
            AgentRole::Agent,
            "Namaste! I am Vyapaar Agent, your local in-process AI trading assistant powered by SmolLM2 (Candle GGUF).\n\
Ask me anything about market microstructure, order books, technical strategies, risk management, or Indian equities (NSE/BSE).\n\
Press 'i' or Enter to start typing your prompt.",
        ));

        Self {
            active_tab: ActiveTab::Watchlist,
            should_quit: false,
            chat,
            selected_index: 0,
            input_buffer: String::new(),
            is_input_mode: false,
            is_loading: false,
            model_name: "SmolLM2-360M-Instruct (Candle GGUF)".to_string(),
            pending_prompt: None,
        }
    }

    /// Set model name display.
    pub fn with_model_name(mut self, name: impl Into<String>) -> Self {
        self.model_name = name.into();
        self
    }

    /// Handle a key event through the keybinding map.
    pub fn on_key(&mut self, key: KeyEvent) {
        if key.kind == KeyEventKind::Release {
            return;
        }

        if self.active_tab == ActiveTab::AgentChat && self.is_input_mode {
            match key.code {
                KeyCode::Esc => {
                    self.is_input_mode = false;
                }
                KeyCode::Enter => {
                    let prompt = self.input_buffer.trim().to_string();
                    if !prompt.is_empty() {
                        self.chat
                            .push(AgentMessage::new(AgentRole::User, prompt.clone()));
                        self.chat.push(AgentMessage::new(AgentRole::Agent, ""));
                        self.pending_prompt = Some(prompt);
                        self.input_buffer.clear();
                        self.is_loading = true;
                        self.is_input_mode = false;
                    }
                }
                KeyCode::Backspace => {
                    self.input_buffer.pop();
                }
                KeyCode::Char(c) => {
                    self.input_buffer.push(c);
                }
                _ => {}
            }
            return;
        }

        // Direct tab switching with digits 1 through 7
        if let KeyCode::Char(c @ '1'..='7') = key.code {
            let idx = (c as u8 - b'1') as usize;
            if idx < ActiveTab::ALL.len() {
                self.active_tab = ActiveTab::ALL[idx];
                return;
            }
        }

        // Quick enter to chat input when on Agent tab
        if self.active_tab == ActiveTab::AgentChat && key.code == KeyCode::Char('i') {
            self.is_input_mode = true;
            return;
        }

        if let Some(action) = handle_key(key) {
            self.apply(action);
        }
    }

    /// Apply an action to the app state.
    pub fn apply(&mut self, action: Action) {
        match action {
            Action::Quit => self.should_quit = true,
            Action::NextTab => self.active_tab = self.active_tab.next(),
            Action::PreviousTab => self.active_tab = self.active_tab.previous(),
            Action::Down | Action::Up => {
                self.move_selection(action);
            }
            Action::Submit => {
                if self.active_tab == ActiveTab::AgentChat {
                    self.is_input_mode = true;
                }
            }
        }
    }

    fn move_selection(&mut self, action: Action) {
        match action {
            Action::Down => self.selected_index = self.selected_index.saturating_add(1),
            Action::Up => self.selected_index = self.selected_index.saturating_sub(1),
            _ => {}
        }
    }
}

impl Default for TuiApp {
    fn default() -> Self {
        Self::new()
    }
}
