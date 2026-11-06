use vyapaar_agent_chat::{SessionEvent, TurnId, TurnOutcome};
use vyapaar_agent_llm::Usage;
use vyapaar_tui::{ActiveTab, AgentChatTab, ChatDisplayItem, ChatStatus, KeyAction, TuiApp};

#[test]
fn test_tab_navigation_and_actions() {
    let mut app = TuiApp::new();
    assert_eq!(app.active_tab, ActiveTab::AgentChat);

    app.handle_action(KeyAction::NextTab);
    assert_eq!(app.active_tab, ActiveTab::Screener);

    app.handle_action(KeyAction::NextTab);
    assert_eq!(app.active_tab, ActiveTab::Portfolio);

    app.handle_action(KeyAction::PrevTab);
    assert_eq!(app.active_tab, ActiveTab::Screener);

    app.handle_action(KeyAction::SelectTab(ActiveTab::Orders));
    assert_eq!(app.active_tab, ActiveTab::Orders);

    assert_eq!(app.layout.content_height(), 36);

    app.handle_action(KeyAction::Quit);
    assert!(app.should_quit);
}

#[test]
fn test_agent_chat_event_streaming() {
    let mut tab = AgentChatTab::new();
    assert_eq!(tab.status, ChatStatus::Idle);

    tab.input_buffer = "What is the RSI of INFY?".to_string();
    let query = tab.submit_input().expect("some query");
    assert_eq!(query, "What is the RSI of INFY?");
    assert_eq!(tab.status, ChatStatus::Streaming);
    assert_eq!(tab.items.len(), 1);
    assert!(matches!(tab.items[0], ChatDisplayItem::User(_)));

    let turn_id = TurnId(1);
    tab.handle_event(SessionEvent::TurnStarted {
        turn: turn_id,
        queued_behind: 0,
    });
    tab.handle_event(SessionEvent::TextDelta {
        turn: turn_id,
        round: 1,
        text: "The RSI of INFY ".to_string(),
    });
    tab.handle_event(SessionEvent::TextDelta {
        turn: turn_id,
        round: 1,
        text: "is 58.4.".to_string(),
    });
    assert_eq!(tab.current_streaming, "The RSI of INFY is 58.4.");

    tab.handle_event(SessionEvent::TurnEnded {
        turn: turn_id,
        outcome: TurnOutcome::Completed,
        usage: Usage::default(),
        rounds: 1,
        persisted: true,
    });
    assert_eq!(tab.status, ChatStatus::Idle);
    assert_eq!(tab.items.len(), 2);
    assert_eq!(
        tab.items[1],
        ChatDisplayItem::Assistant("The RSI of INFY is 58.4.".to_string())
    );
}
