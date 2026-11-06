//! Tests for the system feature.

use vyapaar_ui_app::system::{SubsystemState, SystemFeature};

#[test]
fn default_system_is_all_healthy() {
    let s = SystemFeature::default_system();
    assert_eq!(s.subsystems.len(), 5);
    assert!(s.all_healthy());
}

#[test]
fn set_subsystem_state() {
    let mut s = SystemFeature::default_system();
    assert!(s.set_subsystem("market", SubsystemState::Degraded, "feed reconnecting"));
    assert!(!s.all_healthy());
    let market = s.subsystems.iter().find(|r| r.name == "market").unwrap();
    assert_eq!(market.state, SubsystemState::Degraded);
    assert_eq!(market.detail, "feed reconnecting");
}

#[test]
fn set_unknown_subsystem_returns_false() {
    let mut s = SystemFeature::default_system();
    assert!(!s.set_subsystem("nope", SubsystemState::Down, "x"));
}

#[test]
fn exchanges_include_indian_venues() {
    let s = SystemFeature::default_system();
    assert_eq!(s.exchanges.len(), 3);
}

#[test]
fn subsystem_state_ok_helpers() {
    assert!(SubsystemState::Healthy.is_ok());
    assert!(!SubsystemState::Degraded.is_ok());
    assert!(!SubsystemState::Down.is_ok());
}
