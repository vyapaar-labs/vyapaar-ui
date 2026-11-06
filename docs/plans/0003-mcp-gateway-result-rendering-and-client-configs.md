# 0003 — MCP Gateway UI/CLI Integration, Result Block Rendering, and Client Configs

> **Status**: Ready for implementation.
> **Target Crates**: `vyapaar-cli`, `vyapaar-tui`, `vyapaar-highlight`.
> **Depends on reading**:
> - `vyapaar-ui/docs/plans/0001-agent-chat-terminal-from-artha.md`
> - `vyapaar-ui/docs/plans/0002-quantcept-web-ux-reference.md`
> - `vyapaar-agent/docs/plans/0001-mcp-gateway-discovery-and-quantitative-engines.md`

---

## 0. Executive Summary & Objective

With `vyapaar-agent` establishing a two-tier discovery gateway, universal `vyapaar/result/v1` envelope, and 15 quantitative engines, `vyapaar-ui` must deliver the presentation and operator interface for these tools:
1. **Visual Presentation Block Rendering**: Turning `DisplayBlock`s (`Table`, `MetricGrid`, `Chart`/sparkline, `Alert`) into ANSI-colored terminal output in `vyapaar-cli` and Ratatui widgets in `vyapaar-tui`.
2. **Tool Gateway CLI Commands**: Providing power users with direct inspection tools (`vyapaar tools search`, `vyapaar tools describe`, `vyapaar tools call`).
3. **One-Click MCP Client Config Exporters**: Generating ready-to-run configuration snippets for external coding agents (Claude Code, Cursor, Copilot/VS Code, Codex) that connect directly to the local `vyapaar-cli mcp` stdio daemon.

---

## 1. UI & Terminal Interaction Architecture

```
┌────────────────────────────────────────────────────────────────────────┐
│                          vyapaar-ui Surface                            │
├───────────────────────────────────┬────────────────────────────────────┤
│         vyapaar-cli               │            vyapaar-tui             │
│                                   │                                    │
│  • vyapaar tools search           │  • AgentChat Tab                   │
│  • vyapaar tools describe         │    - Streaming token markdown      │
│  • vyapaar tools call             │    - Collapsible Tool Execution    │
│  • vyapaar mcp config             │    - Embedded Ratatui Tables       │
│  • vyapaar ask --persona <p>      │    - Unicode Sparkline Bar Charts  │
└─────────────────┬─────────────────┴──────────────────┬─────────────────┘
                  │                                    │
┌─────────────────▼────────────────────────────────────▼─────────────────┐
│                     Block Presentation Engine                          │
│                                                                        │
│   • TableRenderer: ANSI borders, aligned headers, zero-float decimals  │
│   • MetricGridRenderer: Side-by-side KPI boxes with delta colors       │
│   • SparklineRenderer: Unicode block characters ( ▂▃▄▅▆▇█)             │
│   • AlertRenderer: Styled badges ([INFO], [WARN], [CRITICAL])          │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │ Parses
┌───────────────────────────────────▼────────────────────────────────────┐
│              Universal Result Protocol (vyapaar/result/v1)             │
│                Emitted by vyapaar-agent-chat / MCP                     │
└────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Component Specifications

### 2.1 Result Envelope Block Renderers

Tools returning `vyapaar/result/v1` contain an optional `blocks` array. `vyapaar-ui` renders each block type cleanly:

#### 1. `DisplayBlock::Table`
- **CLI**: Rendered using UTF-8 box-drawing characters (`┌─┬─┐`, `│ │ │`, `└─┴─┘`), with right-aligned numeric columns.
- **TUI**: Rendered inside `ratatui::widgets::Table` with selectable row highlight, scrollbar, and fixed header.

#### 2. `DisplayBlock::MetricGrid`
- Renders KPI cards (e.g., Sharpe Ratio: `1.84`, Max Drawdown: `-8.20%`, Annualized Return: `28.4%`).
- Value coloring: Green for positive return/Sharpe, Red for drawdown/VaR breach, Cyan for neutral.

#### 3. `DisplayBlock::Chart`
- **CLI**: Renders high-density Unicode sparklines (` ▂▃▅▆▇█`) for quick 1-line trend visualization.
- **TUI**: Renders `ratatui::widgets::Sparkline` or multi-series `ratatui::widgets::Chart` directly inside the AgentChat turn view.

#### 4. `DisplayBlock::Alert`
- Styled warning boxes for risk limit warnings, margin shortfall alerts, or statutory cost notices.

---

### 2.2 CLI Tool Gateway Commands

Add a new top-level `tools` command family to `vyapaar-cli`:

```bash
# 1. Search across all 15 engines
vyapaar tools search "GARCH volatility"
# Output:
#   Found 2 matching tools:
#   - vyapaar_volatility_fit_garch (volatility): Estimate GARCH(1,1)/EGARCH models
#   - vyapaar_volatility_forecast_var (volatility): Value-at-Risk & ES out-of-sample forecast

# 2. Inspect exact schema and usage
vyapaar tools describe vyapaar_volatility_fit_garch
# Output:
#   Tool: vyapaar_volatility_fit_garch
#   Category: volatility
#   Description: Fits univariate GARCH-family conditional volatility models...
#   Parameters:
#     - series_ref (string, required): Ref ID ($vyapaar/ref/...) or inline array
#     - model (enum: "garch", "egarch", "gjr"): Default "garch"
#     - p (integer): Default 1
#     - q (integer): Default 1

# 3. Direct invocation
vyapaar tools call vyapaar_calc_statutory_costs '{"segment":"equity_delivery","turnover":"100000","side":"sell"}'
```

---

### 2.3 One-Click MCP Client Configuration Exporter

Coding agents (Claude Code, Cursor, Codex, VS Code) require an MCP config file pointing to the server. Add `vyapaar mcp config`:

```bash
# Generate configuration for Claude Code
vyapaar mcp config --client claude-code

# Generate configuration for Cursor
vyapaar mcp config --client cursor

# Generate configuration for VS Code (GitHub Copilot Agent mode)
vyapaar mcp config --client vscode
```

#### Generated Cursor Configuration (`.cursor/mcp.json`):
```json
{
  "mcpServers": {
    "vyapaar": {
      "command": "vyapaar",
      "args": ["mcp", "--stdio"],
      "env": {
        "VYAPAAR_LOG": "info",
        "VYAPAAR_WORKSPACE": "/path/to/workspace"
      }
    }
  }
}
```

#### Generated Claude Code Configuration (`~/.claude.json`):
```json
{
  "mcpServers": {
    "vyapaar": {
      "command": "vyapaar",
      "args": ["mcp", "--stdio"]
    }
  }
}
```

---

## 3. Implementation Phases

### Phase 1: Block Presentation Engine (`crates/vyapaar-tui` & `crates/vyapaar-cli`)
- Create `block_render.rs` in `vyapaar-cli` implementing ANSI table, metric, and sparkline rendering.
- Add `ResultBlockWidget` in `crates/vyapaar-tui/src/widgets/` for in-TUI rendering.

### Phase 2: Gateway CLI Commands (`crates/vyapaar-cli/src/lib.rs`)
- Add `Command::Tools { action: ToolsAction }` with subcommands `search`, `describe`, and `call`.
- Connect calls to `vyapaar-agent-mcp` gateway services.

### Phase 3: MCP Config Generator (`crates/vyapaar-cli/src/mcp_config.rs`)
- Implement `Command::Mcp { subcommand: McpCommand::Config { client } }`.
- Validate generated paths and permissions.

### Phase 4: Full End-to-End Turn Test
- Unit test verifying `vyapaar ask` correctly formats and prints nested result envelope blocks.
- Unit test verifying `vyapaar tools search` and `vyapaar tools describe` output.
