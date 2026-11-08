use rust_decimal::Decimal;
use vyapaar_ui_core::strategy::{
    StrategyDescriptor, StrategyParameterSpec, StrategySourceKind, TradeBias,
    calculate_deflated_sharpe, calculate_max_drawdown, calculate_risk_sizing, calculate_sharpe,
    calculate_statutory_charges,
};

#[test]
fn test_strategy_descriptor_builder() {
    let desc = StrategyDescriptor::new(
        "turtle_breakout",
        "Turtle Breakout System",
        "0.1.0",
        StrategySourceKind::Builtin,
    )
    .with_description("Classic Donchian channel breakout trend-following system")
    .with_parameter(
        StrategyParameterSpec::new("entry_period", Decimal::new(20, 0))
            .with_range(Decimal::new(5, 0), Decimal::new(60, 0), Decimal::new(1, 0))
            .with_description("Entry breakout lookback bars"),
    )
    .with_instrument("NSE:NIFTY50-INDEX");

    assert_eq!(desc.id, "turtle_breakout");
    assert_eq!(desc.source, StrategySourceKind::Builtin);
    assert_eq!(desc.parameters.len(), 1);
    assert_eq!(desc.parameters[0].default, Decimal::new(20, 0));
    assert_eq!(desc.parameters[0].min, Some(Decimal::new(5, 0)));
    assert_eq!(desc.parameters[0].max, Some(Decimal::new(60, 0)));
    assert_eq!(desc.target_instruments, vec!["NSE:NIFTY50-INDEX"]);
}

#[test]
fn test_risk_sizing_calculation() {
    let capital = Decimal::new(1000000, 0); // ₹10,00,000
    let risk_pct = Decimal::new(10, 1); // 1.0% = ₹10,000
    let entry = Decimal::new(24000, 0);
    let stop_loss = Decimal::new(23800, 0); // ₹200 risk per share

    let sizing = calculate_risk_sizing(capital, risk_pct, entry, stop_loss);

    assert_eq!(sizing.risk_amount, Decimal::new(10000, 0));
    assert_eq!(sizing.quantity, 50); // 10000 / 200 = 50
    assert_eq!(sizing.notional_value, Decimal::new(1200000, 0)); // 50 * 24000
    assert_eq!(sizing.margin_utilization_pct, Decimal::new(120, 0));
}

#[test]
fn test_statutory_charges_calculation() {
    let notional = Decimal::new(1000000, 0); // ₹10,00,000 notional
    let target_profit = Decimal::new(20000, 0);
    let target_loss = Decimal::new(10000, 0);

    let charges = calculate_statutory_charges(notional, true, target_profit, target_loss);

    assert_eq!(charges.stt, Decimal::new(1000, 0)); // 0.1% on delivery
    assert!(charges.exchange_turnover > Decimal::ZERO);
    assert!(charges.sebi_charges > Decimal::ZERO);
    assert!(charges.stamp_duty > Decimal::ZERO);
    assert!(charges.gst > Decimal::ZERO);
    assert!(charges.total_charges > Decimal::new(1000, 0));
    assert!(charges.net_risk_reward_ratio < Decimal::new(2, 0)); // Adjusted downward from 2.0 due to friction
}

#[test]
fn test_sharpe_and_drawdown_metrics() {
    let returns = vec![
        Decimal::new(1, 2),
        Decimal::new(-5, 3),
        Decimal::new(2, 2),
        Decimal::new(15, 3),
        Decimal::new(-8, 3),
        Decimal::new(12, 3),
        Decimal::new(4, 3),
    ];
    let sharpe = calculate_sharpe(&returns);
    assert!(sharpe > Decimal::ZERO);

    let equity = vec![
        Decimal::new(100, 0),
        Decimal::new(110, 0),
        Decimal::new(105, 0), // -4.54% from 110
        Decimal::new(120, 0),
        Decimal::new(114, 0), // -5.0% from 120
        Decimal::new(125, 0),
    ];
    let max_dd = calculate_max_drawdown(&equity);
    assert_eq!(max_dd, Decimal::new(5, 0));
}

#[test]
fn test_deflated_sharpe_ratio() {
    // High Sharpe with 1 trial should have very high confidence
    let dsr_single = calculate_deflated_sharpe(Decimal::new(25, 1), 1, 100);
    assert!(dsr_single > Decimal::new(9, 1));

    // Moderate Sharpe with 100 trials suffers from multiple testing penalty
    let dsr_mined = calculate_deflated_sharpe(Decimal::new(11, 1), 100, 100);
    assert!(dsr_mined < dsr_single);
}

#[test]
fn test_trade_bias_formatting() {
    assert_eq!(TradeBias::Buy.as_str(), "BUY / LONG");
    assert_eq!(TradeBias::Sell.as_str(), "SELL / SHORT");
    assert_eq!(TradeBias::Neutral.as_str(), "NEUTRAL / HOLD");
}
