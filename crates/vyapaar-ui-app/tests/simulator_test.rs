//! Tests for the simulator feature.

use rust_decimal::Decimal;
use vyapaar_ui_app::simulator::{EquityCurvePoint, SimulatorFeature, SimulatorPhase};

fn dec(s: &str) -> Decimal {
    s.parse().unwrap()
}

#[test]
fn new_simulator_is_idle() {
    let s = SimulatorFeature::new();
    assert_eq!(s.phase, SimulatorPhase::Idle);
    assert!(s.equity_curve.is_empty());
    assert!(!s.has_results());
}

#[test]
fn run_lifecycle_idle_to_complete() {
    let mut s = SimulatorFeature::new();
    s.start();
    assert_eq!(s.phase, SimulatorPhase::Running);
    s.push_equity(EquityCurvePoint { index: 0, equity: dec("100000") });
    s.push_equity(EquityCurvePoint { index: 1, equity: dec("102000") });
    s.complete(dec("0.02"), dec("0.05"), dec("1.2"));
    assert_eq!(s.phase, SimulatorPhase::Complete);
    assert!(s.has_results());
    assert_eq!(s.equity_curve.len(), 2);
    assert_eq!(s.total_return, dec("0.02"));
}

#[test]
fn push_equity_only_in_running_phase() {
    let mut s = SimulatorFeature::new();
    // Idle: pushes are ignored.
    s.push_equity(EquityCurvePoint { index: 0, equity: dec("1") });
    assert!(s.equity_curve.is_empty());

    s.start();
    s.push_equity(EquityCurvePoint { index: 0, equity: dec("1") });
    assert_eq!(s.equity_curve.len(), 1);
}

#[test]
fn failed_run_reports_failed() {
    let mut s = SimulatorFeature::new();
    s.start();
    s.fail();
    assert_eq!(s.phase, SimulatorPhase::Failed);
    assert!(!s.has_results());
}
