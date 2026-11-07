# 0001 — CLI `ask`/`scan` wiring status

**Status**: partially closed (LLM `ask`/`chat`/`tui` resolved; `scan` open)

## What's stubbed or hardcoded

- `vyapaar-cli/src/runner.rs`'s `run_scan` returns `CliError::NotWired` pending
  headless universe integration.
- `run_ask`, `run_chat`, and `tui` AgentChat are **fully wired** and functional:
  - Supports pure Rust in-process GGUF inference (`CandleGgufProvider` powered by
    Hugging Face Candle and `shimmytok` for SmolLM2/LLaMA architectures).
  - Supports standard OpenAI-compatible endpoints (`OpenAiProvider`).
  - Supports `--model-path` / `-m` and `VYAPAAR_GGUF_PATH` environment variables.
  - Streaming token output in both CLI and Ratatui TUI.

## Why

The workarounds stand in for backends that live in other crates and are being
connected across the workspace boundary:

- `ask` / `chat` / `tui`: **Resolved**. Connected via `vyapaar-agent-llm::CandleGgufProvider`
  and `vyapaar-agent-llm::OpenAiProvider`, dynamically selected based on CLI arguments
  or environment configuration.
- `scan`: Needs a row universe + evaluator. The canonical screener wire contract
  is `vyapaar_core::screener`, and the UI's own `ScreenerFeature`/`apply_scan`
  already evaluate in-process (see `vyapaar-ui/docs/plans/0005` §2 item 2,
  resolution: build against `vyapaar_core::screener`). The *headless* CLI has no
  data source to scan, so it stays `NotWired` rather than silently running over an
  empty/fake universe.

Per `docs/plans/0005` this is a presentation-layer gap only: no domain logic is
missing, only the host-side plumbing from a running `vyapaar-runtime`/broker to
the CLI.

## How to close `scan`

1. For `scan`: parse the query into a `ScreenerScanRequest`, load a universe
   (fixture or a `vyapaar-runtime-api-rest` `GET /screener/scan` response),
   and evaluate through `vyapaar-ui-api-wasm::apply_scan` (or a live
   `vyapaar-runtime-api-rest` client).
2. Replace `CliError::NotWired` for scan with real evaluation outcomes and extend
   `tests/unit_tests.rs` accordingly.
