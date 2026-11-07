# व्यापार・界面 vyapaar-ui

> *Unified Workstations: WASM/Native Shell Strategy Studio, Screener, Ratatui Terminal UI, CLI binary, and syntax highlighting.*

Trader and quantitative workstation interfaces across terminal and browser.

## Subcrates

- `vyapaar-ui` — Workspace facade
- `vyapaar-ui-core` — Common UI state, theme definitions, and view models
- `vyapaar-ui-app` — Desktop & WASM canvas strategy studio
- `vyapaar-ui-charts` — High-performance Rust financial charting primitives
- `vyapaar-ui-client` — WebSocket & REST client connection to `vyapaar-runtime`
- `vyapaar-ui-assets` — Embedded fonts, icons, and stylesheets
- `vyapaar-ui-api-wasm` — WASM bridge bindings for web integration
- `vyapaar-tui` — Ratatui-based terminal dashboard with interactive LLM Agent chat
- `vyapaar-cli` — Command line interface (`vyapaar` CLI) with one-shot ask & interactive chat
- `vyapaar-highlight` — Zero-I/O WASM-safe syntax highlighter (hand-rolled lexer, no tree-sitter)

## Local LLM / SmolLM2 Integration (CLI & TUI)

`vyapaar-cli` and `vyapaar-tui` support pure Rust in-process GGUF inference (no external Python or daemon processes required) using Hugging Face Candle and `shimmytok`:

```bash
# 1. Download SmolLM2-360M (default ~230MB) or 1.7B GGUF:
./scripts/download-smollm.sh 360m

# 2. Run one-shot CLI query:
vyapaar ask "Explain VWAP vs TWAP" --model-path ./models/SmolLM2-360M-Instruct-Q4_K_M.gguf

# 3. Interactive terminal chat REPL:
vyapaar chat --model-path ./models/SmolLM2-360M-Instruct-Q4_K_M.gguf

# 4. Interactive full-screen Ratatui terminal workstation (tab 2: AgentChat):
vyapaar tui --model-path ./models/SmolLM2-360M-Instruct-Q4_K_M.gguf

# Or configure via environment variable:
export VYAPAAR_GGUF_PATH="./models/SmolLM2-360M-Instruct-Q4_K_M.gguf"
vyapaar chat
```

## Build & Test

```bash
make prod
make check
make test
```

License: AGPL-3.0.
