# `vyapaar-ui` — Inter-Agent Handoff (Layer 5)

> Updated: 2026-11-09 — PLAN-0001 Stage 4 (CLI & agent jobs unification) complete
> alongside the earlier Layer 5 / Tier 5 work below.

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
  - `cargo test --workspace --profile prod` ✅ 59 targets, 264 passed, 0 failed
    (`vyapaar-cli` covers the 48: 11 Stage 4 e2e + 12 strategy/run + 25 unit)
  - `cargo fmt` ✅ on all Stage 4 touched files (pre-existing drift in
    `vyapaar-tui` / `vyapaar-ui-core` committed files predates this batch and is
    out of scope here)
  - `make prod` ✅ fresh `target/prod`

## 2. Decisions & Discoveries
- CLI `ask`/`chat`/`tui` wired to LLM backends (GGUF in-process / OpenAI
  endpoints); `scan` remains open — see `docs/gaps/0001-cli-ask-scan-not-wired.md`.
- `trades.json` for job runs records only `{"total_trades": N}` because
  `vyapaar-ui-core::strategy::BacktestReport` carries aggregates only — filed as
  `docs/gaps/0002-ui-core-backtest-no-trade-rows.md` (cited in code).
- Registry ops order per run: claim → pool dispatch → release_lease →
  record_usage (breach/manual wins over lease) → advance cron cursor; cron
  evaluation via the runtime `JobScheduler` (`cron` crate, Vixie dom/dow OR).
- Determinism: all job/manifest timestamps are explicit `DateTime<Utc>` passed
  from the caller; journal events carry only seq + type + engine output.
- Zero `#[cfg(test)]` inside `crates/*/src/`; tests live in `<crate>/tests/`.

## 3. Immediate Next Steps
1. Wire remaining CLI `scan` to the canonical screener (close gap `0001`).
2. Leptos view layer for `vyapaar-ui-app` (feature state models are done; no CSR
   rendering yet).
3. Emit trade rows from `vyapaar-ui-core::strategy::simulate_strategy` (close
   gap `0002`) so job journals become replayable tax-lot evidence.
4. TUI `run()` event loop (crossterm `event::read` + `TuiApp::on_key`).

## 4. Operating Invariants
- Zero `#[cfg(test)]` / `#[cfg(test)] mod tests` inside `crates/*/src/`.
- Zero floats: all financial math is `rust_decimal::Decimal`.
- WASM crates (`ui-core`, `ui-assets`, `ui-charts`, `ui-client`, `ui-api-wasm`,
  `ui-app`) must compile to `wasm32-unknown-unknown`; no `tokio` there. The
  `vyapaar-runtime-*` path deps are CLI-binary-only.
- Tests live in `<crate>/tests/` parallel to `src/`.
- Edition 2024, rust-version 1.85, AGPL-3.0.
