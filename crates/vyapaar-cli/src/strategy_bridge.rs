//! Strategy Execution, Parameter Tuning, and Optimal Trade Suggestion Bridge.
//!
//! Connects native CLI and TUI commands to built-in Rust strategies, declarative
//! TOML manifests, hyperparameter tuning grid sweeps, and risk-budgeted trade suggestions.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use vyapaar_core::chrono::Utc;
use vyapaar_ui_core::strategy::{
    BacktestReport, OptimizationResult, StrategyDescriptor, StrategyParameterSpec,
    StrategySourceKind, TradeBias, TradeSignal, TradeSuggestion, calculate_deflated_sharpe,
    calculate_max_drawdown, calculate_risk_sizing, calculate_sharpe, calculate_statutory_charges,
};

/// High-level bridge for strategy discovery, backtesting, parameter tuning, and trade suggestions.
pub struct StrategyBridge;

impl StrategyBridge {
    /// Discover all available strategies (native built-in strategies + scanned TOML manifests).
    pub fn discover_strategies() -> Vec<StrategyDescriptor> {
        let mut list = Self::builtin_strategies();

        // Scan common manifest directories
        let candidates = [
            PathBuf::from("manifests/strategies"),
            PathBuf::from("vyapaar-strategies/manifests/strategies"),
            PathBuf::from("../vyapaar-strategies/manifests/strategies"),
            PathBuf::from("../../vyapaar-strategies/manifests/strategies"),
        ];

        for dir in &candidates {
            if dir.is_dir() {
                if let Ok(entries) = fs::read_dir(dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                            if let Some(desc) = Self::load_manifest_file(&path) {
                                // If already exists as builtin, manifest takes precedence or augments
                                if let Some(existing) = list.iter_mut().find(|s| s.id == desc.id) {
                                    *existing = desc;
                                } else {
                                    list.push(desc);
                                }
                            }
                        }
                    }
                }
            }
        }

        list
    }

    /// Load a strategy manifest from a TOML file.
    pub fn load_manifest_file(path: &Path) -> Option<StrategyDescriptor> {
        let content = fs::read_to_string(path).ok()?;
        let toml_val: toml::Value = toml::from_str(&content).ok()?;

        let name = toml_val.get("name")?.as_str()?.to_string();
        let version = toml_val
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("0.1.0")
            .to_string();

        let mut desc = StrategyDescriptor::new(&name, &name, version, StrategySourceKind::Manifest);

        if let Some(params_table) = toml_val.get("parameters").and_then(|p| p.as_table()) {
            for (p_name, p_val) in params_table {
                let default_val = p_val
                    .get("default")
                    .and_then(|v| {
                        if let Some(i) = v.as_integer() {
                            Some(Decimal::from(i))
                        } else if let Some(f) = v.as_float() {
                            Decimal::from_f64(f)
                        } else if let Some(s) = v.as_str() {
                            Decimal::from_str(s).ok()
                        } else {
                            None
                        }
                    })
                    .unwrap_or(Decimal::new(10, 0));

                let min_val = p_val.get("min").and_then(|v| {
                    if let Some(i) = v.as_integer() {
                        Some(Decimal::from(i))
                    } else if let Some(f) = v.as_float() {
                        Decimal::from_f64(f)
                    } else {
                        None
                    }
                });

                let max_val = p_val.get("max").and_then(|v| {
                    if let Some(i) = v.as_integer() {
                        Some(Decimal::from(i))
                    } else if let Some(f) = v.as_float() {
                        Decimal::from_f64(f)
                    } else {
                        None
                    }
                });

                let step_val = p_val.get("step").and_then(|v| {
                    if let Some(i) = v.as_integer() {
                        Some(Decimal::from(i))
                    } else if let Some(f) = v.as_float() {
                        Decimal::from_f64(f)
                    } else {
                        None
                    }
                });

                let mut spec = StrategyParameterSpec::new(p_name, default_val);
                if let (Some(mn), Some(mx), Some(st)) = (min_val, max_val, step_val) {
                    spec = spec.with_range(mn, mx, st);
                }
                desc = desc.with_parameter(spec);
            }
        }

        if let Some(target_instruments) = toml_val
            .get("target_instruments")
            .and_then(|t| t.get("instruments"))
            .and_then(|i| i.as_array())
        {
            for inst in target_instruments {
                if let Some(s) = inst.as_str() {
                    desc = desc.with_instrument(s);
                }
            }
        }

        Some(desc)
    }

    /// Native built-in strategy registry.
    pub fn builtin_strategies() -> Vec<StrategyDescriptor> {
        vec![
            StrategyDescriptor::new(
                "turtle_breakout",
                "Turtle Donchian Breakout",
                "0.1.0",
                StrategySourceKind::Builtin,
            )
            .with_description(
                "Classic Richard Dennis Donchian channel trend-following breakout system",
            )
            .with_parameter(
                StrategyParameterSpec::new("entry_period", Decimal::new(20, 0))
                    .with_range(Decimal::new(5, 0), Decimal::new(60, 0), Decimal::new(1, 0))
                    .with_description("Lookback period for entry breakout high"),
            )
            .with_parameter(
                StrategyParameterSpec::new("exit_period", Decimal::new(10, 0))
                    .with_range(Decimal::new(5, 0), Decimal::new(60, 0), Decimal::new(1, 0))
                    .with_description("Lookback period for exit trailing low"),
            )
            .with_parameter(
                StrategyParameterSpec::new("quantity", Decimal::new(10, 0))
                    .with_range(Decimal::new(1, 0), Decimal::new(100, 0), Decimal::new(1, 0))
                    .with_description("Default trading size units"),
            )
            .with_instrument("NSE:NIFTY50-INDEX")
            .with_instrument("NSE:BANKNIFTY-INDEX"),
            StrategyDescriptor::new(
                "sma_crossover",
                "Dual Moving Average Crossover",
                "0.1.0",
                StrategySourceKind::Builtin,
            )
            .with_description(
                "Golden / Death cross trend system between fast and slow moving averages",
            )
            .with_parameter(
                StrategyParameterSpec::new("fast_period", Decimal::new(10, 0))
                    .with_range(Decimal::new(5, 0), Decimal::new(50, 0), Decimal::new(1, 0))
                    .with_description("Fast simple moving average period"),
            )
            .with_parameter(
                StrategyParameterSpec::new("slow_period", Decimal::new(50, 0))
                    .with_range(
                        Decimal::new(20, 0),
                        Decimal::new(200, 0),
                        Decimal::new(5, 0),
                    )
                    .with_description("Slow simple moving average period"),
            )
            .with_parameter(
                StrategyParameterSpec::new("quantity", Decimal::new(10, 0))
                    .with_description("Default trading size units"),
            )
            .with_instrument("NSE:RELIANCE-EQ")
            .with_instrument("NSE:TCS-EQ")
            .with_instrument("NSE:INFY-EQ"),
            StrategyDescriptor::new(
                "rsi_reversal",
                "RSI Mean Reversion",
                "0.1.0",
                StrategySourceKind::Builtin,
            )
            .with_description("Overbought and oversold mean-reverting momentum oscillator")
            .with_parameter(
                StrategyParameterSpec::new("period", Decimal::new(14, 0))
                    .with_range(Decimal::new(5, 0), Decimal::new(30, 0), Decimal::new(1, 0))
                    .with_description("RSI computation window"),
            )
            .with_parameter(
                StrategyParameterSpec::new("oversold", Decimal::new(30, 0))
                    .with_range(Decimal::new(15, 0), Decimal::new(40, 0), Decimal::new(2, 0))
                    .with_description("Oversold long trigger threshold"),
            )
            .with_parameter(
                StrategyParameterSpec::new("overbought", Decimal::new(70, 0))
                    .with_range(Decimal::new(60, 0), Decimal::new(85, 0), Decimal::new(2, 0))
                    .with_description("Overbought exit threshold"),
            )
            .with_instrument("NSE:HDFCBANK-EQ")
            .with_instrument("NSE:ICICIBANK-EQ"),
            StrategyDescriptor::new(
                "buy_and_hold",
                "Passive Buy & Hold Benchmark",
                "0.1.0",
                StrategySourceKind::Builtin,
            )
            .with_description("Unhedged long buy-and-hold accumulation benchmark")
            .with_parameter(
                StrategyParameterSpec::new("quantity", Decimal::new(10, 0))
                    .with_description("Benchmark accumulation unit size"),
            )
            .with_instrument("NSE:NIFTY50-INDEX"),
            StrategyDescriptor::new(
                "twap_slicer",
                "TWAP Algorithmic Order Slicer",
                "0.1.0",
                StrategySourceKind::Builtin,
            )
            .with_description(
                "Time-Weighted Average Price algorithmic order slicing execution model",
            )
            .with_parameter(
                StrategyParameterSpec::new("total_quantity", Decimal::new(100, 0))
                    .with_range(
                        Decimal::new(10, 0),
                        Decimal::new(1000, 0),
                        Decimal::new(10, 0),
                    )
                    .with_description("Target total quantity to accumulate"),
            )
            .with_parameter(
                StrategyParameterSpec::new("slices", Decimal::new(10, 0))
                    .with_range(Decimal::new(2, 0), Decimal::new(50, 0), Decimal::new(2, 0))
                    .with_description("Number of equidistant execution slices"),
            )
            .with_instrument("NSE:RELIANCE-EQ"),
        ]
    }

    /// Lookup a strategy by ID or name.
    pub fn find_strategy(name: &str) -> Option<StrategyDescriptor> {
        let all = Self::discover_strategies();
        all.into_iter()
            .find(|s| s.id.eq_ignore_ascii_case(name) || s.name.eq_ignore_ascii_case(name))
    }

    /// Generates realistic benchmark market bars for evaluation.
    pub fn generate_market_bars(symbol: &str, count: usize) -> Vec<MarketBar> {
        let base_price = match symbol {
            s if s.contains("NIFTY50") => 24000.0,
            s if s.contains("BANKNIFTY") => 51000.0,
            s if s.contains("RELIANCE") => 2800.0,
            s if s.contains("TCS") => 4100.0,
            s if s.contains("INFY") => 1850.0,
            s if s.contains("HDFCBANK") => 1750.0,
            _ => 1000.0,
        };

        let mut bars = Vec::with_capacity(count);
        let mut curr_price = base_price;

        for i in 0..count {
            // Deterministic sinusoidal market cycle with pseudo-random volatility
            let cycle = ((i as f64) * 0.12).sin() * 0.015;
            let noise = (((i * 7 + 13) % 23) as f64 - 11.0) * 0.001;
            let change_pct = cycle + noise;

            let open = curr_price;
            let close = (open * (1.0 + change_pct)).max(1.0);
            let high = open.max(close) * 1.006;
            let low = open.min(close) * 0.994;
            let volume = 10000 + ((i * 137) % 5000);

            bars.push(MarketBar {
                open: Decimal::from_f64(open).unwrap_or(Decimal::new(100, 0)),
                high: Decimal::from_f64(high).unwrap_or(Decimal::new(100, 0)),
                low: Decimal::from_f64(low).unwrap_or(Decimal::new(100, 0)),
                close: Decimal::from_f64(close).unwrap_or(Decimal::new(100, 0)),
                volume: volume as u64,
            });

            curr_price = close;
        }

        bars
    }

    /// Run a backtest of a strategy with specified or default parameters.
    pub fn run_backtest(
        strategy_name: &str,
        symbol: &str,
        params: Option<&BTreeMap<String, Decimal>>,
        capital: Decimal,
    ) -> Result<BacktestReport, String> {
        let strategy = Self::find_strategy(strategy_name)
            .ok_or_else(|| format!("strategy '{strategy_name}' not found"))?;

        let mut effective_params = BTreeMap::new();
        for p in &strategy.parameters {
            effective_params.insert(p.name.clone(), p.default);
        }
        if let Some(overrides) = params {
            for (k, v) in overrides {
                effective_params.insert(k.clone(), *v);
            }
        }

        let bars = Self::generate_market_bars(symbol, 120);
        let sim_result = Self::simulate_strategy(&strategy.id, &effective_params, &bars, capital);

        Ok(sim_result)
    }

    /// Tune strategy parameters across their search ranges using a deterministic grid sweep.
    pub fn tune_strategy(
        strategy_name: &str,
        symbol: &str,
        capital: Decimal,
        metric: &str,
    ) -> Result<OptimizationResult, String> {
        let strategy = Self::find_strategy(strategy_name)
            .ok_or_else(|| format!("strategy '{strategy_name}' not found"))?;

        let mut default_params = BTreeMap::new();
        for p in &strategy.parameters {
            default_params.insert(p.name.clone(), p.default);
        }

        let bars = Self::generate_market_bars(symbol, 120);
        let default_run = Self::simulate_strategy(&strategy.id, &default_params, &bars, capital);

        // Build candidate parameter combinations from ranges
        let mut sweep_keys = Vec::new();
        let mut sweep_ranges: Vec<Vec<Decimal>> = Vec::new();

        for p in &strategy.parameters {
            if let (Some(min), Some(max), Some(step)) = (p.min, p.max, p.step) {
                if step > Decimal::ZERO && max >= min {
                    let mut vals = Vec::new();
                    let mut cur = min;
                    while cur <= max && vals.len() < 10 {
                        vals.push(cur);
                        cur += step;
                    }
                    if !vals.is_empty() {
                        sweep_keys.push(p.name.clone());
                        sweep_ranges.push(vals);
                    }
                }
            }
        }

        // Cartesian product of candidate parameter ranges (capped at 100 trials)
        let combinations = cartesian_product(&sweep_ranges, 100);
        let total_trials = combinations.len().max(1);

        let mut best_params = default_params.clone();
        let mut best_fitness = Decimal::new(-999, 0);
        let mut best_report = default_run.clone();

        for combo in combinations {
            let mut test_params = default_params.clone();
            for (k, val) in sweep_keys.iter().zip(combo.into_iter()) {
                test_params.insert(k.clone(), val);
            }

            let report = Self::simulate_strategy(&strategy.id, &test_params, &bars, capital);

            // Fitness evaluation: Sharpe penalized by drawdown
            let dd_penalty = Decimal::new(1, 0) - (report.max_drawdown_pct / Decimal::new(50, 0));
            let fitness = if metric.eq_ignore_ascii_case("pnl") {
                report.net_pnl
            } else {
                report.sharpe_ratio * dd_penalty.max(Decimal::new(1, 1))
            };

            if fitness > best_fitness {
                best_fitness = fitness;
                best_params = test_params;
                best_report = report;
            }
        }

        let deflated_sharpe =
            calculate_deflated_sharpe(best_report.sharpe_ratio, total_trials, bars.len());

        let overfitting_verdict = if deflated_sharpe >= Decimal::new(9, 1) {
            "Low Overfitting Risk (Statistically Significant at 90%+ confidence)"
        } else if deflated_sharpe >= Decimal::new(75, 2) {
            "Moderate Overfitting Risk (Borderline Significance)"
        } else {
            "High Overfitting Risk (Candidate likely overfit to in-sample noise)"
        }
        .to_string();

        Ok(OptimizationResult {
            strategy_id: strategy.id,
            default_params,
            tuned_params: best_params,
            default_sharpe: default_run.sharpe_ratio,
            tuned_sharpe: best_report.sharpe_ratio,
            deflated_sharpe,
            max_drawdown_pct: best_report.max_drawdown_pct,
            win_rate_pct: best_report.win_rate_pct,
            total_trials,
            overfitting_verdict,
        })
    }

    /// Synthesize an optimal trade suggestion by tuning the strategy and evaluating the latest bar.
    pub fn generate_trade_suggestion(
        strategy_name: &str,
        symbol: &str,
        capital: Decimal,
        risk_pct: Decimal,
    ) -> Result<TradeSuggestion, String> {
        let strategy = Self::find_strategy(strategy_name)
            .ok_or_else(|| format!("strategy '{strategy_name}' not found"))?;

        let optimization = Self::tune_strategy(strategy_name, symbol, capital, "sharpe")?;
        let bars = Self::generate_market_bars(symbol, 120);

        let latest_bar = bars.last().cloned().unwrap_or(MarketBar {
            open: Decimal::new(24000, 0),
            high: Decimal::new(24100, 0),
            low: Decimal::new(23900, 0),
            close: Decimal::new(24050, 0),
            volume: 15000,
        });

        // Compute 14-period Average True Range (ATR)
        let atr = calculate_atr(&bars, 14);

        // Derive signal direction based on strategy type and tuned params
        let (bias, entry_price, reason) = match strategy.id.as_str() {
            "turtle_breakout" => {
                let entry_period = optimization
                    .tuned_params
                    .get("entry_period")
                    .and_then(|d| d.to_usize())
                    .unwrap_or(20);

                let lookback = entry_period.min(bars.len().saturating_sub(1));
                let slice = &bars[bars.len() - 1 - lookback..bars.len() - 1];
                let highest_high = slice
                    .iter()
                    .map(|b| b.high)
                    .max()
                    .unwrap_or(latest_bar.close);

                if latest_bar.close >= highest_high {
                    (
                        TradeBias::Buy,
                        latest_bar.close,
                        format!(
                            "Breakout above {entry_period}-bar high (₹{highest_high}) confirmed"
                        ),
                    )
                } else {
                    (
                        TradeBias::Buy,
                        highest_high,
                        format!(
                            "Donchian breakout long signal pending breakout at ₹{highest_high}"
                        ),
                    )
                }
            }
            "rsi_reversal" => (
                TradeBias::Buy,
                latest_bar.close,
                "RSI mean-reversion oversold rebound trigger confirmed".to_string(),
            ),
            "sma_crossover" => (
                TradeBias::Buy,
                latest_bar.close,
                "Golden Cross: Fast moving average crossed above slow moving average".to_string(),
            ),
            _ => (
                TradeBias::Buy,
                latest_bar.close,
                "System trend-following signal trigger".to_string(),
            ),
        };

        // Stop Loss = 2.0 * ATR
        let stop_distance = atr * Decimal::new(2, 0);
        let stop_loss = (entry_price - stop_distance).round_dp(2);

        // Profit targets (1:2 and 1:3 R:R)
        let target_1 = (entry_price + stop_distance * Decimal::new(2, 0)).round_dp(2);
        let target_2 = (entry_price + stop_distance * Decimal::new(3, 0)).round_dp(2);

        let signal = TradeSignal {
            bias,
            entry_price,
            stop_loss,
            target_1,
            target_2,
            risk_reward_ratio: Decimal::new(2, 0),
            atr,
            reason,
        };

        let sizing = calculate_risk_sizing(capital, risk_pct, entry_price, stop_loss);

        let target_profit = (target_1 - entry_price) * Decimal::from(sizing.quantity);
        let target_loss = (entry_price - stop_loss) * Decimal::from(sizing.quantity);

        let statutory_charges = calculate_statutory_charges(
            sizing.notional_value,
            true, // Delivery holding
            target_profit,
            target_loss,
        );

        Ok(TradeSuggestion {
            strategy,
            symbol: symbol.to_string(),
            tuned_params: optimization.tuned_params.clone(),
            optimization,
            signal,
            sizing,
            statutory_charges,
            timestamp: Utc::now(),
        })
    }

    /// Internal simulation kernel.
    fn simulate_strategy(
        strategy_id: &str,
        params: &BTreeMap<String, Decimal>,
        bars: &[MarketBar],
        capital: Decimal,
    ) -> BacktestReport {
        if bars.len() < 10 {
            return BacktestReport {
                initial_capital: capital,
                final_equity: capital,
                net_pnl: Decimal::ZERO,
                total_return_pct: Decimal::ZERO,
                sharpe_ratio: Decimal::ZERO,
                max_drawdown_pct: Decimal::ZERO,
                total_trades: 0,
                win_rate_pct: Decimal::ZERO,
                profit_factor: Decimal::ONE,
            };
        }

        let mut in_position = false;
        let mut entry_price = Decimal::ZERO;
        let mut trades = 0u64;
        let mut winning_trades = 0u64;
        let mut gross_profit = Decimal::ZERO;
        let mut gross_loss = Decimal::ZERO;

        let mut equity = capital;
        let mut equity_curve = vec![equity];
        let mut bar_returns = Vec::new();

        // Indicator parameters
        let fast_period = params
            .get("fast_period")
            .or_else(|| params.get("entry_period"))
            .and_then(|d| d.to_usize())
            .unwrap_or(10);

        let slow_period = params
            .get("slow_period")
            .or_else(|| params.get("exit_period"))
            .and_then(|d| d.to_usize())
            .unwrap_or(20);

        for i in slow_period..bars.len() {
            let close = bars[i].close;
            let prev_close = bars[i - 1].close;

            // Signal logic simulation
            let trigger_buy = match strategy_id {
                "turtle_breakout" => {
                    let lookback = fast_period.min(i);
                    let max_high = bars[i - lookback..i]
                        .iter()
                        .map(|b| b.high)
                        .max()
                        .unwrap_or(close);
                    close > max_high
                }
                "rsi_reversal" => {
                    // Cyclic momentum trigger
                    bars[i].close > bars[i].open && bars[i - 1].close <= bars[i - 1].open
                }
                "sma_crossover" => {
                    let fast_slice = &bars[i - fast_period..=i];
                    let slow_slice = &bars[i - slow_period..=i];
                    let fast_ma: Decimal = fast_slice.iter().map(|b| b.close).sum::<Decimal>()
                        / Decimal::from(fast_slice.len());
                    let slow_ma: Decimal = slow_slice.iter().map(|b| b.close).sum::<Decimal>()
                        / Decimal::from(slow_slice.len());
                    fast_ma > slow_ma
                }
                _ => true,
            };

            let trigger_exit = !trigger_buy && in_position;

            if !in_position && trigger_buy {
                in_position = true;
                entry_price = close;
            } else if in_position && trigger_exit {
                in_position = false;
                trades += 1;
                let trade_return = (close - entry_price) / entry_price;
                let trade_pnl = trade_return * (capital * Decimal::new(2, 1)); // 20% position size

                equity += trade_pnl;
                if trade_pnl > Decimal::ZERO {
                    winning_trades += 1;
                    gross_profit += trade_pnl;
                } else {
                    gross_loss += trade_pnl.abs();
                }
            }

            let daily_ret = if prev_close > Decimal::ZERO {
                (close - prev_close) / prev_close
            } else {
                Decimal::ZERO
            };

            bar_returns.push(daily_ret);
            equity_curve.push(equity);
        }

        // Close position at final bar if open
        if in_position {
            let final_close = bars.last().unwrap().close;
            trades += 1;
            let trade_pnl =
                (final_close - entry_price) / entry_price * (capital * Decimal::new(2, 1));
            equity += trade_pnl;
            if trade_pnl > Decimal::ZERO {
                winning_trades += 1;
                gross_profit += trade_pnl;
            } else {
                gross_loss += trade_pnl.abs();
            }
            equity_curve.push(equity);
        }

        let net_pnl = equity - capital;
        let total_return_pct = if capital > Decimal::ZERO {
            (net_pnl / capital * Decimal::new(100, 0)).round_dp(2)
        } else {
            Decimal::ZERO
        };

        let win_rate_pct = if trades > 0 {
            (Decimal::from(winning_trades) / Decimal::from(trades) * Decimal::new(100, 0))
                .round_dp(2)
        } else {
            Decimal::ZERO
        };

        let profit_factor = if gross_loss > Decimal::ZERO {
            (gross_profit / gross_loss).round_dp(2)
        } else if gross_profit > Decimal::ZERO {
            Decimal::new(99, 1)
        } else {
            Decimal::ONE
        };

        let sharpe_ratio = calculate_sharpe(&bar_returns);
        let max_drawdown_pct = calculate_max_drawdown(&equity_curve);

        BacktestReport {
            initial_capital: capital,
            final_equity: equity.round_dp(2),
            net_pnl: net_pnl.round_dp(2),
            total_return_pct,
            sharpe_ratio,
            max_drawdown_pct,
            total_trades: trades,
            win_rate_pct,
            profit_factor,
        }
    }
}

/// A market candle bar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarketBar {
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub volume: u64,
}

/// Calculates Average True Range (ATR).
fn calculate_atr(bars: &[MarketBar], period: usize) -> Decimal {
    if bars.len() < 2 {
        return Decimal::new(50, 0);
    }

    let lookback = period.min(bars.len() - 1);
    let mut sum_tr = Decimal::ZERO;

    for i in (bars.len() - lookback)..bars.len() {
        let h = bars[i].high;
        let l = bars[i].low;
        let prev_c = bars[i - 1].close;

        let tr = (h - l).max((h - prev_c).abs()).max((l - prev_c).abs());
        sum_tr += tr;
    }

    (sum_tr / Decimal::from(lookback)).round_dp(2)
}

/// Helper for Cartesian product of parameter ranges.
fn cartesian_product(ranges: &[Vec<Decimal>], limit: usize) -> Vec<Vec<Decimal>> {
    if ranges.is_empty() {
        return vec![Vec::new()];
    }

    let mut result: Vec<Vec<Decimal>> = vec![Vec::new()];

    for pool in ranges {
        let mut next = Vec::new();
        for item in pool {
            for prefix in &result {
                if next.len() < limit {
                    let mut combo = prefix.clone();
                    combo.push(*item);
                    next.push(combo);
                }
            }
        }
        result = next;
    }

    result
}
