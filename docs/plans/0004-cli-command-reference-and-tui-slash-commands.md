# 0004 — Full CLI Command Reference & Interactive TUI Slash Command Palette

> **Status**: Ready for implementation.
> **Target Crates**: `vyapaar-cli`, `vyapaar-tui`, `vyapaar-highlight`.
> **Depends on reading**:
> - `vyapaar-ui/docs/plans/0001-agent-chat-terminal-from-artha.md`
> - `vyapaar-ui/docs/plans/0002-quantcept-web-ux-reference.md`
> - `vyapaar-ui/docs/plans/0003-mcp-gateway-result-rendering-and-client-configs.md`
> - `vyapaar-agent/docs/plans/0002-autonomous-jobs-memory-and-strategy-verification-engine.md`
> - Reference documentation:
>   - Quantcept CLI Reference (`https://quantcept.io/docs/reference/cli`)
>   - Quantcept Slash Commands (`https://quantcept.io/docs/cli/commands`)

---

## 0. Executive Summary & Objective

Quantcept establishes a two-layer interface model for quantitative workstations:
1. **Headless & Operating System CLI**: High-throughput terminal subcommands (`ask`, `jobs`, `plugin`, `memory`, `verify`, `doctor`, `purge`) suitable for cron tasks, pipeline scripts, and non-interactive developer workflows.
2. **Interactive TUI Slash Commands**: In-terminal modal palette and quick-actions (`/help`, `/agent`, `/memory`, `/jobs`, `/checkpoints`, `/undo`, `/export`, `/positions`) operable via `/` or `Ctrl+P`.

This document specifies the exact mapping, argument structures, and implementation plan for bringing the full CLI and Slash Command surface into `vyapaar-cli` and `vyapaar-tui`.

---

## 1. System Architecture

```
┌────────────────────────────────────────────────────────────────────────┐
│                        User Interaction Surface                        │
├───────────────────────────────────┬────────────────────────────────────┤
│           vyapaar-cli             │            vyapaar-tui             │
│   (Headless Binary Commands)      │    (Interactive Multi-Tab TUI)     │
│                                   │                                    │
│  • vyapaar [message] [--continue] │  • Command Palette (/ or Ctrl+P)   │
│  • vyapaar ask <prompt>           │  • In-Session Slash Commands       │
│  • vyapaar jobs {add,list,run...} │    - /agent <name|off|default>     │
│  • vyapaar verify [file]          │    - /undo, /redo, /checkpoints    │
│  • vyapaar memory {tick,stats...} │    - /export <json|txt>            │
│  • vyapaar plugin / skills        │    - /jobs, /positions, /memory    │
│  • vyapaar doctor [--fix]         │    - /theme <dark|light>           │
│  • vyapaar purge [--yes]          │  • Autocomplete & Modal Navigation │
└─────────────────┬─────────────────┴──────────────────┬─────────────────┘
                  │                                    │
┌─────────────────▼────────────────────────────────────▼─────────────────┐
│               Command Router & Dispatcher (vyapaar-cli)                │
└─────────────────┬────────────────────────────────────┬─────────────────┘
                  │                                    │
┌─────────────────▼──────────────┐   ┌─────────────────▼─────────────────┐
│       vyapaar-agent-chat       │   │       vyapaar-agent-core          │
│   • Headless Turn Runner       │   │   • Job Scheduler & Approvals     │
│   • Persona Prompt Injection   │   │   • Local Memory Vector Store     │
│   • Checkpoint/Undo Engine     │   │   • Strategy Verification Engine  │
└────────────────────────────────┘   └───────────────────────────────────┘
```

---

## 2. CLI Binary Command Surface (`vyapaar-cli`)

`vyapaar-cli` acts as the unified binary entrypoint (`vyapaar`). Global options apply before any subcommand:

```bash
vyapaar [OPTIONS] [COMMAND]

Global Options:
  --log-level <debug|info|warn|error|silent>  [env: VYAPAAR_LOG_LEVEL]
  -v, --verbose                              Shortcut for --log-level debug
  -q, --quiet                                Shortcut for --log-level silent
  -h, --help                                 Print help
  -V, --version                              Print version
```

### 2.1 Interactive Terminal Launch
```bash
vyapaar [message]
vyapaar --continue, -c           # Resumes latest session in current directory
vyapaar --resume, -r [session_id]# Resumes specific session or opens session picker
vyapaar --skip-permissions       # Turns on session auto-accept (risk gate still strictly enforced)
```

### 2.2 Headless Turn (`ask`)
```bash
vyapaar ask <prompt> [OPTIONS]

Options:
  --persona <name>          Specialized financial persona (quant, analyst, trader, etc.)
  --mode <paper|backtest>   Runtime execution mode (default: paper)
  --profile <name>          Configuration profile (default: default)
  --timeout-min <N>         Maximum timeout in minutes (default: 20)
  --format <text|json>      Output format (default: text)
```
- Standard output receives the final synthesized answer.
- Standard error receives progress, tool dispatch logs, and execution traces.

### 2.3 Scheduled Autonomous Jobs (`jobs`)
Enables autonomous research and market monitoring that runs without manual supervision:
```bash
vyapaar jobs add <goal...> [OPTIONS]
  Options:
    --schedule <cron>       Standard 5-field cron expression (e.g. "0 9 * * 1-5")
    --max-turns <N>         Maximum conversation turns per run (default: 20)
    --max-inr <N>           Maximum statutory/data cost budget in INR
    --max-tokens <N>        Maximum token consumption budget
    --max-tool-calls <N>    Maximum tool executions permitted
    --missed-policy <skip|catch_up>
    --read-only             Enforce pure advisory mode (default: true)
    --once                  Run immediately once and exit

vyapaar jobs list           # Lists registered jobs with status, next run, and budgets
vyapaar jobs run <id>       # Manually triggers an immediate job run
vyapaar jobs tick [--all]   # Evaluates cron schedules against system time (called by systemd/cron)
vyapaar jobs install        # Prints systemd service or crontab snippet
vyapaar jobs logs <id>      # Streams execution history and tool traces for a job
vyapaar jobs pause <id>     # Pauses job execution
vyapaar jobs resume <id>    # Resumes a paused job
vyapaar jobs approvals      # Lists pending actions requiring human confirmation
vyapaar jobs approve <id>   # Confirms a pending action
vyapaar jobs deny <id>      # Rejects a pending action
```

### 2.4 Strategy Verification (`verify`)
Static verification of strategy manifests and Python/Rust algorithms to prevent catastrophic backtest bias:
```bash
vyapaar verify <file> [--strict]

Checks performed:
  1. Lookahead Bias: Detects future candle indexing or unlagged close price access.
  2. Data Leakage: Detects indicator/scaler fitting before train/test split.
  3. NoFX Violation: Detects direct socket/broker calls bypassing vyapaar-risk.
  Exit code 0: Verification passed.
  Exit code 1: Bias or leak detected (or warnings when --strict is passed).
  Exit code 2: File missing or syntax error.
```

### 2.5 Local Knowledge & Memory (`memory`)
```bash
vyapaar memory tick         # Condenses recent chat transcripts into semantic memory facts
vyapaar memory stats        # Displays memory index statistics (notes, vectors, disk size)
vyapaar memory import <dir> # Ingests external research documents, notes, or PDFs
vyapaar memory install      # Prints systemd timer snippet for periodic memory distillation
```

### 2.6 Environment Diagnostics & Repair (`doctor`)
```bash
vyapaar doctor [--fix] [--json]

Diagnostics:
  • Lakehouse & Parquet data path accessibility
  • SEBI pre-trade risk engine configuration
  • Local SQLite database integrity (sessions, memory, ledger)
  • Python virtual environment & quantitative dependencies
  • Active broker API credentials & connectivity
  Flag --fix: Automatically provisions missing directories, SQLite tables, and dependencies.
```

### 2.7 State Wipe (`purge`)
```bash
vyapaar purge [--dry-run] [--yes]
  Removes ephemeral caches, session transcripts, and temporary data references.
  Refuses to run non-interactively without --yes.
  Never deletes user configuration, strategy source code, or signed ledger entries.
```

---

## 3. Interactive TUI Slash Command Palette (`vyapaar-tui`)

Within `vyapaar-tui`, pressing `/` in the prompt composer or `Ctrl+P` anywhere opens the fuzzy-searchable **Slash Command Palette**.

```
┌────────────────────────────────────────────────────────┐
│ Command Palette (Ctrl+P or /)                          │
├────────────────────────────────────────────────────────┤
│ > /a                                                   │
│   /agent          Switch or inspect active persona     │
│   /artifacts      Browse generated reports & charts    │
│   /auto           Toggle automatic tool approval       │
│   /account        Inspect profile and risk boundaries  │
├────────────────────────────────────────────────────────┤
│ [Enter] Select   [Tab] Autocomplete   [Esc] Dismiss    │
└────────────────────────────────────────────────────────┘
```

### 3.1 Complete Slash Command Catalog

| Command | Arguments | Scope | Functionality |
| :--- | :--- | :--- | :--- |
| **`/help`** | `[command]` | Global | Displays command reference or detailed syntax for a specific command. |
| **`/agent`** | `<name\|off\|default>` | Session | Switches the active persona (`quant`, `analyst`, `trader`, etc.) or resets to base. |
| **`/new`** | None | Global | Archives the current turn context and starts a fresh conversation session. |
| **`/resume`** | `[session_id]` | Global | Opens the session history picker to switch conversations. |
| **`/clear`** | None | Session | Clears visual scrollback history from the active terminal buffer. |
| **`/copy`** | `[full\|tools]` | Session | Copies the latest answer, full turn transcript, or tool outputs to clipboard. |
| **`/export`** | `<json\|txt>` | Session | Exports the conversation transcript to `~/Downloads/vyapaar/`. |
| **`/undo`** | None | Session | Reverts the last conversation turn, removing it from context. |
| **`/redo`** | None | Session | Restores a previously undone conversation turn. |
| **`/checkpoints`**| None | Session | Lists and restores saved turn checkpoints within the session. |
| **`/auto`** | None | Session | Toggles auto-approval for low-risk advisory tools. |
| **`/theme`** | `<dark\|light>` | Global | Switches the UI color palette between Dark and Light mode. |
| **`/perf`** | None | Global | Displays CPU, memory usage, and turn inference latency statistics. |
| **`/context`** | None | Session | Visualizes current context token budget (system prompt, tools, memory, history). |
| **`/remember`** | `<fact>` | Global | Immediately saves a permanent knowledge fact to local memory. |
| **`/memory`** | None | Global | Opens the interactive memory inspector modal (browse notes, edit, delete with `d`).|
| **`/jobs`** | None | Global | Opens the interactive background jobs manager (view schedules, run, edit, delete).|
| **`/positions`** | None | Global | Opens active paper trading positions, open orders, and PnL breakdown. |
| **`/skills`** | None | Global | Lists installed financial skills and allowed tool sets. |
| **`/mcp`** | None | Global | Lists connected MCP servers or live-connects a new server. |
| **`/verify`** | `[file]` | Session | Triggers in-session lookahead bias scan on a strategy file. |
| **`/quit`, `/exit`**| None | Global | Closes the TUI cleanly, preserving session state. |

### 3.2 Modal Keyboard Navigation Rules
- **`Up` / `Down`**: Navigate selection lists within modals.
- **`Enter`**: Confirm selection or advance form field.
- **`Esc`**: Dismiss active modal or step back to the main workstation tab.
- **`Tab`**: Autocomplete highlighted slash command.
- **`d`**: Delete highlighted item (inside `/jobs`, `/memory`, or `/checkpoints`).
- **`e`**: Edit highlighted item.

---

## 4. Implementation Phases

### Phase 1: `vyapaar-cli` Clap Command Tree
- Refactor `crates/vyapaar-cli/src/lib.rs` to replace the 5-command stub with the full `Command` enum:
  `Tui`, `Ask`, `Jobs`, `Verify`, `Memory`, `Plugin`, `Skills`, `Doctor`, `Purge`, `Mcp`, `Scan`.
- Connect each subcommand to corresponding backend service ports.

### Phase 2: `vyapaar-tui` Palette & Slash Command Parser
- Implement `SlashCommand` enum and tokenizer in `crates/vyapaar-tui/src/slash.rs`.
- Build the `PaletteWidget` popup overlay rendered above the active tab when `/` or `Ctrl+P` is active.
- Wire modal hotkeys (`Up`, `Down`, `Enter`, `Esc`, `Tab`, `d`).

### Phase 3: Headless Execution & Output Formatters
- Support `--format json` and `--quiet` across all CLI subcommands for headless scripting.
- Implement ANSI progress bars and stderr chatter separation in `vyapaar ask`.

### Phase 4: Integration & Conformance Tests
- Write automated tests in `crates/vyapaar-cli/tests/` for CLI flag parsing and exit codes.
- Test TUI slash command handling in `crates/vyapaar-tui/tests/`.
