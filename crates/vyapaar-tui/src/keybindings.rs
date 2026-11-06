//! Keybinding map: crossterm key events → application actions.

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// An action the TUI understands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    /// Quit the application.
    Quit,
    /// Activate the next tab.
    NextTab,
    /// Activate the previous tab.
    PreviousTab,
    /// Move selection down.
    Down,
    /// Move selection up.
    Up,
    /// Submit the current input (chat send, scan run).
    Submit,
}

/// A single keybinding (for documentation and remapping).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KeyBinding {
    /// The key code bound.
    pub key: KeyCode,
    /// Modifiers required.
    pub modifiers: KeyModifiers,
    /// The action this binding triggers.
    pub action: Action,
}

/// The default keybinding table.
pub const DEFAULT_KEYBINDINGS: &[KeyBinding] = &[
    KeyBinding {
        key: KeyCode::Char('q'),
        modifiers: KeyModifiers::NONE,
        action: Action::Quit,
    },
    KeyBinding {
        key: KeyCode::Char('l'),
        modifiers: KeyModifiers::NONE,
        action: Action::NextTab,
    },
    KeyBinding {
        key: KeyCode::Char('h'),
        modifiers: KeyModifiers::NONE,
        action: Action::PreviousTab,
    },
    KeyBinding {
        key: KeyCode::Tab,
        modifiers: KeyModifiers::NONE,
        action: Action::NextTab,
    },
    KeyBinding {
        key: KeyCode::BackTab,
        modifiers: KeyModifiers::SHIFT,
        action: Action::PreviousTab,
    },
    KeyBinding {
        key: KeyCode::Down,
        modifiers: KeyModifiers::NONE,
        action: Action::Down,
    },
    KeyBinding {
        key: KeyCode::Up,
        modifiers: KeyModifiers::NONE,
        action: Action::Up,
    },
    KeyBinding {
        key: KeyCode::Char('j'),
        modifiers: KeyModifiers::NONE,
        action: Action::Down,
    },
    KeyBinding {
        key: KeyCode::Char('k'),
        modifiers: KeyModifiers::NONE,
        action: Action::Up,
    },
    KeyBinding {
        key: KeyCode::Enter,
        modifiers: KeyModifiers::NONE,
        action: Action::Submit,
    },
];

/// Look up the action for a key event using the default keybinding table.
pub fn handle_key(key: KeyEvent) -> Option<Action> {
    DEFAULT_KEYBINDINGS
        .iter()
        .find(|b| b.key == key.code && b.modifiers == key.modifiers)
        .map(|b| b.action)
}
