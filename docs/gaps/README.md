# Gaps — `vyapaar-ui`

Every gap filed in this repo, newest last. See
`.agents/rules/gap-tracking.md` for the binding rule.

## Open

| # | Title | Area |
| :--- | :--- | :--- |
| — | — | — |

## Closed

| # | Title | Area | Resolution |
| :--- | :--- | :--- | :--- |
| 0001 | CLI `scan` was `NotWired`; wired to canonical screener | `vyapaar-cli` runner | Implemented via `crates/vyapaar-cli/src/scan.rs` + `runner.rs` (`CommandOutcome::ScanResults`); tests in `crates/vyapaar-cli/tests/unit_tests.rs`. Gap doc removed (implemented). |
| 0002 | Job `trades.json` wrote aggregates only; no per-trade rows | `vyapaar-ui-core` strategy | `BacktestReport.trades: Vec<TradeRow>` + `simulate_strategy` row emission; `jobs_runtime.rs` journals the row list; tests in `crates/vyapaar-ui-core/tests/strategy_primitives_test.rs`. Gap doc removed (implemented). |
