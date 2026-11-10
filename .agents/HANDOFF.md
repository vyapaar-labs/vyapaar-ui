# `vyapaar-ui` — Inter-Agent Handoff (Layer 5)

> Updated: 2026-11-10 — added `vyapaar serve` (no server binary existed anywhere in the
> workspace before this). PLAN-0001 Stage 4 (CLI & agent jobs unification) complete;
> `docs/plans` and `docs/gaps` reconciled against the green tree (plan `0001` moved to
> Implemented; gaps `0001` and `0002` CLOSED). Alongside the earlier Layer 5 / Tier 5
> work below.

## 1. Phase / Slice Status
- **PLAN-0001 STAGE 4 COMPLETE** (`vyapaar-runtime/docs/plans/0001`, CLI half —
  runtime Stages 1–3 landed in the `vyapaar-runtime` repo):
  - `vyapaar-cli/src/jobs_runtime.rs`: `JobsRuntime` — `vyapaar jobs
    add/list/run/tick/pause/resume/approvals/approve/deny/logs` wired to
    `BoundedWorkerPool` + `FileJobRegistry` + `FileJournalWriter` (path deps on
    `vyapaar-runtime-jobs` / `-api` / `-api-rest`, **native binaries only**; must
    never leak into WASM modules).
  - Job-run journaling: per-run journal dir `journals/<run_id>/` with
    `manifest.json` (`kind: "job"`), `events.ndjson` (`run.queued`, `run.started`,
    `run.result|run.failed`, `run.finalized`), `trades.json`, `metrics.json`.
    Budget-429 refusals mint no run id and write no journal.
  - Agent tool calls journal: `StrategyBridgeRunner` (`kind: "tool"`,
    `tool.started` + `tool.result|tool.failed`) — best-effort, never breaks tool
    results; `jobs add --approval auto|manual` gates breaches into the approvals
    queue (`record_usage` breach/manual → `PendingApproval`, `deny` → Paused,
    `approve` → re-activates).
  - CLI args: `--approval/--strategy/--symbol/--capital/--params` on
    `vyapaar jobs add`; boot recovery wired in `main.rs`
    (`jobs_runtime::boot_recover`, prints recovered-run count unless quiet).
  - **Acceptance roundtrip proven** in
    `crates/vyapaar-cli/tests/jobs_runtime_test.rs` (11 tests): add → tick →
    journal verified; once-jobs, failed-strategy journals, all five refusal
    classes, budget-breach → deny → Paused, manual approval gating, saturated
    pool backpressure, agent-tool journaling, boot recovery, legacy
    `jobs.json` migration, `resolve_job` prefix matching.
- **TIER 5 / LAYER 5 COMPLETE** (earlier work, see git history): nine crates;
  seven-tab TUI workstation; feature state models; string-decimal wire contracts.
- Verification gate:
  - `cargo clippy --workspace --all-targets --profile prod -- -D warnings` ✅
  - `cargo test --workspace --profile prod` ✅ as of Stage 4: 59 targets, 275 passed
    (`vyapaar-cli` unit tests 33 incl. 7 scan tests; `vyapaar-ui-core` strategy
    primitives 9 incl. 3 trade-row tests); `cargo test -p vyapaar-cli --profile prod` ✅
    34 passed after the `vyapaar serve` batch (added `parse_cli_roundtrip_serve`)
  - `cargo fmt` ✅ on all Stage 4 touched files (pre-existing drift in
    `vyapaar-tui` / `vyapaar-ui-core` committed files predates this batch and is
    out of scope here)
  - `make prod` ✅ fresh `target/prod`

## 2. Decisions & Discoveries
- CLI `ask`/`chat`/`tui` wired to LLM backends (GGUF in-process / OpenAI
  endpoints); **`scan` closed** — wired to canonical screener via
  `crates/vyapaar-cli/src/scan.rs` + `runner.rs`
  (`CommandOutcome::ScanResults`; tests in `crates/vyapaar-cli/tests/unit_tests.rs`).
- **`trades.json` closed** — job runs now journal per-trade rows (serialized
  `BacktestReport.trades`) instead of `{"total_trades": N}`; `BacktestReport`
  carries `trades: Vec<TradeRow>` emitted by `simulate_strategy`
  (tests in `crates/vyapaar-ui-core/tests/strategy_primitives_test.rs`).
- Registry ops order per run: claim → pool dispatch → release_lease →
  record_usage (breach/manual wins over lease) → advance cron cursor; cron
  evaluation via the runtime `JobScheduler` (`cron` crate, Vixie dom/dow OR).
- Determinism: all job/manifest timestamps are explicit `DateTime<Utc>` passed
  from the caller; journal events carry only seq + type + engine output.
- Zero `#[cfg(test)]` inside `crates/*/src/`; tests live in `<crate>/tests/`.
- **`vyapaar serve` added** — `vyapaar-runtime-api-rest::bootstrap::{ServeConfig,
  run_server_blocking}` was a complete, tested library with no binary ever calling it
  (same gap existed in `honba-ya`); `vyapaar-cli` already depended on the crate. Added
  `Command::Serve(ServeArgs)` (`args.rs`), `run_serve` (`runner.rs`) mapping CLI flags
  (`data_dir` positional, `--addr`, `--cors-origin`, `--profiles-dir`, `--portfolios-dir`,
  `--journals-dir`, `--tokens-file`) onto `ServeConfig`, and the `ServeSuccess` outcome
  (`main.rs`). Smoke-tested against `vyapaar-data/data` (empty catalog): binds, serves
  `GET /profile` over real HTTP, shuts down cleanly on signal. `vyapaar-ui-web`'s
  `settings`/`alerts` stores are being wired to the now-reachable `/profile` API in the
  same batch (see that repo's own HANDOFF/gaps); `watchlists`/`portfolios`/
  `risk-profiles`/`workspaces` don't fit the current `Profile` schema — tracked as
  `vyapaar-runtime/docs/gaps/0004`.

## 3. Immediate Next Steps
1. Leptos view layer for `vyapaar-ui-app` (feature state models are done; no CSR
   rendering yet).
2. TUI `run()` event loop (crossterm `event::read` + `TuiApp::on_key`).
3. Optional: replace `scan` fixture universe with a live
   `vyapaar-runtime-api-rest` `GET /screener/scan` response.
4. Optional: replace backtest `base_ts`/`generate_market_bars` fixture with real
   fill events for live parity.

## 4. Operating Invariants
- Zero `#[cfg(test)]` / `#[cfg(test)] mod tests` inside `crates/*/src/`.
- Zero floats: all financial math is `rust_decimal::Decimal`.
- WASM crates (`ui-core`, `ui-assets`, `ui-charts`, `ui-client`, `ui-api-wasm`,
  `ui-app`) must compile to `wasm32-unknown-unknown`; no `tokio` there. The
  `vyapaar-runtime-*` path deps are CLI-binary-only.
- Tests live in `<crate>/tests/` parallel to `src/`.
- Edition 2024, rust-version 1.85, AGPL-3.0.
