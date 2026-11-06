//! The four-tab workstation model.

use serde::{Deserialize, Serialize};

use crate::agent_chat::ChatTranscript;
use crate::keybindings::{Action, handle_key};
use ratatui::crossterm::event::KeyEvent;

/// The four workstation tabs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActiveTab {
    /// Neural AST screener.
    Screener,
    /// Portfolio snapshot.
    Portfolio,
    /// Live orders.
    Orders,
    /// Agent chat.
    AgentChat,
}

impl ActiveTab {
    /// All tabs in tab-order.
    pub const ALL: [ActiveTab; 4] = [
        ActiveTab::Screener,
        ActiveTab::Portfolio,
        ActiveTab::Orders,
        ActiveTab::AgentChat,
    ];

    /// The display title of the tab.
    pub fn title(&self) -> &'static str {
        match self {
            Self::Screener => "Screener",
            Self::Portfolio => "Portfolio",
            Self::Orders => "Orders",
            Self::AgentChat => "Agent",
        }
    }

    /// The next tab in tab-order (wrapping).
    pub fn next(self) -> Self {
        match self {
            Self::Screener => Self::Portfolio,
            Self::Portfolio => Self::Orders,
            Self::Orders => Self::AgentChat,
            Self::AgentChat => Self::Screener,
        }
    }

    /// The previous tab in tab-order (wrapping).
    pub fn previous(self) -> Self {
        match self {
            Self::Screener => Self::AgentChat,
            Self::Portfolio => Self::Screener,
            Self::Orders => Self::Portfolio,
            Self::AgentChat => Self::Orders,
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
}

impl TuiApp {
    /// Create a new TUI app starting on the screener tab.
    pub fn new() -> Self {
        Self {
            active_tab: ActiveTab::Screener,
            should_quit: false,
            chat: ChatTranscript::new(),
            selected_index: 0,
        }
    }

    /// Handle a key event through the keybinding map.
    pub fn on_key(&mut self, key: KeyEvent) {
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
            Action::Submit => {}
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
