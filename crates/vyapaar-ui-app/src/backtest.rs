//! Backtest results feature: reactive state for the results viewer.
//!
//! A thin view over `vyapaar-runtime-backtest`'s `BacktestResult` (per
//! `docs/plans/0005-fincept-terminal-screen-inventory.md` §2 item 3). The host
//! (TUI/CLI/WASM) projects a finished run's summary and equity curve into this
//! feature; the feature owns lifecycle state and folds incoming equity points.
//! WASM-safe: no I/O, no tokio, all math exact `Decimal`.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use vyapaar_ui_core::atoms::EquityPoint;
use vyapaar_ui_core::backtest::{BacktestResultView, BacktestStatus, BacktestSummary};

/// The backtest results feature state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BacktestFeature {
    /// The presentational results view.
    pub view: BacktestResultView,
}

impl BacktestFeature {
    /// Create an empty backtest feature.
    pub fn new() -> Self {
        Self {
            view: BacktestResultView::empty(),
        }
    }

    /// Mark a run as started, clearing any previous results.
    pub fn start_run(&mut self) {
        self.view = BacktestResultView {
            status: BacktestStatus::Running,
            summary: None,
            equity_curve: Vec::new(),
        };
    }

    /// Append one equity-curve point to the running results.
    pub fn push_equity(&mut self, point: EquityPoint) {
        if self.view.status == BacktestStatus::Running {
            self.view.equity_curve.push(point);
        }
    }

    /// Complete the run with its summary metrics.
    pub fn complete_run(&mut self, summary: BacktestSummary) {
        self.view.status = BacktestStatus::Complete;
        self.view.summary = Some(summary);
    }

    /// Fail the run.
    pub fn fail_run(&mut self) {
        self.view.status = BacktestStatus::Failed;
    }

    /// Whether a completed run's results are present.
    pub fn has_results(&self) -> bool {
        self.view.has_results()
    }

    /// The total-return percentage of the last completed run, if any.
    pub fn total_return_pct(&self) -> Decimal {
        self.view
            .summary
            .as_ref()
            .map(BacktestSummary::total_return_pct)
            .unwrap_or_default()
    }
}

impl Default for BacktestFeature {
    fn default() -> Self {
        Self::new()
    }
}
