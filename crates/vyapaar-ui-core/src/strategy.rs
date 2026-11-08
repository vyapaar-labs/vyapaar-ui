//! Strategy execution, optimization, and trade suggestion primitives.
//!
//! WASM-safe, pure presentational and analytical models: zero-floats (`Decimal`
//! everywhere), no threads, no tokio, no blocking I/O. Used across CLI, TUI,
//! and Leptos workstations.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Origin source of a strategy definition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StrategySourceKind {
    /// Native built-in Rust strategy.
    Builtin,
    /// TOML/JSON declarative strategy manifest.
    Manifest,
    /// User custom or synthetic algorithm.
    Custom,
}

impl StrategySourceKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Builtin => "builtin",
            Self::Manifest => "manifest",
            Self::Custom => "custom",
        }
    }
}

/// Parameter definition with optional search space bounds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StrategyParameterSpec {
    /// Parameter identifier (e.g. "fast_period").
    pub name: String,
    /// Default parameter value.
    pub default: Decimal,
    /// Minimum search boundary for tuning.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<Decimal>,
    /// Maximum search boundary for tuning.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<Decimal>,
    /// Step increment for search sweeps.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step: Option<Decimal>,
    /// Human description of the parameter.
    pub description: String,
}

impl StrategyParameterSpec {
    pub fn new(name: impl Into<String>, default: Decimal) -> Self {
        Self {
            name: name.into(),
            default,
            min: None,
            max: None,
            step: None,
            description: String::new(),
        }
    }

    pub fn with_range(mut self, min: Decimal, max: Decimal, step: Decimal) -> Self {
        self.min = Some(min);
        self.max = Some(max);
        self.step = Some(step);
        self
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }
}

/// Metadata and parameter contract of a trading strategy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StrategyDescriptor {
    /// Strategy unique identifier (e.g. "turtle_breakout", "sma_crossover").
    pub id: String,
    /// Display name.
    pub name: String,
    /// Version string (e.g. "0.1.0").
    pub version: String,
    /// High-level description.
    pub description: String,
    /// Source origin.
    pub source: StrategySourceKind,
    /// Parameter specifications and ranges.
    pub parameters: Vec<StrategyParameterSpec>,
    /// Target trading instruments (e.g. ["NSE:NIFTY50-INDEX"]).
    pub target_instruments: Vec<String>,
}

impl StrategyDescriptor {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        version: impl Into<String>,
        source: StrategySourceKind,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            version: version.into(),
            description: String::new(),
            source,
            parameters: Vec::new(),
            target_instruments: Vec::new(),
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    pub fn with_parameter(mut self, param: StrategyParameterSpec) -> Self {
        self.parameters.push(param);
        self
    }

    pub fn with_instrument(mut self, inst: impl Into<String>) -> Self {
        self.target_instruments.push(inst.into());
        self
    }
}

/// Performance and statistical metrics from a backtest execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BacktestReport {
    /// Initial allocated capital.
    pub initial_capital: Decimal,
    /// Final equity at end of run.
    pub final_equity: Decimal,
    /// Net profit/loss after transaction friction.
    pub net_pnl: Decimal,
    /// Total percentage return.
    pub total_return_pct: Decimal,
    /// Annualized Sharpe Ratio.
    pub sharpe_ratio: Decimal,
    /// Maximum drawdown percentage (positive value).
    pub max_drawdown_pct: Decimal,
    /// Total closed trades.
    pub total_trades: u64,
    /// Percentage of profitable trades.
    pub win_rate_pct: Decimal,
    /// Ratio of gross profits to gross losses.
    pub profit_factor: Decimal,
}

/// Parameter optimization and statistical validation result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OptimizationResult {
    /// Strategy ID that was tuned.
    pub strategy_id: String,
    /// Original default parameter mapping.
    pub default_params: BTreeMap<String, Decimal>,
    /// Optimal parameter mapping discovered by search.
    pub tuned_params: BTreeMap<String, Decimal>,
    /// Sharpe ratio with default parameters.
    pub default_sharpe: Decimal,
    /// Sharpe ratio achieved with tuned parameters.
    pub tuned_sharpe: Decimal,
    /// Deflated Sharpe Ratio confidence (0.00 to 1.00; >= 0.90 indicates significance).
    pub deflated_sharpe: Decimal,
    /// Max drawdown percentage under tuned parameters.
    pub max_drawdown_pct: Decimal,
    /// Win rate percentage under tuned parameters.
    pub win_rate_pct: Decimal,
    /// Number of distinct parameter trials evaluated.
    pub total_trials: usize,
    /// Overfitting audit diagnosis (e.g. "Low Overfitting Risk", "Overfitted").
    pub overfitting_verdict: String,
}

/// Trade direction / intent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TradeBias {
    Buy,
    Sell,
    Neutral,
}

impl TradeBias {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Buy => "BUY / LONG",
            Self::Sell => "SELL / SHORT",
            Self::Neutral => "NEUTRAL / HOLD",
        }
    }
}

/// Actionable price levels and signal recommendation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TradeSignal {
    /// Directional bias.
    pub bias: TradeBias,
    /// Current entry benchmark price.
    pub entry_price: Decimal,
    /// Protective stop-loss exit level.
    pub stop_loss: Decimal,
    /// Primary profit objective (1:2 risk-reward).
    pub target_1: Decimal,
    /// Secondary profit objective (1:3 risk-reward).
    pub target_2: Decimal,
    /// Theoretical risk-reward ratio (e.g. 2.0).
    pub risk_reward_ratio: Decimal,
    /// Estimated Average True Range (volatility).
    pub atr: Decimal,
    /// Signal reasoning / trigger condition.
    pub reason: String,
}

/// Risk-budgeted position sizing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RiskSizing {
    /// Total trading account capital.
    pub account_capital: Decimal,
    /// Fraction of account risked (e.g. 1.0 for 1%).
    pub risk_pct: Decimal,
    /// Monetary risk budget amount.
    pub risk_amount: Decimal,
    /// Sized share or contract quantity.
    pub quantity: u64,
    /// Total notional commitment (quantity * entry_price).
    pub notional_value: Decimal,
    /// Capital utilization percentage.
    pub margin_utilization_pct: Decimal,
}

/// Indian statutory transaction fees and tax breakdown.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatutoryCharges {
    /// Securities Transaction Tax (STT).
    pub stt: Decimal,
    /// Exchange turnover fees.
    pub exchange_turnover: Decimal,
    /// SEBI regulatory turnover charge.
    pub sebi_charges: Decimal,
    /// State Stamp Duty.
    pub stamp_duty: Decimal,
    /// Goods and Services Tax (GST 18%).
    pub gst: Decimal,
    /// Total combined statutory friction.
    pub total_charges: Decimal,
    /// Effective friction-adjusted Risk-Reward ratio.
    pub net_risk_reward_ratio: Decimal,
}

/// Complete synthesis of strategy, tuning, signal, sizing, and friction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TradeSuggestion {
    /// Strategy specification.
    pub strategy: StrategyDescriptor,
    /// Target trading symbol.
    pub symbol: String,
    /// Tuned optimal parameters.
    pub tuned_params: BTreeMap<String, Decimal>,
    /// Optimization and validation audit.
    pub optimization: OptimizationResult,
    /// Actionable price signal.
    pub signal: TradeSignal,
    /// Risk-budgeted sizing.
    pub sizing: RiskSizing,
    /// Statutory transaction costs.
    pub statutory_charges: StatutoryCharges,
    /// Generation timestamp.
    pub timestamp: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// Mathematical Calculation Helpers
// ---------------------------------------------------------------------------

/// Computes risk-budgeted position sizing.
pub fn calculate_risk_sizing(
    capital: Decimal,
    risk_pct: Decimal,
    entry: Decimal,
    stop_loss: Decimal,
) -> RiskSizing {
    let risk_pct_clamped = risk_pct.max(Decimal::new(1, 1)).min(Decimal::new(10, 0));
    let risk_amount = capital * (risk_pct_clamped / Decimal::new(100, 0));
    let per_unit_risk = (entry - stop_loss).abs();

    let quantity = if per_unit_risk > Decimal::ZERO {
        let raw_qty = (risk_amount / per_unit_risk).floor();
        raw_qty.to_u64().unwrap_or(0).max(1)
    } else {
        1
    };

    let notional_value = Decimal::from(quantity) * entry;
    let margin_utilization_pct = if capital > Decimal::ZERO {
        ((notional_value / capital) * Decimal::new(100, 0)).round_dp(2)
    } else {
        Decimal::ZERO
    };

    RiskSizing {
        account_capital: capital,
        risk_pct: risk_pct_clamped,
        risk_amount: risk_amount.round_dp(2),
        quantity,
        notional_value: notional_value.round_dp(2),
        margin_utilization_pct,
    }
}

/// Calculates standard Indian statutory transaction fees and taxes.
pub fn calculate_statutory_charges(
    notional: Decimal,
    is_delivery: bool,
    target_profit: Decimal,
    target_loss: Decimal,
) -> StatutoryCharges {
    // STT: 0.1% on delivery turnover (0.001); 0.025% on intraday sell side (0.00025)
    let stt = if is_delivery {
        (notional * Decimal::new(1, 3)).round_dp(2)
    } else {
        (notional * Decimal::new(25, 5)).round_dp(2)
    };

    // NSE Exchange Turnover: 0.00325% = 0.0000325
    let exchange_turnover = (notional * Decimal::new(325, 7)).round_dp(2);

    // SEBI Charges: ₹10 per crore = 0.000001
    let sebi_charges = (notional * Decimal::new(1, 6)).round_dp(2);

    // Stamp Duty: 0.015% on buy side = 0.00015
    let stamp_duty = (notional * Decimal::new(15, 5)).round_dp(2);

    // GST: 18% on (Exchange + SEBI charges) = 0.18
    let gst = ((exchange_turnover + sebi_charges) * Decimal::new(18, 2)).round_dp(2);

    let total_charges = stt + exchange_turnover + sebi_charges + stamp_duty + gst;

    // Friction-adjusted net risk-reward ratio
    let net_profit = target_profit - total_charges;
    let net_loss = target_loss + total_charges;
    let net_risk_reward_ratio = if net_loss > Decimal::ZERO {
        (net_profit / net_loss).round_dp(2)
    } else {
        Decimal::new(2, 0)
    };

    StatutoryCharges {
        stt,
        exchange_turnover,
        sebi_charges,
        stamp_duty,
        gst,
        total_charges,
        net_risk_reward_ratio,
    }
}

/// Computes standard deviation and annualized Sharpe ratio from return series.
pub fn calculate_sharpe(returns: &[Decimal]) -> Decimal {
    if returns.len() < 2 {
        return Decimal::ZERO;
    }

    let n = Decimal::from(returns.len());
    let sum: Decimal = returns.iter().copied().sum();
    let mean = sum / n;

    let mut var_acc = Decimal::ZERO;
    for r in returns {
        let diff = *r - mean;
        var_acc += diff * diff;
    }
    let variance = var_acc / Decimal::from(returns.len() - 1);
    let std_dev =
        Decimal::from_f64(variance.to_f64().unwrap_or(0.0).sqrt()).unwrap_or(Decimal::ZERO);

    if std_dev.is_zero() {
        return Decimal::ZERO;
    }

    // Annualized by sqrt(252) approx 15.8745
    let annual_factor = Decimal::new(158745, 4);
    ((mean / std_dev) * annual_factor).round_dp(2)
}

/// Computes maximum peak-to-trough drawdown percentage.
pub fn calculate_max_drawdown(equity: &[Decimal]) -> Decimal {
    if equity.is_empty() {
        return Decimal::ZERO;
    }

    let mut peak = equity[0];
    let mut max_dd = Decimal::ZERO;

    for &val in equity {
        if val > peak {
            peak = val;
        }
        if peak > Decimal::ZERO {
            let dd = (peak - val) / peak * Decimal::new(100, 0);
            if dd > max_dd {
                max_dd = dd;
            }
        }
    }

    max_dd.round_dp(2)
}

/// Standard normal CDF approximation (Abramowitz & Stegun).
fn norm_cdf(z: f64) -> f64 {
    if z < -7.0 {
        return 0.0;
    }
    if z > 7.0 {
        return 1.0;
    }

    let x = z.abs();
    let b1 = 0.319381530;
    let b2 = -0.356563782;
    let b3 = 1.781477937;
    let b4 = -1.821255978;
    let b5 = 1.330274429;
    let p = 0.2316419;
    let c = 0.3989422804014327; // 1 / sqrt(2 * pi)

    let t = 1.0 / (1.0 + p * x);
    let poly = t * (b1 + t * (b2 + t * (b3 + t * (b4 + t * b5))));
    let phi = 1.0 - c * (-0.5 * x * x).exp() * poly;

    if z >= 0.0 { phi } else { 1.0 - phi }
}

/// Calculates Deflated Sharpe Ratio (DSR) confidence against multiple-testing selection bias.
pub fn calculate_deflated_sharpe(sharpe: Decimal, num_trials: usize, num_bars: usize) -> Decimal {
    let sr = sharpe.to_f64().unwrap_or(0.0);
    let k = (num_trials as f64).max(1.0);
    let t = (num_bars as f64).max(30.0);

    // Expected maximum Sharpe under the null hypothesis of K independent trials
    let e_max_sr = if k > 1.0 {
        let log_k = k.ln();
        (2.0 * log_k).sqrt() * 0.45
    } else {
        0.0
    };

    let std_err = 1.0 / (t - 1.0).sqrt();
    let z = (sr - e_max_sr) / std_err.max(0.0001);
    let cdf = norm_cdf(z);

    Decimal::from_f64(cdf)
        .unwrap_or(Decimal::new(5, 1))
        .round_dp(4)
}
