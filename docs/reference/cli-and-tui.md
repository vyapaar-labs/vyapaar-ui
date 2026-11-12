# Vyapaar Workstation: CLI & TUI Slash Commands Reference

> **Standards Alignment**: Quantcept CLI Specification ([CLI Reference](https://quantcept.io/docs/reference/cli), [Slash Commands](https://quantcept.io/docs/cli/commands))  
> **Crates**: [`vyapaar-cli`](../../crates/vyapaar-cli), [`vyapaar-tui`](../../crates/vyapaar-tui), [`vyapaar-highlight`](../../crates/vyapaar-highlight)  
> **Target Platforms**: Linux, macOS, Windows (native binary `vyapaar`)

---

## 1. System Overview & Dual Interface Architecture

Vyapaar establishes a dual-layer interface architecture for Indian equities quantitative analysis, risk management, and algorithmic trading:

```
┌────────────────────────────────────────────────────────────────────────┐
│                        User Interaction Surface                        │
├───────────────────────────────────┬────────────────────────────────────┤
│           vyapaar-cli             │            vyapaar-tui             │
│   (Headless Binary Commands)      │    (Interactive Multi-Tab TUI)     │
│                                   │                                    │
│  • vyapaar [message] [--continue] │  • Command Palette (/ or Ctrl+P)   │
│  • vyapaar ask <prompt>           │  • In-Session Slash Commands (40)  │
│  • vyapaar jobs {add,list,run...} │    - /agent <name|off|default>     │
│  • vyapaar verify [file]          │    - /undo, /redo, /checkpoints    │
│  • vyapaar memory {tick,stats...} │    - /export <json|txt>            │
│  • vyapaar plugin & marketplace   │    - /jobs, /positions, /memory    │
│  • vyapaar skills & mcp           │    - /theme <dark|light>           │
│  • vyapaar doctor [--fix]         │    - /perf, /context, /unbind      │
│  • vyapaar purge [--yes]          │  • Autocomplete & Modal Hotkeys    │
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
│   • Candle GGUF / OpenAI bridge│   │   • SEBI Pre-Trade Risk Gate (<15µs│
└────────────────────────────────┘   └───────────────────────────────────┘
```

1. **Headless CLI Binary (`vyapaar-cli`)**: High-throughput terminal subcommands (`ask`, `jobs`, `verify`, `memory`, `plugin`, `skills`, `mcp`, `doctor`, `purge`) optimized for cron jobs, CI/CD verification pipelines, and scripting.
2. **Interactive TUI Workstation (`vyapaar-tui`)**: Ratatui terminal workstation featuring 7 core tabs (`Watchlist`, `Screener`, `Portfolio`, `Orders`, `Backtest`, `Algos`, `AgentChat`) and a modal slash command palette accessible via `/` or `Ctrl+P`.

---

## 2. Global Options & Environment Precedence

The global options apply to all subcommands before argument dispatch:

```bash
vyapaar [OPTIONS] [COMMAND]

Global Options:
  -m, --model-path <PATH>      Path to local GGUF model file (SmolLM2, DeepSeek-R1, etc.)
  -t, --temperature <FLOAT>    Sampling temperature (0.0 to 1.0)
      --log-level <LEVEL>      Log level: debug | info | warn | error | silent
  -v, --verbose                Shortcut for --log-level debug
  -q, --quiet                  Shortcut for --log-level silent
      --skip-permissions       Auto-accept tool confirmation prompts
                               (Aliases: --dangerously-skip-permissions, --yolo)
  -c, --continue               Resume latest session in current directory
  -r, --resume <SESSION_ID>    Resume specific session by ID or open picker
  -h, --help                   Print help information
  -V, --version                Print version information
```

### Log Level Resolution Precedence
Quantcept requires strict log level resolution before argument processing so flags are not consumed as job goals or prompts:
1. `VYAPAAR_LOG_LEVEL` or `QUANTCEPT_LOG_LEVEL` environment variable.
2. Explicit CLI flag: `--log-level <debug|info|warn|error|silent>`.
3. Shorthand flags: `-v` / `--verbose` (`debug`), `-q` / `--quiet` (`silent`).
4. Local settings file (`~/.vyapaar/settings.json`).
5. Built-in default: `info`.

---

## 3. Headless CLI Command Reference (`vyapaar-cli`)

### 3.1 Terminal Workstation Launch
```bash
# Launch interactive TUI on default Watchlist tab
vyapaar

# Launch interactive TUI directly with an initial question (focuses Agent tab)
vyapaar "Analyze NIFTY 50 intraday momentum and call-put ratio"

# Resume the latest conversation session in this workspace
vyapaar --continue
vyapaar -c

# Resume a specific session checkpoint
vyapaar --resume chk-002
vyapaar -r chk-002

# Auto-approve advisory tool calls for the current process
vyapaar --skip-permissions
vyapaar --dangerously-skip-permissions
vyapaar --yolo
```

> [!NOTE]
> `--skip-permissions` turns on auto-accept for tool prompts in that specific process. It does not alter persistent `settings.json`, nor can it bypass SEBI pre-trade risk gates or mandatory broker order rules.

---

### 3.2 Single-Turn Question Answering (`vyapaar ask`)
Executes a single conversational or quantitative turn without opening the interactive terminal UI.

```bash
vyapaar ask <prompt> [OPTIONS]

Options:
  --persona <name>        Specialized financial desk (quant, analyst, trader, etc.)
  --mode <paper|backtest> Execution mode (default: paper)
  --profile <name>        Configuration profile (default: default)
  --timeout-min <N>       Execution timeout in minutes (default: 20)
  --format <text|json>    Output format (default: text)
```

**Stream Separation Rule**:
- **`stdout`**: Receives only the final synthesized response text (or machine-readable JSON object when `--format json` is passed).
- **`stderr`**: Receives initialization messages, model checkpoints, tool dispatches, and progress indicators.

#### JSON Output Shape (`--format json`)
```json
{
  "prompt": "Calculate Sharpe ratio for the turtle breakout strategy",
  "answer": "The strategy yields a Sharpe ratio of 1.84 with max drawdown of 9.2%...",
  "persona": "quant",
  "mode": "paper",
  "timeout_min": 20,
  "model": "SmolLM2-360M-Instruct (Candle GGUF)"
}
```

---

### 3.3 Autonomous Scheduled Jobs (`vyapaar jobs`)
Enables autonomous research, morning briefs, and market scanning without manual supervision.

```bash
# Default command lists all jobs
vyapaar jobs

# Add an autonomous scheduled research job
vyapaar jobs add <goal...> [OPTIONS]
  Options:
    --schedule <cron|json>      Cron expression (e.g. "30 9 * * 1-5") or schedule JSON
    --max-turns <N>             Turn cap per run (default: 20)
    --max-inr <N>               Statutory/compute budget cap in INR
    --max-usd <N>               Compute budget cap in USD
    --max-tokens <N>            Token consumption budget cap
    --max-tool-calls <N>        Maximum allowed tool invocations
    --max-data-calls <N>        Maximum allowed market data requests
    --missed-policy <skip|catch_up>
    --success <json>            Success evaluation criteria JSON
    --read-only                 Enforce read-only advisory mode (default: true)
    --once                      Execute immediately once and exit

# List all active and scheduled jobs
vyapaar jobs list

# Immediately trigger execution of a job by ID
vyapaar jobs run <id>

# Evaluate cron schedules against current system time (called by systemd/cron)
vyapaar jobs tick [--all]

# Print scheduler integration snippet (systemd timer or crontab)
vyapaar jobs install

# Display execution logs and tool traces for a job
vyapaar jobs logs <id>

# Pause or resume a job
vyapaar jobs pause <id>
vyapaar jobs resume <id>

# Human-in-the-loop trade approval queue
vyapaar jobs approvals
vyapaar jobs approve <id>
vyapaar jobs deny <id>
```

- `add` generates and prints an **8-character hexadecimal ID** (e.g. `5ecc84e6`).
- `tick` without `--all` evaluates only jobs registered in the current directory workspace.
- `install` outputs cron and `systemd` user service snippets without modifying system configuration.

---

### 3.4 Strategy Verification (`vyapaar verify`)
Performs static AST analysis on strategy source files to detect backtest biases and leakage prior to paper or live execution.

```bash
vyapaar verify [file] [--strict]
```

**Checks Performed**:
1. **Lookahead Bias Detection**:
   - Detects future candle indexing or lead function access (e.g. `.shift(-k)`, `shift(-1)`, `[t+1]`, `[i+1]`, `.lead()`).
2. **Data Leakage Detection**:
   - Detects scaler or indicator fitting prior to train/test split (e.g. `.fit()` called globally without split boundaries).
3. **SEBI Pre-Trade Compliance**:
   - Verifies orders route through `vyapaar-risk` sizing and throttle gates.

**Exit Code Contract**:
- `0`: Verification passed cleanly (no lookahead bias).
- `1`: Lookahead bias detected, or data leakage warning encountered with `--strict`.
- `2`: File missing or unreadable.

---

### 3.5 Local Semantic Memory (`vyapaar memory`)
Maintains an in-process semantic vector index (`~/.vyapaar/memory.db`) using 384-dimensional embeddings.

```bash
# Default command prints stats
vyapaar memory

# Distill recent chat transcripts into semantic memory facts
vyapaar memory tick

# Display memory index statistics (indexed notes, vector size, disk storage)
vyapaar memory stats

# Display or configure active embedding model
vyapaar memory model [name]

# Import research documents, PDFs, or Markdown into memory
vyapaar memory import [path]

# Print a 30-minute systemd distillation timer snippet
vyapaar memory install
```

---

### 3.6 Plugins & Marketplace (`vyapaar plugin`)
Manages modular tools, extensions, and marketplace feeds:

```bash
# List installed plugins
vyapaar plugin
vyapaar plugin list

# Install, uninstall, enable, or disable plugins
vyapaar plugin install <name@marketplace|source>
vyapaar plugin uninstall <name>
vyapaar plugin enable <name>
vyapaar plugin disable <name>

# Validate plugin manifest at given path (defaults to current dir; exit 1 on error)
vyapaar plugin validate [path]

# Manage marketplace sources
vyapaar plugin marketplace add <source>
vyapaar plugin marketplace list
vyapaar plugin marketplace remove <name>
vyapaar plugin marketplace update [name]
```

---

### 3.7 Financial Skills Suites (`vyapaar skills`)
Lists bundled specialized quantitative and trading skills:

```bash
vyapaar skills
vyapaar skills list
vyapaar skills show <name>
```

**The 8 Bundled Desks**:
- `vyapaar-ai`: The all-round coordinator.
- `quant`: Statistical factor models, backtesting, empirical validation.
- `analyst`: Fundamental research, financial statement analysis, earnings quality.
- `trader`: Price action, intraday support/resistance, order flow execution.
- `risk-manager`: Downside protection, stress testing, VaR, position sizing.
- `portfolio-manager`: Multi-asset allocation, factor drift, rebalancing.
- `macro-strategist`: Macro regimes, RBI monetary policy, bond yield curves.
- `value-investor`: Moats, owner earnings, intrinsic value, margin of safety.
- `devils-advocate`: Rigorous bear cases, counter-theses, hidden structural risks.

---

### 3.8 Model Context Protocol (`vyapaar mcp`)
Connects external MCP server tools into the workstation:

```bash
vyapaar mcp
vyapaar mcp list
vyapaar mcp connect <name> <command>
```

---

### 3.9 Diagnostics & Auto-Repair (`vyapaar doctor`)
Verifies workstation environment health, Lakehouse storage paths, and sidecars:

```bash
vyapaar doctor
vyapaar doctor --fix
vyapaar doctor --json
```

- `--fix`: Automatically provisions missing directories (`~/.vyapaar/lakehouse`, `~/.vyapaar/cache`) and SQLite tables.
- `--json`: Emits machine-readable health diagnostics.

---

### 3.10 State Purge (`vyapaar purge`)
Removes ephemeral cache files, session transcripts, and temporary data.

```bash
# Preview files to be deleted without removing
vyapaar purge --dry-run

# Confirm permanent non-interactive cleanup
vyapaar purge --yes
```

> [!CAUTION]
> `vyapaar purge` deletes `~/.vyapaar/cache`, `~/.vyapaar/tmp`, and `~/.learnings`. It strictly refuses to execute without `--yes` or `--dry-run`. It never deletes strategy source files, ledger entries, or user configuration.

---

### 3.11 Summary Table of CLI Exit Codes

| Exit Code | Meaning | Conditions |
| :---: | :--- | :--- |
| **`0`** | Success | Command executed successfully; verification passed cleanly. |
| **`1`** | General Failure | Lookahead bias detected; strict verification failure; missing required parameters; invalid plugin manifest. |
| **`2`** | Missing File | Target strategy or import path does not exist on disk. |

---

## 4. Interactive TUI Slash Command Palette (`vyapaar-tui`)

Within `vyapaar-tui`, pressing `/` anywhere or `Ctrl+P` opens the **Command Palette**.

```
┌────────────────────────────────────────────────────────────────────────┐
│ Command Palette ([Enter] Run  [Tab] Complete  [Esc] Dismiss  [Up/Down])│
├────────────────────────────────────────────────────────────────────────┤
│ > /a▋                                                                  │
│ ────────────────────────────────────────────────────────────────────── │
│ ▶ /agent          Switch or inspect active persona                     │
│   /artifacts      Browse generated reports, tearsheets, and charts     │
│   /auto           Toggle auto-approval for low-risk advisory tools     │
│   /account        Inspect profile, plan tier, and SEBI risk boundaries │
└────────────────────────────────────────────────────────────────────────┘
```

### 4.1 Palette Hotkeys & Conventions
- **`Up` / `Down`**: Navigate matching slash commands.
- **`Tab`**: Autocompletes highlighted command into prompt composer with trailing space (e.g. `/agent `).
- **`Enter`**: Executes zero-parameter commands immediately, or enters composer mode for parameter commands.
- **`Esc`**: Dismisses palette or returns to Normal mode.
- **`1` through `7`**: Instant tab switching across all screens.
- **`i` / `Enter`**: Focuses chat input when on the `AgentChat` tab.

---

### 4.2 Complete 40-Command Slash Catalog

| Command | Arguments | Scope | Description |
| :--- | :--- | :--- | :--- |
| **`/help`** | `[command]` | Global | Displays command reference or syntax for a specific command. |
| **`/agent`** | `<name\|off\|default>` | Session | Switches persona (`quant`, `analyst`, `trader`, etc.) or resets. |
| **`/new`** | None | Session | Archives current context and starts a fresh conversation turn. |
| **`/resume`** | `[session_id]` | Global | Resumes session or opens interactive session history picker. |
| **`/clear`** | None | Session | Clears visual scrollback history from the active terminal buffer. |
| **`/copy`** | `[full\|tools]` | Session | Copies latest answer, full turn transcript, or tool outputs to clipboard. |
| **`/export`** | `<json\|txt>` | Session | Exports session transcript to `~/Downloads/vyapaar/`. |
| **`/undo`** | None | Session | Reverts the last conversation turn, removing it from context. |
| **`/redo`** | None | Session | Restores a previously undone conversation turn. |
| **`/checkpoints`** | None | Session | Lists and restores saved turn checkpoints within the session. |
| **`/auto`** | None | Session | Toggles auto-approval for low-risk advisory tools. |
| **`/theme`** | `<dark\|light>` | Global | Switches the UI color palette between Dark and Light mode. |
| **`/perf`** | None | Global | Displays CPU, memory usage, and turn inference latency statistics. |
| **`/context`** | None | Session | Visualizes current context token budget (instructions, tools, memory, turns). |
| **`/unbind`** | None | Session | Unbinds active persona or model override back to default. |
| **`/remember`** | `<fact>` | Global | Saves a permanent fact directly into local vector memory. |
| **`/memory`** | None | Global | Opens interactive semantic memory inspector modal. |
| **`/knowledge`**| `<query>` | Global | Queries local semantic vector memory store for matching notes. |
| **`/jobs`** | None | Global | Opens background jobs manager (view schedules, run, edit, delete). |
| **`/cloud`** | None | Global | Displays synchronized cloud watchlists and portfolios. |
| **`/positions`**| None | Global | Shows active paper trading positions, open orders, and PnL breakdown. |
| **`/artifacts`**| None | Session | Browses generated reports, tearsheets, CSV exports, and charts. |
| **`/workspaces`**| None | Global | Lists and manages directory workspaces (`d` delete, `r` rename, `a` add). |
| **`/skills`** | None | Global | Lists installed financial skills and allowed tool sets. |
| **`/plugins`** | None | Global | Lists installed plugins and enabled extensions. |
| **`/marketplace`**| None | Global | Browses plugin marketplace sources and packages. |
| **`/mcp`** | None | Global | Lists connected MCP servers or live-connects a server. |
| **`/learnings`**| None | Global | Inspects distilled learnings (`d` download, `g` URL, `x` delete). |
| **`/computer-use`**| `<key\|off>`| Global | Configures or toggles computer use integration. |
| **`/buddy`** | `[pet\|mute\|name]`| Global | Companion desk pet status and settings. |
| **`/settings`** | None | Global | Inspects workstation configuration settings. |
| **`/account`** | None | Global | Inspects profile, tier, and SEBI pre-trade risk boundaries. |
| **`/upgrade`** | None | Global | Displays subscription tiers and upgrade details. |
| **`/billing`** | None | Global | Displays account billing, statutory compute credits, and usage. |
| **`/usage`** | None | Global | Displays token consumption, statutory limits, and latency (`r` refresh).|
| **`/logout`** | None | Global | Logs out of cloud sync session cleanly. |
| **`/verify`** | `[file]` | Session | Runs static strategy lookahead bias verification in-session. |
| **`/create-agent`**| `<name>` | Global | Creates a custom agent persona. |
| **`/delete-agent`**| `<name>` | Global | Deletes a custom agent persona. |
| **`/quit`, `/exit`**| None | Global | Exits the workstation cleanly, preserving session state. |

---

### 4.3 Modal Hotkey Map

Inside interactive modals, the following single-key bindings are supported:
- **`e`**: Edit the highlighted goal (in `/jobs`).
- **`d`**: Delete the highlighted item (in `/jobs`, `/memory`, `/artifacts`, `/workspaces`).
- **`r`**: Refresh statistics (in `/usage`) or rename workspace (in `/workspaces`).
- **`a`**: Add a new workspace (in `/workspaces`).
- **`g`**: Mint a short-lived sharing URL (in `/learnings`).
- **`x` twice**: Permanently delete learning rule (in `/learnings`).

---

## 5. Verification & Testing

The complete test suite runs under `cargo test`:

```bash
cargo test -p vyapaar-cli -p vyapaar-tui
```

- **`vyapaar-cli` test suite (23 tests)**: Validates roundtrip CLI argument parsing, environment variable precedence, JSON formatting, lookahead bias exit codes, and jobs lifecycle.
- **`vyapaar-tui` test suite (17 tests)**: Validates tab navigation, keybindings, command palette fuzzy filtering, Tab autocompletion, `/` shortcut activation, and all 40 slash command executions.
