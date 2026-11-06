//! Tests for the workstation router.

use vyapaar_ui_app::router::{AppRoute, Router, RouterError};

#[test]
fn default_router_starts_at_home() {
    let r = Router::default_router();
    assert_eq!(r.active, AppRoute::Home);
    assert!(r.previous.is_none());
}

#[test]
fn navigate_records_previous() {
    let mut r = Router::new(AppRoute::Home);
    r.navigate(AppRoute::Screener).unwrap();
    assert_eq!(r.active, AppRoute::Screener);
    assert_eq!(r.previous, Some(AppRoute::Home));
}

#[test]
fn navigate_to_same_route_errors() {
    let mut r = Router::new(AppRoute::Studio);
    assert_eq!(
        r.navigate(AppRoute::Studio),
        Err(RouterError::AlreadyActive)
    );
    assert_eq!(r.active, AppRoute::Studio);
}

#[test]
fn back_returns_to_previous() {
    let mut r = Router::new(AppRoute::Home);
    r.navigate(AppRoute::System).unwrap();
    r.back();
    assert_eq!(r.active, AppRoute::Home);
    assert!(r.previous.is_none());
}

#[test]
fn back_without_history_stays_put() {
    let mut r = Router::new(AppRoute::Home);
    r.back();
    assert_eq!(r.active, AppRoute::Home);
}

#[test]
fn route_labels_and_paths() {
    assert_eq!(AppRoute::Screener.label(), "Screener");
    assert_eq!(AppRoute::Screener.path(), "/screener");
    assert_eq!(AppRoute::Home.path(), "/");
    assert_eq!(AppRoute::ALL.len(), 5);
}

#[test]
fn route_serde_roundtrip() {
    let route = AppRoute::Studio;
    let json = serde_json::to_string(&route).unwrap();
    let back: AppRoute = serde_json::from_str(&json).unwrap();
    assert_eq!(back, route);
}
