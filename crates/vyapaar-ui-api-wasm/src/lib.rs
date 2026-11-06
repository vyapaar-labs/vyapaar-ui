//! # व्यापार・face vyapaar-ui-api-wasm
//!
//! Pure, WASM-safe computation bindings for the Vyapaar workstation
//! (migration of `honba-kao-api-wasm`'s `bindings.rs` / `lib.rs` and its
//! eight test suites: analytics, indicators, margin, pairing, payoff,
//! pricing, replay, screener).
//!
//! Every function here is a deterministic, zero-I/O computation over exact
//! `rust_decimal::Decimal` and `vyapaar-core` wire types — the same code runs
//! inside the WASM Strategy Studio and in native tests. The WASM shim layer
//! (JS `bindgen`) is deliberately thin and lives outside this crate.
//!
//! Contracts:
//! - **WASM-safe**: no `tokio`, no filesystem, no OS threads.
//! - **Zero-floats**: all math is `Decimal`; no `f32`/`f64` in financial paths.
//! - **Tests in `tests/`**: `analytics_test.rs`, `indicators_test.rs`,
//!   `margin_test.rs`, `pairing_test.rs`, `payoff_test.rs`, `pricing_test.rs`,
//!   `replay_test.rs`, `screener_test.rs`.

pub mod analytics;
pub mod bindings;
pub mod indicators;
pub mod margin;
pub mod pairing;
pub mod payoff;
pub mod pricing;
pub mod replay;
pub mod screener;

pub use analytics::{drawdown, sharpe_ratio, sortino_ratio};
pub use bindings::{ApiError, ApiResult, Bindings};
pub use indicators::{ema, rsi, sma};
pub use margin::margin_required;
pub use pairing::pair_series;
pub use payoff::{OptionType, payoff};
pub use pricing::{notional, pnl};
pub use replay::{PositionView, replay_ticks};
pub use screener::{ScanError, apply_scan};
