//! Re-export of StrategyBridge and MarketBar from vyapaar-ui-core,
//! and bridge runner implementing StrategyRunner for agent tool calling.
//!
//! Every tool call is journaled under the runtime journals directory as a
//! `kind: "tool"` run (PLAN-0001 Stage 4), best-effort: a journal failure
//! never breaks the tool result the agent sees. Touch journals follow the
//! same manifest conventions as job runs (`status` snapshots with wall-clock
//! timestamps; deterministic events), so `recover_interrupted_runs` normalizes
//! an interrupted agent turn on the next boot.

use async_trait::async_trait;
use rust_decimal::Decimal;
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::PathBuf;
use vyapaar_agent_chat_tools::ports::strategy::StrategyRunner;
pub use vyapaar_ui_core::strategy::{MarketBar, StrategyBridge, TradeSuggestion};

use crate::jobs_runtime::{error_detail_value, mint_run_id};
use vyapaar_runtime_api::{RunEvent, RunStatus};
use vyapaar_runtime_api_rest::FileJournalWriter;

/// Strategy runner bridging agent chat tools to StrategyBridge execution engine.
#[derive(Debug, Default, Clone)]
pub struct StrategyBridgeRunner {
    /// Overrides the journals root for isolation (tests); defaults to
    /// `~/.vyapaar/journals` (or `VYAPAAR_HOME/journals`).
    pub journal_home: Option<PathBuf>,
}

impl StrategyBridgeRunner {
    /// A runner journaling under the default Vyapaar home.
    pub fn new() -> Self {
        Self { journal_home: None }
    }

    /// A runner journaling under an explicit home (tests).
    pub fn with_journal_home(dir: PathBuf) -> Self {
        Self {
            journal_home: Some(dir),
        }
    }

    /// Where tool-run journals live.
    fn journals_dir(&self) -> PathBuf {
        self.journal_home
            .clone()
            .unwrap_or_else(crate::runner::vyapaar_home_dir)
            .join("journals")
    }

    /// Runs one engine call and journals it as a `kind: "tool"` run.
    fn journaled(
        &self,
        tool: &str,
        input: Value,
        engine: impl FnOnce() -> Result<Value, String>,
    ) -> Result<Value, String> {
        let started_at = chrono::Utc::now();
        let outcome = engine();
        self.journal_tool_run(tool, &input, &outcome, started_at);
        outcome
    }

    /// Best-effort journal write for one tool call (never surfaces errors).
    fn journal_tool_run(
        &self,
        tool: &str,
        input: &Value,
        outcome: &Result<Value, String>,
        started_at: chrono::DateTime<chrono::Utc>,
    ) {
        let Ok(run_id) = mint_run_id() else {
            return;
        };
        let running = match RunStatus::Pending.next(RunEvent::Start) {
            Ok(status) => status,
            Err(_) => return,
        };
        let running_manifest = serde_json::json!({
            "manifest_version": 1,
            "kind": "tool",
            "run_id": run_id.as_str(),
            "tool": tool,
            "input": input,
            "status": running,
            "created_at": started_at.to_rfc3339(),
            "started_at": started_at.to_rfc3339(),
        });
        let Ok(mut writer) =
            FileJournalWriter::create_from_value(&self.journals_dir(), &run_id, &running_manifest)
        else {
            return;
        };
        let _ = writer.append_event(&serde_json::json!({
            "seq": 1,
            "type": "tool.started",
            "tool": tool,
            "input": input,
        }));

        let (status, result_value, error) = match outcome {
            Ok(value) => match running.next(RunEvent::Complete) {
                Ok(status) => (status, Some(value.clone()), None),
                Err(_) => return,
            },
            Err(err) => match running.next(RunEvent::Fail) {
                Ok(status) => (status, None, Some(error_detail_value(err))),
                Err(_) => return,
            },
        };

        let result_event = match outcome {
            Ok(_) => serde_json::json!({
                "seq": 2,
                "type": "tool.result",
                "tool": tool,
            }),
            Err(_) => serde_json::json!({
                "seq": 2,
                "type": "tool.failed",
                "tool": tool,
            }),
        };
        let _ = writer.append_event(&result_event);

        let mut final_manifest = running_manifest.clone();
        final_manifest["status"] = serde_json::json!(status);
        final_manifest["finished_at"] = serde_json::json!(chrono::Utc::now().to_rfc3339());
        if let Some(err) = error {
            final_manifest["error"] = err;
        }
        // Tool runs carry no tax-lot data: `trades.json` stays an empty object
        // and the engine result lands in `metrics.json`.
        let metrics = result_value.unwrap_or_else(|| serde_json::json!({}));
        let _ = writer.finalize_value(&final_manifest, &serde_json::json!({}), &metrics);
    }
}

#[async_trait]
impl StrategyRunner for StrategyBridgeRunner {
    async fn run_backtest(
        &self,
        strategy: &str,
        symbol: &str,
        capital: Decimal,
        params: Option<BTreeMap<String, Decimal>>,
    ) -> Result<Value, String> {
        self.journaled(
            "run_backtest",
            serde_json::json!({
                "strategy": strategy,
                "symbol": symbol,
                "capital": capital.to_string(),
                "params": params,
            }),
            || {
                let report =
                    StrategyBridge::run_backtest(strategy, symbol, params.as_ref(), capital)?;
                serde_json::to_value(&report).map_err(|e| e.to_string())
            },
        )
    }

    async fn tune_strategy(
        &self,
        strategy: &str,
        symbol: &str,
        capital: Decimal,
        metric: &str,
    ) -> Result<Value, String> {
        self.journaled(
            "tune_strategy",
            serde_json::json!({
                "strategy": strategy,
                "symbol": symbol,
                "capital": capital.to_string(),
                "metric": metric,
            }),
            || {
                let opt = StrategyBridge::tune_strategy(strategy, symbol, capital, metric)?;
                serde_json::to_value(&opt).map_err(|e| e.to_string())
            },
        )
    }

    async fn generate_trade_suggestion(
        &self,
        strategy: &str,
        symbol: &str,
        capital: Decimal,
        risk_pct: Decimal,
    ) -> Result<Value, String> {
        self.journaled(
            "generate_trade_suggestion",
            serde_json::json!({
                "strategy": strategy,
                "symbol": symbol,
                "capital": capital.to_string(),
                "risk_pct": risk_pct.to_string(),
            }),
            || {
                let sugg =
                    StrategyBridge::generate_trade_suggestion(strategy, symbol, capital, risk_pct)?;
                serde_json::to_value(&sugg).map_err(|e| e.to_string())
            },
        )
    }
}

/// Renders a standardized terminal card for a trade suggestion.
pub fn render_suggestion_card(sugg: &TradeSuggestion) -> String {
    let tuned_summary = sugg
        .tuned_params
        .iter()
        .map(|(k, v)| {
            let def_val = sugg
                .optimization
                .default_params
                .get(k)
                .copied()
                .unwrap_or(*v);
            format!("{k}={v} (def: {def_val})")
        })
        .collect::<Vec<_>>()
        .join(", ");

    let dsr_pct = (sugg.optimization.deflated_sharpe * Decimal::new(100, 0)).round_dp(2);
    let risk_per_unit = (sugg.signal.entry_price - sugg.signal.stop_loss).abs();
    let risk_pct_trade = if sugg.signal.entry_price > Decimal::ZERO {
        (risk_per_unit / sugg.signal.entry_price * Decimal::new(100, 0)).round_dp(2)
    } else {
        Decimal::ZERO
    };

    let target_1_diff = (sugg.signal.target_1 - sugg.signal.entry_price).abs();
    let target_1_pct = if sugg.signal.entry_price > Decimal::ZERO {
        (target_1_diff / sugg.signal.entry_price * Decimal::new(100, 0)).round_dp(2)
    } else {
        Decimal::ZERO
    };

    let target_2_diff = (sugg.signal.target_2 - sugg.signal.entry_price).abs();
    let target_2_pct = if sugg.signal.entry_price > Decimal::ZERO {
        (target_2_diff / sugg.signal.entry_price * Decimal::new(100, 0)).round_dp(2)
    } else {
        Decimal::ZERO
    };

    format!(
        "╔═══════════════════════════════════════════════════════════════════════════╗
║          VYAPAAR OPTIMAL TRADE SUGGESTION: {:<31}║
╚═══════════════════════════════════════════════════════════════════════════╝

1. PARAMETER TUNING & STATISTICAL AUDIT
   ────────────────────────────────────────────────────────────────────────
   • Tuned Parameters : {}
   • Performance      : Sharpe {:.2} (vs {:.2} def) | Max DD: {:.2}% | Win Rate: {:.2}%
   • Deflated Sharpe  : {}% confidence ({})

2. OPTIMAL TRADE EXECUTION CARD
   ────────────────────────────────────────────────────────────────────────
   • Target Asset     : {}
   • Direction / Bias : {}
   • Trigger / Reason : {}
   • Entry Level      : ₹{:.2}
   • Stop Loss        : ₹{:.2} (-₹{:.2} / -{:.2}% / 2.0 ATR)
   • Target 1 (1:2 RR): ₹{:.2} (+₹{:.2} / +{:.2}%)
   • Target 2 (1:3 RR): ₹{:.2} (+₹{:.2} / +{:.2}%)

3. CAPITAL ALLOCATION & RISK BUDGETING
   ────────────────────────────────────────────────────────────────────────
   • Available Capital: ₹{:.2}
   • Risk Budget ({:.1}%): ₹{:.2}
   • Recommended Size : {} units (Notional: ₹{:.2})
   • Margin Commitment: {:.2}% of capital
   • Est. Friction Fee: ₹{:.2} (STT, GST, SEBI Turnover, Stamp Duty)
   • Net Risk-Reward  : 1 : {:.2} (friction-adjusted)
",
        sugg.strategy.name.to_uppercase(),
        tuned_summary,
        sugg.optimization.tuned_sharpe,
        sugg.optimization.default_sharpe,
        sugg.optimization.max_drawdown_pct,
        sugg.optimization.win_rate_pct,
        dsr_pct,
        sugg.optimization.overfitting_verdict,
        sugg.symbol,
        sugg.signal.bias.as_str(),
        sugg.signal.reason,
        sugg.signal.entry_price,
        sugg.signal.stop_loss,
        risk_per_unit,
        risk_pct_trade,
        sugg.signal.target_1,
        target_1_diff,
        target_1_pct,
        sugg.signal.target_2,
        target_2_diff,
        target_2_pct,
        sugg.sizing.account_capital,
        sugg.sizing.risk_pct,
        sugg.sizing.risk_amount,
        sugg.sizing.quantity,
        sugg.sizing.notional_value,
        sugg.sizing.margin_utilization_pct,
        sugg.statutory_charges.total_charges,
        sugg.statutory_charges.net_risk_reward_ratio,
    )
}
