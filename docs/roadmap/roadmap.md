# `vyapaar-ui` — TDD & Engineering Roadmap

## Milestone 1: Portable UI Core, Assets, Client, Charts & API-WASM (P0)
- [ ] Port `vyapaar-ui-core` (from `honba-kao-ui-core`), `vyapaar-ui-assets` (from `honba-kao-assets`), `vyapaar-ui-client` (from `honba-kao-client`), `vyapaar-ui-charts` (from `openalgo-charts-rs`), and `vyapaar-ui-api-wasm` (from `honba-kao-api-wasm`).

## Milestone 2: Leptos 0.8 WASM Workstation (`vyapaar-ui-app`) (P0)
- [ ] Port `vyapaar-ui-app` (from `honba-kao-app`), including the Futuristic Neural AST Screener (`telemetry`, `signal_matrix`, `constellation`, `filter_bar`, `detail_drawer`) and `/components` showcase route.
- [ ] Verify `cargo check -p vyapaar-ui-app --target wasm32-unknown-unknown` passes cleanly.

## Milestone 3: Native Terminal Workstation (`vyapaar-highlight`, `vyapaar-tui`, `vyapaar-cli`) (P1)
- [ ] Implement `vyapaar-highlight` (token-based syntax highlighter for Python/Rust/TOML/AST).
- [ ] Implement `vyapaar-tui` (Ratatui state model & terminal layout views for Screener, Portfolio, and AI Agent).
- [ ] Implement `vyapaar-cli` (`Clap` command parser & dispatcher for `tui`, `ask`, `scan`, `version`).
