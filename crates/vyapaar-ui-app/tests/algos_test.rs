//! Tests for the algo management feature.

use vyapaar_ui_app::algos::AlgosFeature;
use vyapaar_ui_core::algos::{AlgoRow, AlgoStatus};

#[test]
fn new_feature_is_empty() {
    let f = AlgosFeature::new();
    assert!(f.view.is_empty());
    assert_eq!(f.selected, None);
}

#[test]
fn upsert_and_select() {
    let mut f = AlgosFeature::new();
    f.upsert(AlgoRow::new("turtle", "1.0.0", AlgoStatus::Stopped));
    f.upsert(AlgoRow::new("twap", "2.0.0", AlgoStatus::Paused));
    f.select(1);
    assert_eq!(f.selected_row().unwrap().name, "twap");
    // Selection clamps to bounds.
    f.select(99);
    assert_eq!(f.selected_row().unwrap().name, "twap");
}

#[test]
fn set_status_and_live_count() {
    let mut f = AlgosFeature::new();
    f.upsert(AlgoRow::new("turtle", "1.0.0", AlgoStatus::Stopped));
    assert!(f.set_status("turtle", AlgoStatus::Active));
    assert!(!f.set_status("missing", AlgoStatus::Active));
    assert_eq!(f.live_count(), 1);
}

#[test]
fn select_on_empty_list_clears_selection() {
    let mut f = AlgosFeature::new();
    f.select(0);
    assert_eq!(f.selected, None);
}
