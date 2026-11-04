# `vyapaar-ui` — DDD Architecture & Dual Workstation Bounded Context

## 1. Bounded Context & Responsibility
`vyapaar-ui` (Tier 5A, evolved from `honba-kao`) delivers the **Dual Workstation Experience**:
1. **Browser WASM Workstation (`vyapaar-ui-app`)**: Leptos 0.8 CSR Strategy Studio visual canvas, Futuristic Neural AST Screener (`⚡ Matrix`, `🌌 Constellation`, `🧬 Tape`, `☷ Heatmap`), Backtest Workbench, and Design System Showcase (`/components`).
2. **Terminal Workstation (`vyapaar-tui` & `vyapaar-cli`)**: Single static binary terminal UI (`vyapaar tui`), interactive AI research assistant (`vyapaar ask`), and syntax highlighter (`vyapaar-highlight`).

## 2. Sub-Crate Partitioning (Strict Target Isolation)

| Sub-Crate | Target | Responsibility |
| :--- | :--- | :--- |
| `vyapaar-ui-core` | Portable (`wasm32` + native) | Design system primitives (`HfButton`, `HfBadge`, `HfChip`, `HfDrawer`, `HfTable`), screener & home view-models. Zero OS/Tokio deps. |
| `vyapaar-ui-assets` | Portable (`wasm32` + native) | Embedded themes, CSS tokens (`components.css`), fonts, audio cues, and deterministic fixtures. |
| `vyapaar-ui-client` | Portable (`wasm32` + native) | Typed REST & WebSocket streaming client state machine (`stream.rs`, `events.rs`). |
| `vyapaar-ui-charts` | Portable / WASM | Ported from `openalgo-charts-rs`: HTML5 Canvas candlestick, depth, curve, and indicator rendering engine. |
| `vyapaar-ui-api-wasm` | Portable / WASM | Zero-float `Decimal` Black-Scholes options pricing, payoff curves, margin calculator, screener AST engine, and replay bindings. |
| `vyapaar-ui-app` | WASM (`leptos 0.8 csr`) | Full browser workstation application (`features/screener`, `features/studio`, `features/simulator`, `features/system`). |
| `vyapaar-highlight` **[NEW]**| Native / Portable | Syntax highlighting token spans for strategy code, TOML manifests, and AST expressions. |
| `vyapaar-tui` **[NEW]** | Native (`ratatui`) | Terminal UI dashboard panes (Watchlist, Order Book Depth, Screener Matrix,OMS Blotter, AI Chat). |
| `vyapaar-cli` **[NEW]** | Native (`clap`) | Unified static binary entrypoint (`vyapaar ask`, `vyapaar tui`, `vyapaar scan`, `vyapaar status`). |
