//! Re-export of StrategyBridge and MarketBar from vyapaar-ui-core,
//! and bridge runner implementing StrategyRunner for agent tool calling.

use async_trait::async_trait;
use rust_decimal::Decimal;
use serde_json::Value;
use std::collections::BTreeMap;
use vyapaar_agent_chat_tools::ports::strategy::StrategyRunner;
pub use vyapaar_ui_core::strategy::{MarketBar, StrategyBridge, TradeSuggestion};

/// Strategy runner bridging agent chat tools to StrategyBridge execution engine.
#[derive(Debug, Default, Clone)]
pub struct StrategyBridgeRunner;

#[async_trait]
impl StrategyRunner for StrategyBridgeRunner {
    async fn run_backtest(
        &self,
        strategy: &str,
        symbol: &str,
        capital: Decimal,
        params: Option<BTreeMap<String, Decimal>>,
    ) -> Result<Value, String> {
        let report = StrategyBridge::run_backtest(strategy, symbol, params.as_ref(), capital)?;
        serde_json::to_value(&report).map_err(|e| e.to_string())
    }

    async fn tune_strategy(
        &self,
        strategy: &str,
        symbol: &str,
        capital: Decimal,
        metric: &str,
    ) -> Result<Value, String> {
        let opt = StrategyBridge::tune_strategy(strategy, symbol, capital, metric)?;
        serde_json::to_value(&opt).map_err(|e| e.to_string())
    }

    async fn generate_trade_suggestion(
        &self,
        strategy: &str,
        symbol: &str,
        capital: Decimal,
        risk_pct: Decimal,
    ) -> Result<Value, String> {
        let sugg = StrategyBridge::generate_trade_suggestion(strategy, symbol, capital, risk_pct)?;
        serde_json::to_value(&sugg).map_err(|e| e.to_string())
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
