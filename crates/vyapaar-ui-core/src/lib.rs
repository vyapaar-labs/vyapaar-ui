//! # व्यापार・face vyapaar-ui-core
//!
//! Presentational primitives for the Vyapaar workstation: pure layout and
//! rendering atoms/molecules, screener table primitives, home dashboard
//! primitives, and strategy optimization / trade suggestion models.
//!
//! Contracts (migration-guide.md §1 and `vyapaar-ui/.agents/AGENTS.md`):
//! - **Presentational purity**: atoms describe *layout/rendering* intent;
//!   reactive state signals live in the feature crates (`vyapaar-ui-app`).
//! - **WASM-safe**: zero-I/O, no `tokio`, compiles for `wasm32-unknown-unknown`.
//! - **Zero-floats**: prices, deltas, and ratios use `rust_decimal::Decimal`
//!   (re-exported through `vyapaar_core`).
//! - **Tests in `tests/`**: `ui_core_primitives_test.rs`,
//!   `screener_primitives_test.rs`, `home_primitives_test.rs`, `strategy_primitives_test.rs`.

pub mod algos;
pub mod atoms;
pub mod backtest;
pub mod home;
pub mod portfolio;
pub mod screener;
pub mod strategy;

pub use algos::{AlgoManagerViewModel, AlgoRow, AlgoStatus};
pub use atoms::{
    Accent, Badge, BadgeTone, Delta, EquityPoint, MetricValue, Spacing, StatusPill, Tag, TextAlign,
    TrendDirection, TrendPill,
};
pub use backtest::{BacktestResultView, BacktestStatus, BacktestSummary};
pub use home::{AccountSummary, MarketStatus, MarketStatusKind, WatchlistRow, WatchlistViewModel};
pub use portfolio::{
    PortfolioHoldingRow, PortfolioPositionRow, PortfolioTotals, PortfolioViewModel,
};
pub use screener::{
    ScreenerColumn, ScreenerFilterChip, ScreenerRowModel, ScreenerSort, ScreenerSortOrder,
    ScreenerViewModel,
};
pub use strategy::{
    BacktestReport, OptimizationResult, RiskSizing, StatutoryCharges, StrategyDescriptor,
    StrategyParameterSpec, StrategySourceKind, TradeBias, TradeRow, TradeSignal, TradeSuggestion,
    calculate_deflated_sharpe, calculate_max_drawdown, calculate_risk_sizing, calculate_sharpe,
    calculate_statutory_charges,
};
