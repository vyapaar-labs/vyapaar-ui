//! # व्यापार・face vyapaar-ui-core
//!
//! Presentational primitives for the Vyapaar workstation: pure layout and
//! rendering atoms/molecules, screener table primitives, and home dashboard
//! primitives.
//!
//! Contracts (migration-guide.md §1 and `vyapaar-ui/.agents/AGENTS.md`):
//! - **Presentational purity**: atoms describe *layout/rendering* intent;
//!   reactive state signals live in the feature crates (`vyapaar-ui-app`).
//! - **WASM-safe**: zero-I/O, no `tokio`, compiles for `wasm32-unknown-unknown`.
//! - **Zero-floats**: prices, deltas, and ratios use `rust_decimal::Decimal`
//!   (re-exported through `vyapaar_core`).
//! - **Tests in `tests/`**: `ui_core_primitives_test.rs`,
//!   `screener_primitives_test.rs`, `home_primitives_test.rs`.

pub mod atoms;
pub mod home;
pub mod screener;

pub use atoms::{
    Accent, Badge, BadgeTone, Delta, EquityPoint, MetricValue, Spacing, StatusPill, Tag,
    TextAlign, TrendDirection, TrendPill,
};
pub use home::{AccountSummary, MarketStatus, MarketStatusKind, WatchlistRow};
pub use screener::{
    ScreenerColumn, ScreenerFilterChip, ScreenerRowModel, ScreenerSort, ScreenerSortOrder,
    ScreenerViewModel,
};
