//! Unit tests for the Ratatui TUI workspace tab navigation, rendering, and slash commands.

use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use vyapaar_highlight::HighlightClass;

use vyapaar_tui::agent_chat::{AgentMessage, AgentRole, ChatTranscript};
use vyapaar_tui::app::{ActiveTab, TuiApp};
use vyapaar_tui::keybindings::{Action, handle_key};
use vyapaar_tui::layout::LayoutModel;

#[test]
fn new_app_starts_on_watchlist() {
    let app = TuiApp::new();
    assert_eq!(app.active_tab, ActiveTab::Watchlist);
    assert!(!app.should_quit);
    assert_eq!(app.selected_index, 0);
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
}

#[test]
fn tab_order_wraps() {
    let mut tab = ActiveTab::Watchlist;
    for expected in [
        ActiveTab::Screener,
        ActiveTab::Portfolio,
        ActiveTab::Orders,
        ActiveTab::Backtest,
        ActiveTab::Algos,
        ActiveTab::AgentChat,
        ActiveTab::Watchlist,
    ] {
        tab = tab.next();
        assert_eq!(tab, expected);
    }

    tab = ActiveTab::Watchlist;
    assert_eq!(tab.previous(), ActiveTab::AgentChat);
}

#[test]
fn keybindings_map_chars_and_navigation() {
    let key = |code: KeyCode| KeyEvent {
        code,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    };

    assert_eq!(handle_key(key(KeyCode::Char('q'))), Some(Action::Quit));
    assert_eq!(handle_key(key(KeyCode::Tab)), Some(Action::NextTab));
    assert_eq!(handle_key(key(KeyCode::Char('j'))), Some(Action::Down));
    assert_eq!(handle_key(key(KeyCode::Char('k'))), Some(Action::Up));
    assert_eq!(handle_key(key(KeyCode::Enter)), Some(Action::Submit));
    assert_eq!(handle_key(key(KeyCode::Char('x'))), None);
}

#[test]
fn app_apply_quit_and_tabs() {
    let mut app = TuiApp::new();
    app.apply(Action::NextTab);
    assert_eq!(app.active_tab, ActiveTab::Screener);
    app.apply(Action::PreviousTab);
    assert_eq!(app.active_tab, ActiveTab::Watchlist);
    app.apply(Action::Quit);
    assert!(app.should_quit);
}

#[test]
fn selection_moves_and_clamps_at_zero() {
    let mut app = TuiApp::new();
    assert_eq!(app.selected_index, 0);
    app.apply(Action::Up);
    assert_eq!(app.selected_index, 0);
    app.apply(Action::Down);
    assert_eq!(app.selected_index, 1);
    app.apply(Action::Down);
    assert_eq!(app.selected_index, 2);
    app.apply(Action::Up);
    assert_eq!(app.selected_index, 1);
}

#[test]
fn chat_transcript_appends_and_highlights() {
    let mut chat = ChatTranscript::new();
    assert!(chat.is_empty());
    chat.push(AgentMessage::new(AgentRole::User, "hi"));
    chat.push(AgentMessage::new(AgentRole::Agent, "hello"));
    assert_eq!(chat.len(), 2);
    assert_eq!(chat.last().unwrap().role, AgentRole::Agent);
}

#[test]
fn layout_model_splits_regions() {
    let area = ratatui::layout::Rect::new(0, 0, 100, 30);
    let layout = LayoutModel::new(area);
    assert_eq!(layout.header.height, 1);
    assert_eq!(layout.tabs.height, 1);
    assert_eq!(layout.footer.height, 1);
    assert_eq!(layout.content.height, 27);
    assert_eq!(layout.content.width, 100);
}

#[test]
fn terminal_can_draw_an_empty_app_frame() {
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    let app = TuiApp::new();
    terminal
        .draw(|frame| {
            let _ = frame.area();
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

#[test]
fn test_draw_agent_chat_with_messages() {
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = TuiApp::new();
    app.active_tab = ActiveTab::AgentChat;
    app.chat
        .push(AgentMessage::new(AgentRole::User, "Hello agent"));
    app.chat.push(AgentMessage::new(
        AgentRole::Agent,
        "Hello trader! How can I help you?",
    ));
    terminal
        .draw(|frame| vyapaar_tui::ui::draw(frame, &app))
        .unwrap();
    let mut found_user = false;
    let mut found_agent = false;
    for y in 0..24 {
        let mut row = String::new();
        for x in 0..80 {
            row.push_str(terminal.backend().buffer()[(x, y)].symbol());
        }
        if row.contains("Hello agent") {
            found_user = true;
        }
        if row.contains("Hello trader!") {
            found_agent = true;
        }
    }
    assert!(found_user);
    assert!(found_agent);
}

#[test]
fn test_slash_commands_execution() {
    let mut app = TuiApp::new();
    app.active_tab = ActiveTab::AgentChat;

    // Test /help
    app.execute_slash_command("/help");
    assert!(
        app.chat
            .last()
            .unwrap()
            .text
            .contains("Vyapaar Slash Commands")
    );

    // Test /agent
    app.execute_slash_command("/agent quant-deepseek");
    assert_eq!(app.model_name, "quant-deepseek");
    assert!(app.chat.last().unwrap().text.contains("quant-deepseek"));

    // Test /undo and /redo
    app.chat
        .push(AgentMessage::new(AgentRole::User, "What is VaR?"));
    app.chat
        .push(AgentMessage::new(AgentRole::Agent, "Value at Risk."));
    let count_before = app.chat.len();
    app.execute_slash_command("/undo");
    assert_eq!(app.chat.len(), count_before - 2 + 1); // 2 popped + 1 notice added

    app.execute_slash_command("/redo");
    assert!(app.chat.messages.iter().any(|m| m.text == "Value at Risk."));

    // Test /clear
    app.execute_slash_command("/clear");
    assert!(app.chat.is_empty());
}

#[test]
fn test_command_palette_modal_and_filtering() {
    let mut app = TuiApp::new();
    app.is_palette_open = true;
    app.palette_query = "job".to_string();

    let filtered = app.filtered_palette();
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].0, "/jobs");

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|frame| vyapaar_tui::ui::draw(frame, &app))
        .unwrap();

    let mut found_palette = false;
    for y in 0..24 {
        let mut row = String::new();
        for x in 0..80 {
            row.push_str(terminal.backend().buffer()[(x, y)].symbol());
        }
        if row.contains("Command Palette") {
            found_palette = true;
            break;
        }
    }
    assert!(found_palette);
}
