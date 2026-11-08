//! Unit tests for the Ratatui TUI workspace tab navigation, rendering, and slash commands.

use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use vyapaar_highlight::HighlightClass;

use vyapaar_tui::agent_chat::{AgentMessage, AgentRole, ChatTranscript};
use vyapaar_tui::app::{ActiveTab, SLASH_COMMANDS, TuiApp};
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

    // Test /help with specific command
    app.execute_slash_command("/help agent");
    assert!(app.chat.last().unwrap().text.contains("Command /agent:"));

    // Test /agent
    app.execute_slash_command("/agent quant-deepseek");
    assert!(app.model_name.contains("quant-deepseek"));
    assert!(app.chat.last().unwrap().text.contains("quant-deepseek"));

    app.execute_slash_command("/agent off");
    assert!(
        app.chat
            .last()
            .unwrap()
            .text
            .contains("Reset active persona")
    );

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
fn test_hardened_quantcept_slash_commands() {
    let mut app = TuiApp::new();
    app.active_tab = ActiveTab::AgentChat;

    // /perf
    app.execute_slash_command("/perf");
    assert!(app.chat.last().unwrap().text.contains("Inference Speed"));

    // /context
    app.execute_slash_command("/context");
    assert!(app.chat.last().unwrap().text.contains("Token Budget"));

    // /remember
    app.execute_slash_command("/remember Always hedge NIFTY options delta");
    assert!(app.chat.last().unwrap().text.contains("Remembered:"));

    // /skills
    app.execute_slash_command("/skills");
    assert!(app.chat.last().unwrap().text.contains("quant"));
    assert!(app.chat.last().unwrap().text.contains("analyst"));

    // /mcp
    app.execute_slash_command("/mcp");
    assert!(app.chat.last().unwrap().text.contains("context7"));

    // /account
    app.execute_slash_command("/account");
    assert!(app.chat.last().unwrap().text.contains("SEBI Risk Engine"));

    // /settings
    app.execute_slash_command("/settings");
    assert!(app.chat.last().unwrap().text.contains("Theme"));

    // /usage
    app.execute_slash_command("/usage");
    assert!(app.chat.last().unwrap().text.contains("Local Tokens"));

    // /positions
    app.execute_slash_command("/positions");
    assert!(
        app.chat
            .last()
            .unwrap()
            .text
            .contains("Paper Trading Positions")
    );

    // /copy
    app.execute_slash_command("/copy full");
    assert!(
        app.chat
            .last()
            .unwrap()
            .text
            .contains("Copied full conversation transcript")
    );

    // /create-agent & /delete-agent
    app.execute_slash_command("/create-agent momentum-trader");
    assert!(app.chat.last().unwrap().text.contains("momentum-trader"));

    app.execute_slash_command("/delete-agent momentum-trader");
    assert!(app.chat.last().unwrap().text.contains("Deleted persona"));

    // /exit
    app.execute_slash_command("/exit");
    assert!(app.should_quit);
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

#[test]
fn test_palette_tab_autocomplete() {
    let mut app = TuiApp::new();
    app.is_palette_open = true;
    app.palette_query = "agent".to_string();

    let tab_key = KeyEvent {
        code: KeyCode::Tab,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    };

    app.on_key(tab_key);
    assert!(!app.is_palette_open);
    assert_eq!(app.active_tab, ActiveTab::AgentChat);
    assert!(app.is_input_mode);
    assert!(app.input_buffer.starts_with("/agent"));
}

#[test]
fn test_slash_key_opens_palette() {
    let mut app = TuiApp::new();
    assert!(!app.is_palette_open);

    let slash_key = KeyEvent {
        code: KeyCode::Char('/'),
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    };

    app.on_key(slash_key);
    assert!(app.is_palette_open);
    assert_eq!(app.palette_query, "/");
}

#[test]
fn test_all_catalog_slash_commands_present() {
    assert!(SLASH_COMMANDS.len() >= 35);
    let commands: Vec<&str> = SLASH_COMMANDS.iter().map(|(c, _)| *c).collect();
    assert!(commands.contains(&"/help"));
    assert!(commands.contains(&"/agent"));
    assert!(commands.contains(&"/jobs"));
    assert!(commands.contains(&"/memory"));
    assert!(commands.contains(&"/verify"));
    assert!(commands.contains(&"/perf"));
    assert!(commands.contains(&"/context"));
    assert!(commands.contains(&"/skills"));
    assert!(commands.contains(&"/mcp"));
    assert!(commands.contains(&"/quit"));
}

#[test]
fn test_algos_tab_state_and_render() {
    let backend = TestBackend::new(120, 40);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = TuiApp::new();
    app.active_tab = ActiveTab::Algos;

    assert!(app.algos.len() >= 4);
    assert_eq!(app.selected_algo_index, 0);

    // Navigate down with 'j'
    let key_j = KeyEvent {
        code: KeyCode::Char('j'),
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    };
    app.on_key(key_j);
    assert_eq!(app.selected_algo_index, 1);

    // Toggle status with Space
    let initial_status = app.selected_algo().unwrap().status;
    let key_space = KeyEvent {
        code: KeyCode::Char(' '),
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    };
    app.on_key(key_space);
    let new_status = app.selected_algo().unwrap().status;
    assert_eq!(new_status, initial_status.toggle());

    // Render frame
    terminal.draw(|frame| vyapaar_tui::ui::draw(frame, &app)).unwrap();
    let buffer = terminal.backend().buffer();
    let content: String = buffer.content().iter().map(|c| c.symbol()).collect();
    assert!(content.contains("Registered Algorithms"));
    assert!(content.contains("Parameter Inspector"));
    assert!(content.contains("Backtest"));
}

#[test]
fn test_backtest_tab_render_and_quadrants() {
    let backend = TestBackend::new(120, 40);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = TuiApp::new();
    app.active_tab = ActiveTab::Backtest;

    assert!(app.backtest_report.is_some());
    assert!(!app.backtest_equity_curve.is_empty());
    assert!(app.trade_suggestion.is_some());

    terminal.draw(|frame| vyapaar_tui::ui::draw(frame, &app)).unwrap();
    let buffer = terminal.backend().buffer();
    let content: String = buffer.content().iter().map(|c| c.symbol()).collect();
    assert!(content.contains("Key Performance Indicators"));
    assert!(content.contains("Parameter Delta"));
    assert!(content.contains("Analytical Equity Curve"));
    assert!(content.contains("Optimal Trade Suggestion"));
}

#[test]
fn test_slash_commands_phase3_workflow() {
    let mut app = TuiApp::new();

    // 1. /algos
    app.execute_slash_command("/algos");
    assert_eq!(app.active_tab, ActiveTab::Algos);

    // 2. /backtest turtle_breakout
    app.execute_slash_command("/backtest turtle_breakout");
    assert_eq!(app.active_tab, ActiveTab::Backtest);
    assert!(app.backtest_report.is_some());

    // 3. /tune turtle_breakout
    app.execute_slash_command("/tune turtle_breakout");
    assert_eq!(app.active_tab, ActiveTab::Backtest);
    assert!(app.optimization_result.is_some());

    // 4. /suggest turtle_breakout
    app.execute_slash_command("/suggest turtle_breakout");
    assert_eq!(app.active_tab, ActiveTab::Backtest);
    assert!(app.trade_suggestion.is_some());
}

#[test]
fn test_interactive_hotkeys_algos_to_backtest() {
    let mut app = TuiApp::new();
    app.active_tab = ActiveTab::Algos;

    // Press Enter to backtest selected
    let key_enter = KeyEvent {
        code: KeyCode::Enter,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    };
    app.on_key(key_enter);
    assert_eq!(app.active_tab, ActiveTab::Backtest);

    // Return to Algos and press 'T' to tune
    app.active_tab = ActiveTab::Algos;
    let key_t = KeyEvent {
        code: KeyCode::Char('t'),
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    };
    app.on_key(key_t);
    assert_eq!(app.active_tab, ActiveTab::Backtest);
    assert!(app.optimization_result.is_some());

    // Return to Algos and press 'S' to suggest
    app.active_tab = ActiveTab::Algos;
    let key_s = KeyEvent {
        code: KeyCode::Char('s'),
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    };
    app.on_key(key_s);
    assert_eq!(app.active_tab, ActiveTab::Backtest);
    assert!(app.trade_suggestion.is_some());
}
