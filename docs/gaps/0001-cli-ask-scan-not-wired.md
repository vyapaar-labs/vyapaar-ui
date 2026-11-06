# 0001 — CLI `ask`/`scan` still `NotWired`

**Status**: open

## What's stubbed or hardcoded

`vyapaar-cli/src/runner.rs`'s `run_ask` and `run_scan` both return
`CliError::NotWired` instead of running a real turn/scan. The `tui`, `health`
subcommands are real; `ask` and `scan` fail loudly by design.

## Why

The workarounds stand in for backends that live in other repos and are not yet
reachable from this CLI without more wiring:

- `ask` needs a `vyapaar-agent` turn (chat loop + provider) reachable over the
  agent boundary, which the UI workspace does not yet inject (see
  `vyapaar-ui/docs/plans/0003`, `0004`).
- `scan` needs a row universe + evaluator. The canonical screener wire contract
  is `vyapaar_core::screener`, and the UI's own `ScreenerFeature`/`apply_scan`
  already evaluate in-process (see
  `vyapaar-ui/docs/plans/0005` §2 item 2, resolution: build against
  `vyapaar_core::screener`). The *headless* CLI has no data source to scan, so
  it stays `NotWired` rather than silently running over an empty/fake universe.

Per `docs/plans/0005` this is a presentation-layer gap only: no domain logic is
missing, only the host-side plumbing from a running `vyapaar-runtime`/broker to
the CLI.

## How to close it

1. For `scan`: parse the query into a `ScreenerScanRequest`, load a universe
   (fixture or a `vyapaar-runtime-api-rest` `GET /screener/scan` response),
   and evaluate through `vyapaar-ui-api-wasm::apply_scan` (or a live
   `vyapaar-runtime-api-rest` client).
2. For `ask`: inject an `Arc<dyn LlmProvider>` / chat-turn runner from
   `vyapaar-agent` per `vyapaar-ui/docs/plans/0003`/`0004` (OmniRoute provider),
   and drive one headless turn.
3. Replace `CliError::NotWired` returns with real outcomes and extend
   `tests/unit_tests.rs` accordingly.
