//! Deterministic demo fixtures for the workstation.
//!
//! These construct canonical `vyapaar-core` values so the Studio, TUI, and
//! WASM app have stable, reproducible sample data for screenshots and
//! development without any I/O.

use chrono::{DateTime, TimeZone, Utc};
use rust_decimal::Decimal;
use vyapaar_core::market::{Bar, DepthLevel, Tick};
use vyapaar_core::portfolio::{
    PortfolioAccount, PortfolioMode, PortfolioPosition, PortfolioSnapshot, PortfolioSource,
    PortfolioTotals,
};
use vyapaar_core::screener::{
    FilterOp, MetricKeySpec, MetricPeriod, MetricRef, ScreenerFilterGroup, ScreenerFilterPredicate,
    ScreenerScanRequest,
};
use vyapaar_core::types::{Currency, Exchange, InstrumentId, Money, Price, Quantity};
use vyapaar_core::instrument::PositionSide;
use vyapaar_ui_core::screener::ScreenerFilterChip;

fn dec(s: &str) -> Decimal {
    s.parse().unwrap()
}

fn utc(y: i32, m: u32, d: u32, h: u32, min: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(y, m, d, h, min, 0)
        .single()
        .expect("valid UTC timestamp")
}

/// NSE:RELIANCE-EQ instrument.
pub fn reliance() -> InstrumentId {
    InstrumentId::new(Exchange::Nse, "RELIANCE")
}

/// NSE:TCS instrument.
pub fn tcs() -> InstrumentId {
    InstrumentId::new(Exchange::Nse, "TCS")
}

/// A deterministic one-day bar series for RELIANCE.
pub fn reliance_bars(n: usize) -> Vec<Bar> {
    let inst = reliance();
    let mut bars = Vec::with_capacity(n);
    let mut close = dec("2400");
    let mut ts = utc(2026, 11, 2, 9, 15);
    for i in 0..n {
        let open = close;
        let high = open + dec("4");
        let low = open - dec("3");
        close = open + dec(&format!("{}", if i % 2 == 0 { 2 } else { -1 }));
        bars.push(Bar {
            instrument: inst.clone(),
            open: Price::new(open),
            high: Price::new(high),
            low: Price::new(low),
            close: Price::new(close),
            volume: Quantity::from_u64(10_000 + u64::from(i as u32) * 250),
            timestamp: ts,
        });
        ts += chrono::Duration::minutes(1);
    }
    bars
}

/// A deterministic quote tick for RELIANCE.
pub fn reliance_quote_tick() -> Tick {
    Tick::Quote {
        instrument: reliance(),
        bid: Price::new(dec("2401.50")),
        ask: Price::new(dec("2401.90")),
        bid_qty: Quantity::from_u64(350),
        ask_qty: Quantity::from_u64(220),
        last_price: Some(Price::new(dec("2401.75"))),
        exchange_ts: utc(2026, 11, 3, 10, 30),
        receive_ts: utc(2026, 11, 3, 10, 30),
        band: None,
    }
}

/// A deterministic L2 depth tick for TCS.
pub fn tcs_depth_tick() -> Tick {
    Tick::Depth {
        instrument: tcs(),
        bids: vec![
            DepthLevel {
                price: Price::new(dec("3400.10")),
                quantity: Quantity::from_u64(120),
                orders_count: Some(4),
            },
            DepthLevel {
                price: Price::new(dec("3400.00")),
                quantity: Quantity::from_u64(80),
                orders_count: Some(3),
            },
        ],
        asks: vec![
            DepthLevel {
                price: Price::new(dec("3400.40")),
                quantity: Quantity::from_u64(95),
                orders_count: Some(5),
            },
            DepthLevel {
                price: Price::new(dec("3400.60")),
                quantity: Quantity::from_u64(140),
                orders_count: Some(6),
            },
        ],
        last_price: Some(Price::new(dec("3400.25"))),
        exchange_ts: utc(2026, 11, 3, 10, 30),
        receive_ts: utc(2026, 11, 3, 10, 30),
    }
}

/// A deterministic portfolio snapshot with two positions.
pub fn portfolio_snapshot() -> PortfolioSnapshot {
    PortfolioSnapshot {
        account: PortfolioAccount {
            account_id: "demo-account-1".to_string(),
            mode: PortfolioMode::Paper,
            source: PortfolioSource::Simulated,
        },
        positions: vec![
            PortfolioPosition {
                instrument: reliance(),
                side: PositionSide::Long,
                quantity: Quantity::from_u64(50),
                avg_price: Price::new(dec("2390.00")),
                unrealized_pnl: Money::new(dec("587.50"), Currency::Inr),
                realized_pnl: Money::new(dec("120.00"), Currency::Inr),
            },
            PortfolioPosition {
                instrument: tcs(),
                side: PositionSide::Short,
                quantity: Quantity::from_u64(20),
                avg_price: Price::new(dec("3410.00")),
                unrealized_pnl: Money::new(dec("195.00"), Currency::Inr),
                realized_pnl: Money::new(dec("-40.00"), Currency::Inr),
            },
        ],
        holdings: Vec::new(),
        totals: PortfolioTotals {
            cash: Money::new(dec("450000.00"), Currency::Inr),
            margin_used: Money::new(dec("120000.00"), Currency::Inr),
            margin_available: Money::new(dec("330000.00"), Currency::Inr),
            total_pnl: Money::new(dec("862.50"), Currency::Inr),
            net_worth: Money::new(dec("583862.50"), Currency::Inr),
        },
        greeks: None,
        timestamp: utc(2026, 11, 3, 10, 30),
    }
}

/// A deterministic screener scan request: ROE > 15, PE between 10 and 30.
pub fn screener_request() -> ScreenerScanRequest {
    let roe = MetricRef(MetricKeySpec {
        metric: "roe".to_string(),
        period: MetricPeriod::D1,
    });
    let pe = MetricRef(MetricKeySpec {
        metric: "pe".to_string(),
        period: MetricPeriod::D1,
    });
    ScreenerScanRequest {
        universe: Some(vec![reliance(), tcs()]),
        filters: ScreenerFilterGroup {
            predicates: vec![
                ScreenerFilterPredicate {
                    metric: roe,
                    op: FilterOp::Gt,
                    value: dec("15"),
                    value_upper: None,
                },
                ScreenerFilterPredicate {
                    metric: pe,
                    op: FilterOp::Between,
                    value: dec("10"),
                    value_upper: Some(dec("30")),
                },
            ],
            match_all: true,
        },
        sort: None,
        limit: Some(25),
    }
}

/// Filter chips derived from [`screener_request`] for the filter bar.
pub fn filter_chips() -> Vec<ScreenerFilterChip> {
    vec![
        ScreenerFilterChip::new("roe", FilterOp::Gt, dec("15")),
        ScreenerFilterChip::new("pe", FilterOp::Between, dec("10")).with_upper(dec("30")),
    ]
}
