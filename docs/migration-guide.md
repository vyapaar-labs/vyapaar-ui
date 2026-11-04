# `vyapaar-ui` — Subagent Migration & DDD Refactoring Guide

> **Tier**: Tier 5A (Depends on `vyapaar-core`)
> **Source**: `/home/pmallapp/Devel/Finance/honba-workspace/honba-kao`
> **Target**: `/home/pmallapp/Devel/Finance/vyapaar-labs/vyapaar-ui`

---

## 1. Sub-Crate Mapping Table

| Source Crate (`honba-kao/crates/`) | Target Crate (`vyapaar-ui/crates/`) | Notes |
| :--- | :--- | :--- |
| `honba-kao-ui-core` | `vyapaar-ui-core` | Port all UI atoms/molecules and tests (`ui_core_primitives_test.rs`, `screener_primitives_test.rs`, `home_primitives_test.rs`). |
| `honba-kao-assets` | `vyapaar-ui-assets` | Port `assets/` directory (`style/components.css`, fonts, themes) and `src/` (`branding.rs`, `themes.rs`, `fonts.rs`, `fixtures.rs`, `audio.rs`). |
| `honba-kao-client` | `vyapaar-ui-client` | Port `events.rs`, `stream.rs`, `lib.rs`, and `tests/`. |
| `openalgo-charts-rs` | `vyapaar-ui-charts` | Rename crate `openalgo-charts-rs` → `vyapaar-ui-charts` (or re-export alias `openalgo_charts_rs`). Port all 7 chart test suites. |
| `honba-kao-api-wasm` | `vyapaar-ui-api-wasm` | Port `bindings.rs`, `lib.rs`, and all 8 test suites (`analytics`, `indicators`, `margin`, `pairing`, `payoff`, `pricing`, `replay`, `screener`). |
| `honba-kao-app` | `vyapaar-ui-app` | Port all features (`screener`, `studio`, `simulator`, `system`, `router`) and all 14 test suites (105 tests). |
| *(New Crate)* | `vyapaar-highlight` | Implement zero-I/O syntax highlighter (`Language`, `HighlightSpan`, `highlight_code`) with unit tests in `tests/unit_tests.rs`. |
| *(New Crate)* | `vyapaar-tui` | Implement `TuiApp`, `ActiveTab` (`Screener`, `Portfolio`, `Orders`, `AgentChat`), keybinding handler, and layout model with tests in `tests/unit_tests.rs`. |
| *(New Crate)* | `vyapaar-cli` | Implement `CliArgs`, `Command` (`Tui`, `Ask`, `Scan`, `Health`), and runner logic with tests in `tests/unit_tests.rs`. |

---

## 2. Mandatory DDD & Portability Fixes During Migration
1. **Do NOT copy `honba-kao/crates-old/`**: Ignore `crates-old/` completely.
2. **WASM Portability in `vyapaar-ui/Cargo.toml`**:
   - Ensure WASM crates (`vyapaar-ui-core`, `vyapaar-ui-assets`, `vyapaar-ui-client`, `vyapaar-ui-charts`, `vyapaar-ui-api-wasm`, `vyapaar-ui-app`) do **NOT** pull in `tokio` with `net`/`rt-multi-thread` so `cargo check -p vyapaar-ui-app --target wasm32-unknown-unknown` succeeds.
3. **Copy Root Web Assets**: Copy `honba-kao/index.html`, `honba-kao/components.html`, `honba-kao/Trunk.toml`, and `honba-kao/scripts/` into `vyapaar-ui/`, updating crate/binary references from `honba-kao-app` to `vyapaar-ui-app`.

## 3. Verification Commands
```bash
cd /home/pmallapp/Devel/Finance/vyapaar-labs/vyapaar-ui
grep -rn "#\[cfg(test)\]" crates/*/src/ && exit 1 || true
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --workspace
cargo check -p vyapaar-ui-app --target wasm32-unknown-unknown
```
