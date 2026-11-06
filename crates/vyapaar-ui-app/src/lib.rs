//! # व्यापार・face vyapaar-ui-app
//!
//! The Vyapaar workstation application crate: router, screener, strategy
//! studio, simulator, and system features.
//!
//! Contracts (migration-guide.md §1 / `vyapaar-ui/.agents/AGENTS.md`):
//! - **WASM-portable**: this crate must compile cleanly to
//!   `wasm32-unknown-unknown` (`cargo check -p vyapaar-ui-app --target
//!   wasm32-unknown-unknown`). No `tokio`, no OS threads, no blocking I/O.
//! - **Zero-floats**: every financial value is `rust_decimal::Decimal`.
//! - **Feature/state separation**: each feature module owns its *reactive
//!   state* and transitions; presentational primitives come from
//!   `vyapaar-ui-core`. Rendering (Leptos, canvas) is left to the host.
//! - **Tests in `tests/`** (parallel test directory rule).

pub mod router;
pub mod screener;
pub mod simulator;
pub mod studio;
pub mod system;

pub use router::{AppRoute, Router, RouterError};
pub use screener::{ScreenerFeature, ScreenerFeatureError};
pub use simulator::{SimulatorFeature, SimulatorPhase};
pub use studio::{Connection, NodeId, StudioCanvas, StudioNode, StudioNodeKind};
pub use system::{SubsystemState, SystemFeature};
