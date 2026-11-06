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
- `vyapaar-tui` — Ratatui-based terminal dashboard
- `vyapaar-cli` — Command line interface (`vyapaar` CLI)
- `vyapaar-highlight` — Zero-I/O WASM-safe syntax highlighter (hand-rolled lexer, no tree-sitter)

## Build & Test

```bash
make prod
make check
make test
```

License: AGPL-3.0.
