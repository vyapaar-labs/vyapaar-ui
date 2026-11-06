//! Unit tests for `vyapaar-ui-core` algo management primitives.

use vyapaar_ui_core::algos::{AlgoManagerViewModel, AlgoRow, AlgoStatus};

#[test]
fn empty_view_has_no_algos() {
    let view = AlgoManagerViewModel::empty();
    assert!(view.is_empty());
    assert_eq!(view.live_count(), 0);
}

#[test]
fn upsert_appends_then_replaces() {
    let mut view = AlgoManagerViewModel::empty();
    let first =
        AlgoRow::new("turtle", "1.0.0", AlgoStatus::Stopped).with_description("Turtle breakout");
    assert!(!view.upsert(first));
    assert_eq!(view.algos.len(), 1);

    let updated = AlgoRow::new("turtle", "1.1.0", AlgoStatus::Active);
    assert!(view.upsert(updated));
    assert_eq!(view.algos.len(), 1);
    assert_eq!(view.algos[0].version, "1.1.0");
    assert!(view.algos[0].status.is_live());
}

#[test]
fn set_status_updates_named_algo() {
    let mut view = AlgoManagerViewModel::empty();
    view.upsert(AlgoRow::new("twap", "2.0.0", AlgoStatus::Paused));
    assert!(view.set_status("twap", AlgoStatus::Active));
    assert!(!view.set_status("missing", AlgoStatus::Active));
    assert_eq!(view.live_count(), 1);
}

#[test]
fn status_live_semantics() {
    assert!(AlgoStatus::Active.is_live());
    assert!(!AlgoStatus::Paused.is_live());
    assert!(!AlgoStatus::Stopped.is_live());
    assert!(!AlgoStatus::Error.is_live());
}
