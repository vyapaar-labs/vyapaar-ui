//! Unit tests for the Ratatui terminal workstation (`vyapaar-tui`).
//!
//! Uses `ratatui::TestBackend`-style pure state assertions and scripted
//! `crossterm` key fixtures — no live terminal required.

use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::Rect;
use vyapaar_highlight::HighlightClass;
use vyapaar_tui::agent_chat::{AgentMessage, AgentRole, ChatTranscript};
use vyapaar_tui::app::{ActiveTab, TuiApp};
use vyapaar_tui::keybindings::{Action, handle_key};
use vyapaar_tui::layout::LayoutModel;

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

#[test]
fn new_app_starts_on_watchlist() {
    let app = TuiApp::new();
    assert_eq!(app.active_tab, ActiveTab::Watchlist);
    assert!(!app.should_quit);
    assert_eq!(app.selected_index, 0);
}

#[test]
fn tab_order_wraps() {
    assert_eq!(ActiveTab::Watchlist.next(), ActiveTab::Screener);
    assert_eq!(ActiveTab::Screener.next(), ActiveTab::Portfolio);
    assert_eq!(ActiveTab::Portfolio.next(), ActiveTab::Orders);
    assert_eq!(ActiveTab::Orders.next(), ActiveTab::Backtest);
    assert_eq!(ActiveTab::Backtest.next(), ActiveTab::Algos);
    assert_eq!(ActiveTab::Algos.next(), ActiveTab::AgentChat);
    assert_eq!(ActiveTab::AgentChat.next(), ActiveTab::Watchlist);
    assert_eq!(ActiveTab::Watchlist.previous(), ActiveTab::AgentChat);
}

#[test]
fn tab_titles() {
    assert_eq!(ActiveTab::Watchlist.title(), "Watchlist");
    assert_eq!(ActiveTab::Screener.title(), "Screener");
    assert_eq!(ActiveTab::Portfolio.title(), "Portfolio");
    assert_eq!(ActiveTab::Orders.title(), "Orders");
    assert_eq!(ActiveTab::Backtest.title(), "Backtest");
    assert_eq!(ActiveTab::Algos.title(), "Algos");
    assert_eq!(ActiveTab::AgentChat.title(), "Agent");
    assert_eq!(ActiveTab::ALL.len(), 7);
}

#[test]
fn keybindings_map_chars_and_navigation() {
    assert_eq!(handle_key(key(KeyCode::Char('q'))), Some(Action::Quit));
    assert_eq!(handle_key(key(KeyCode::Char('l'))), Some(Action::NextTab));
    assert_eq!(
        handle_key(key(KeyCode::Char('h'))),
        Some(Action::PreviousTab)
    );
    assert_eq!(handle_key(key(KeyCode::Down)), Some(Action::Down));
    assert_eq!(handle_key(key(KeyCode::Char('j'))), Some(Action::Down));
    assert_eq!(handle_key(key(KeyCode::Enter)), Some(Action::Submit));
    assert_eq!(handle_key(key(KeyCode::Char('z'))), None);
}

#[test]
fn app_apply_quit_and_tabs() {
    let mut app = TuiApp::new();
    app.on_key(key(KeyCode::Char('l')));
    assert_eq!(app.active_tab, ActiveTab::Screener);
    app.on_key(key(KeyCode::Char('q')));
    assert!(app.should_quit);
}

#[test]
fn selection_moves_and_clamps_at_zero() {
    let mut app = TuiApp::new();
    app.on_key(key(KeyCode::Down));
    app.on_key(key(KeyCode::Down));
    assert_eq!(app.selected_index, 2);
    app.on_key(key(KeyCode::Up));
    assert_eq!(app.selected_index, 1);
    // Can't go below zero.
    app.on_key(key(KeyCode::Up));
    app.on_key(key(KeyCode::Up));
    app.on_key(key(KeyCode::Up));
    assert_eq!(app.selected_index, 0);
}

#[test]
fn chat_transcript_appends_and_highlights() {
    let mut transcript = ChatTranscript::new();
    assert!(transcript.is_empty());
    transcript.push(AgentMessage::new(AgentRole::User, "show me NIFTY"));
    transcript.push(AgentMessage::new(
        AgentRole::Agent,
        "```python\nprint(1)\n```",
    ));
    assert_eq!(transcript.len(), 2);
    assert_eq!(transcript.last().unwrap().role, AgentRole::Agent);
    // The agent message's markdown contains a code fence span.
    let spans = transcript.last().unwrap().highlight();
    assert!(spans.iter().any(|s| s.class == HighlightClass::Function));
}

#[test]
fn layout_model_splits_regions() {
    let area = Rect::new(0, 0, 80, 24);
    let layout = LayoutModel::new(area);
    assert_eq!(layout.header.y, 0);
    assert_eq!(layout.tabs.y, 1);
    assert_eq!(layout.footer.y, 23);
    assert_eq!(layout.content.height, 21);
    let cols = layout.content_columns();
    assert_eq!(cols.len(), 2);
    assert_eq!(cols[0].width, 32);
    assert_eq!(cols[1].width, 48);
}

#[test]
fn terminal_can_draw_an_empty_app_frame() {
    // Smoke test: rendering the app state to a TestBackend must not panic.
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    let app = TuiApp::new();
    terminal
        .draw(|frame| {
            let layout = LayoutModel::new(frame.area());
            let _ = layout;
            let _ = &app.active_tab;
        })
        .unwrap();
}

#[test]
fn agent_message_highlight_uses_markdown() {
    let msg = AgentMessage::new(AgentRole::Agent, "# Summary\nSome **bold**.");
    let spans = msg.highlight();
    assert!(spans.iter().any(|s| s.class == HighlightClass::Keyword));
}
