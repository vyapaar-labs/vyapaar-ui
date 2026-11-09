# Plans Index — `vyapaar-ui`

Per workspace rule `.agents/rules/gap-tracking.md`:
- Gaps are individual features (`docs/gaps/XXXX-*.md`).
- Plans are collections of gaps (`docs/plans/XXXX-*.md`).
- When implemented, plan documents are cleaned up from `docs/plans/` and recorded in this index.

## Active Plans

| # | Title | Target Scope | Status |
|---|-------|--------------|--------|
| [0002](0002-quantcept-web-ux-reference.md) | UX reference for the `AgentChat` tab, captured from a live product session | `vyapaar-ui` | Live product UX reference |
| [0003](0003-mcp-gateway-result-rendering-and-client-configs.md) | MCP Gateway UI/CLI Integration, Result Block Rendering, and Client Configs | `vyapaar-cli`, `vyapaar-tui` | Ready for implementation |

## Implemented Plans

| # | Title | Target Scope | Implemented In |
|---|-------|--------------|----------------|
| 0001 | Building `vyapaar-cli` / `vyapaar-tui` / `vyapaar-highlight` from the `artha` design work | `vyapaar-cli`, `vyapaar-tui` | Implemented — crate build-out complete (`vyapaar-highlight` lexer, `vyapaar-tui` `TuiApp`/`ActiveTab`/`run_terminal`, `vyapaar-cli` `CliArgs`/`Command` + runner). Residual `scan` wiring tracked as gap `0001`; real `AgentChat`/`Ask` agent wiring blocked upstream on `vyapaar-agent`. Doc retained because plans `0002`/`0003` list it as a "Depends on reading" reference. |
| 0004 | Full CLI Command Reference & Interactive TUI Slash Command Palette | `vyapaar-cli`, `vyapaar-tui`, `vyapaar-highlight` | Implemented (Definitive reference: `docs/reference/cli-and-tui.md`) |
| 0005 | FinceptTerminal's Screen Inventory vs. vyapaar-ui's Roadmap | `vyapaar-ui-core`, `vyapaar-ui-app`, `vyapaar-tui` | Implemented in commits `2cda245` + `acdc137` (Watchlist, Screener, Backtest, Algos, Portfolio features and 7-tab TUI workstation) |
