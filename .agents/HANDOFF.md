# `vyapaar-ui` — Inter-Agent Handoff (Tier 5)

> Updated: 2026-11-05 — Tier 5 (vyapaar-ui) migration complete.

## 1. Phase / Slice Status
- **TIER 5 COMPLETE**: all 9 crates implemented and verified.
- Ported/built: `vyapaar-ui-core`, `vyapaar-ui-assets`, `vyapaar-ui-charts`,
  `vyapaar-ui-client`, `vyapaar-ui-api-wasm`, `vyapaar-ui-app`,
  `vyapaar-highlight` (new), `vyapaar-tui` (new), `vyapaar-cli` (new).
- Verification gate: ✅
  ```bash
  grep -rn "#\[cfg(test)\]" crates/*/src/            # no matches
  cargo fmt --check                                  # clean
  cargo clippy --workspace --all-targets -- -D warnings   # clean
  cargo test --workspace                             # 108 tests, 0 failures
  cargo check -p vyapaar-ui-app --target wasm32-unknown-unknown  # green
  ```
- 108 unit tests total across 28 test files. `vyapaar health` CLI verified.

## 2. Decisions & Discoveries
- honba-kao source tree was **not available** in this environment; all crates
  were implemented fresh against the migration-guide contracts.
- `uuid` v4 (transitive via `vyapaar-core`) requires an RNG source on
  `wasm32-unknown-unknown`. Fixed in the workspace by enabling `uuid`
  `rng-getrandom` + `getrandom` `wasm_js` features from `vyapaar-ui-app`
  (feature unification).
- `ScreenerFilterChip` is a presentational type in `vyapaar-ui-core`; the
  canonical wire filter is `vyapaar_core::screener::ScreenerFilterPredicate`.
- `vyapaar-highlight` is a hand-rolled char lexer (no tree-sitter) to stay
  zero-I/O and WASM-safe, per `docs/plans/0001` §1.3.
- WASM crates depend on zero `tokio`/`ratatui`/`crossterm`/`clap`; native deps
  isolated to `vyapaar-{cli,tui}`.
- `docs/migration-guide.md` §4 records the full per-crate summary.

## 3. Immediate Next Steps (outside tier-5 gate)
1. **Leptos view layer** for `vyapaar-ui-app` (feature state models are done;
   no CSR rendering yet).
2. **Wire `vyapaar-cli ask/scan`** to `vyapaar-agent` / `vyapaar-market`
   screener once those APIs land (currently `NotWired` errors by design).
3. **TUI `run()` event loop** (crossterm `event::read` + `TuiApp::on_key`);
   state/keybinding/layout logic is tested already.
4. When `honba-workspace` is available, diff these fresh implementations
   against `honba-kao` for behavior parity.

## 4. Operating Invariants
- Zero `#[cfg(test)]` / `#[cfg(test)] mod tests` inside `crates/*/src/`.
- Zero floats: all financial math is `rust_decimal::Decimal`.
- WASM crates (`ui-core`, `ui-assets`, `ui-charts`, `ui-client`, `ui-api-wasm`,
  `ui-app`) must compile to `wasm32-unknown-unknown`; do not add `tokio`
  there.
- Tests live in `<crate>/tests/` parallel to `src/`.
- Edition 2024, rust-version 1.85, AGPL-3.0.
