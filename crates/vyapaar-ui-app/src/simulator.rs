//! Simulator feature: backtest run state for the workbench.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// The phase of a simulator run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SimulatorPhase {
    /// No run configured yet.
    Idle,
    /// A run is in progress.
    Running,
    /// The run finished with results.
    Complete,
    /// The run finished with an error.
    Failed,
}

/// A single equity-curve point from a backtest run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct EquityCurvePoint {
    /// Bar index.
    pub index: u32,
    /// Portfolio equity at that bar.
    pub equity: Decimal,
}

/// The simulator feature state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SimulatorFeature {
    /// Current phase.
    pub phase: SimulatorPhase,
    /// Equity curve from the latest run.
    pub equity_curve: Vec<EquityCurvePoint>,
    /// Final total return (fraction, e.g. `0.12` for 12%).
    pub total_return: Decimal,
    /// Final max drawdown (positive fraction).
    pub max_drawdown: Decimal,
    /// Final Sharpe ratio.
    pub sharpe: Decimal,
}

impl SimulatorFeature {
    /// Create a new, idle simulator feature.
    pub fn new() -> Self {
        Self {
            phase: SimulatorPhase::Idle,
            equity_curve: Vec::new(),
            total_return: Decimal::ZERO,
            max_drawdown: Decimal::ZERO,
            sharpe: Decimal::ZERO,
        }
    }

    /// Start a run.
    pub fn start(&mut self) {
        self.phase = SimulatorPhase::Running;
        self.equity_curve.clear();
    }

    /// Append an equity point to the running curve.
    pub fn push_equity(&mut self, point: EquityCurvePoint) {
        if self.phase == SimulatorPhase::Running {
            self.equity_curve.push(point);
        }
    }

    /// Complete the run with summary metrics.
    pub fn complete(&mut self, total_return: Decimal, max_drawdown: Decimal, sharpe: Decimal) {
        self.total_return = total_return;
        self.max_drawdown = max_drawdown;
        self.sharpe = sharpe;
        self.phase = SimulatorPhase::Complete;
    }

    /// Fail the run with an error.
    pub fn fail(&mut self) {
        self.phase = SimulatorPhase::Failed;
    }

    /// Whether the last run produced a (possibly empty) curve.
    pub fn has_results(&self) -> bool {
        self.phase == SimulatorPhase::Complete
    }
}

impl Default for SimulatorFeature {
    fn default() -> Self {
        Self::new()
    }
}
