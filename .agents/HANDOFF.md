# `vyapaar-ui` — Inter-Agent Handoff (Layer 5)

> Updated: 2026-11-06 — Layer 5 (FinceptTerminal screen inventory, plan 0005)
> complete alongside Tier 5.

## 1. Phase / Slice Status
- **TIER 5 COMPLETE**: all 9 crates implemented and verified (see git history).
- **LAYER 5 COMPLETE** (plan `docs/plans/0005-fincept-terminal-screen-inventory.md`):
  - `vyapaar-ui-core`: `BacktestSummary`/`BacktestResultView`, `PortfolioViewModel`
    (+ position/holding/totals rows), `AlgoRow`/`AlgoManagerViewModel`/`AlgoStatus`,
    `WatchlistViewModel` — all Decimal-exact, zero-I/O, WASM-safe.
  - `vyapaar-ui-app`: `WatchlistFeature` (folds Tick/Bar), `BacktestFeature` (run
    lifecycle + equity curve), `PortfolioFeature` (ingests `PortfolioSnapshot`),
    `AlgosFeature`; router gained `Backtest`/`Portfolio`/`Algos` routes.
  - `vyapaar-tui`: seven-tab workstation (`Watchlist`, `Screener`, `Portfolio`,
    `Orders`, `Backtest`, `Algos`, `AgentChat`); starts on Watchlist.
  - Two-screener question **resolved** in plan 0005 §2 item 2: UI builds only on
    `vyapaar_core::screener`; `vyapaar-market-indicators::screener` is a standalone
    duplicate the UI ignores.
- Verification gate: ✅
  ```bash
  cargo fmt --check                                  # clean
  cargo clippy --workspace --all-targets -- -D warnings   # clean
  cargo test --workspace                             # 151 tests, 0 failures
  cargo check -p vyapaar-ui-app --target wasm32-unknown-unknown  # green
  ```

## 2. Decisions & Discoveries
- CLI `ask`/`scan` remain `NotWired` by design; gap filed:
  `docs/gaps/0001-cli-ask-scan-not-wired.md`.
- Screen data flows over canonical wire contracts (`vyapaar_core`), matching the
  `vyapaar-ui-client` event stream (`Tick`/`Bar`/`Portfolio`/`Screener`).
- Zero `#[cfg(test)]` inside `crates/*/src/`; tests live in `<crate>/tests/`.

## 3. Immediate Next Steps
1. Leptos view layer for `vyapaar-ui-app` (feature state models are done; no CSR
   rendering yet).
2. Wire `vyapaar-cli ask/scan` to backends — close `docs/gaps/0001-cli-ask-scan-not-wired.md`.
3. TUI `run()` event loop (crossterm `event::read` + `TuiApp::on_key`).

## 4. Operating Invariants
- Zero `#[cfg(test)]` / `#[cfg(test)] mod tests` inside `crates/*/src/`.
- Zero floats: all financial math is `rust_decimal::Decimal`.
- WASM crates (`ui-core`, `ui-assets`, `ui-charts`, `ui-client`, `ui-api-wasm`,
  `ui-app`) must compile to `wasm32-unknown-unknown`; no `tokio` there.
- Tests live in `<crate>/tests/` parallel to `src/`.
- Edition 2024, rust-version 1.85, AGPL-3.0.
