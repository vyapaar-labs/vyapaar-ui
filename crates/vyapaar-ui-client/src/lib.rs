//! # व्यापार・face vyapaar-ui-client
//!
//! WASM-safe UI event model and subscription stream for the Vyapaar
//! workstation (migration of `honba-kao-client`'s `events.rs` / `stream.rs` /
//! `lib.rs`).
//!
//! Contracts:
//! - **WASM-safe**: no `tokio`, no OS threads, no blocking I/O. The event
//!   stream is a push-based in-memory broadcast — the same seam the
//!   workstation (Leptos app / Ratatui TUI) uses regardless of target.
//! - **Canonical payloads**: events carry `vyapaar-core` wire types
//!   ([`Tick`], [`Bar`], [`OrderEvent`], [`FillReport`],
//!   [`PortfolioSnapshot`], [`ScreenerScanResponse`]), never broker-specific
//!   structs.
//! - **Deterministic delivery order**: events are delivered to subscribers in
//!   publish order; every subscriber receives every event.
//! - **Tests in `tests/`**.

pub mod events;
pub mod stream;

pub use events::{ClientEvent, EventKind, SystemStatus};
pub use stream::{EventStream, Subscription};
