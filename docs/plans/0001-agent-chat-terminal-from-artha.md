# 0001 — Building `vyapaar-cli` / `vyapaar-tui` / `vyapaar-highlight` from the `artha` design work

> **Status**: implemented — crate build-out complete and verified (`cargo check --workspace --profile prod` green, `cargo test --workspace --profile prod` 275 passed, clippy/fmt clean).
> **What was built**: `vyapaar-highlight` (zero-I/O `Language`/`HighlightSpan`/`highlight_code` lexer), `vyapaar-tui` (`TuiApp`, `ActiveTab{Screener,Portfolio,Orders,AgentChat}`, `run_terminal` crossterm event loop), `vyapaar-cli` (`CliArgs`/`Command` dispatch + `run` runner; Ask/Chat/Tui/Health wired; Jobs/Strategy/Verify/etc. implemented).
> **Residual, tracked separately**: CLI `scan` is wired to the canonical screener
> (`crates/vyapaar-cli/src/scan.rs`, `CommandOutcome::ScanResults`); gap `0001`
> closed (gap doc removed). Real `AgentChat`/`Ask` wiring to `vyapaar-agent` is
> blocked upstream on the `vyapaar-agent` port landing.
> **Depends on reading**: `vyapaar-ui/docs/migration-guide.md`, `vyapaar-agent/docs/migration-guide.md`,
> `vyapaar-market/docs/migration-guide.md`, `vyapaar-runtime/docs/migration-guide.md`, and the
> `artha` workspace referenced throughout this doc.

## 0. What this is, in one paragraph

A separate session designed a from-scratch Rust rewrite of a local-first AI finance chat
terminal, under the working name **Artha**, as a 12-crate workspace at
`~/tmp/artha/` (pushed to `git@github.com:dev-pmallapp/artha.git`). Partway
through writing detailed HLD/LLD design docs crate by crate, it turned out `vyapaar-labs`
already exists at `vyapaar-labs` — a much larger, already-planned
"autonomous algorithmic trading platform for NSE/BSE" — and that **`vyapaar-ui`'s own migration
guide already reserves three crates, `vyapaar-cli`, `vyapaar-tui`, and `vyapaar-highlight`, as
brand-new (no port source), currently just empty stubs**, for exactly this kind of terminal
workstation. This doc is the reconciliation: what to keep from the `artha` design work, what to
discard or adapt, and a concrete plan for building out those three crates (plus pointers for a
few adjacent new crates elsewhere in `vyapaar-labs` that the same design work is also relevant
to). **Read §1 before touching any code** — the scope is narrower, and the actual target
contracts are different in specific ways, from what the `artha` docs assume.

## 1. The critical scope correction

`vyapaar-ui/docs/migration-guide.md` already specifies minimal contracts for the three new
crates:

```
vyapaar-highlight: Language, HighlightSpan, highlight_code  — zero-I/O syntax highlighter
vyapaar-tui:        TuiApp, ActiveTab{Screener, Portfolio, Orders, AgentChat}, keybindings, layout
vyapaar-cli:        CliArgs, Command{Tui, Ask, Scan, Health}, runner logic
```

Three things this implies that the `artha` docs (written before this was known) do **not**
assume:

1. **`vyapaar-tui` is a multi-tab trading workstation, not a chat app.** `AgentChat` is *one of
   four* tabs, alongside `Screener`, `Portfolio`, and `Orders` — the other three are driven by
   `vyapaar-market`/`vyapaar-ledger` data, not the agent at all. The `artha-tui` design (chat
   transcript + input box + permission modal + streaming markdown) is the right design **for the
   `AgentChat` tab's content area only**, not for the whole app. `TuiApp` and the tab router
   around it are new work the `artha-tui` docs don't cover.
2. **`vyapaar-cli`'s command surface is much smaller.** Four commands (`Tui`, `Ask`, `Scan`,
   `Health`), not `artha-cli`'s planned `plugin`/`jobs`/`memory`/`verify`/`purge` surface. Treat
   the broader surface as a *backlog*, not a v1 target — build the four-command contract the
   migration guide actually asks for. `Ask` and `Health` map directly onto `artha-core`'s
   `run_turn_headless` and a doctor-style check, respectively; `Scan` is new (screener-related,
   not an `artha` concept at all).
3. **`vyapaar-highlight` must be zero-I/O and WASM-safe**, because (per the same migration
   guide's §2) `vyapaar-highlight` is not in the list of WASM-excluded crates — it's plausibly
   shared with the WASM Strategy Studio (`vyapaar-ui-app`, etc.), which must stay free of
   `tokio` with `net`/`rt-multi-thread` and anything that needs native codegen. **Do not port
   `artha-highlight`'s tree-sitter + WASM-grammar design** (tree-sitter's native grammars don't
   target `wasm32-unknown-unknown` without reintroducing the same heavy WASM blobs the `artha`
   rewrite was trying to get away from in the first place). Build a small hand-rolled
   regex/lexer-based highlighter per language instead. Reassuringly, `artha-tui`'s own docs
   already assumed a compatible seam: *"`artha-highlight` returns byte ranges with a closed class
   enum; the TUI turns them into styled spans"* — that's directly compatible with
   `HighlightSpan`; only the implementation underneath (not the shape) needs to change.

## 2. What's actually done in `artha`, and where it's directly useful

All of this lives at `~/tmp/artha/crates/<name>/docs/{HLD,LLD}.md`
(also on GitHub: `dev-pmallapp/artha`, branch `main`).

| `artha` crate | Status | Where it's useful here |
|---|---|---|
| `artha-tui` | **Done** (HLD+LLD) | **Primary reference for the `AgentChat` tab's content area** in `vyapaar-tui`: streaming markdown render, code-block highlighting via the highlight seam, the permission-prompt modal, input handling, the sync-render/async-core event bridge. Not a reference for `TuiApp`/`ActiveTab`/the other three tabs — that's new design. |
| `artha-core` | **Done** (HLD+LLD) | Reference for **how `AgentChat` should drive a chat session**: the `CoreEvent` stream shape, `SessionHandle` (`submit`/`cancel`/`history`/`set_provider`/background-ticket list/cancel), permission-prompt-as-event-with-oneshot-responder pattern, session resume. **Not** a reference for `vyapaar-agent`'s own agent state machine — that crate is a straight port from `honba-nou-agent` (see §3). Use this doc to sanity-check the *UI-facing* contract `vyapaar-agent` should expose, not to redesign the agent internals. |
| `artha-llm` | **Done** (HLD+LLD) | Secondary reference only — see §3. `vyapaar-agent-llm` is ported from `honba-nou-llm`, which likely already has a working `LlmProvider`-equivalent. Diff this doc against the ported code for gaps (e.g. the streaming tool-call-assembly state machine, retry/backoff policy) rather than treating it as the blueprint. |
| `artha-data` | **Done** (HLD+LLD) | Secondary reference only. The generic `Db`/`JsonStore`/migration-framework pieces may be useful if `vyapaar-runtime-persistence` (ported from `honba-ya-persistence`) is missing something, but that crate's job is explicitly to "implement `vyapaar_ledger` repository traits" against an existing ledger, not a green-field data layer. The checkpoint/git2 "undo agent file edits" feature has **no confirmed home** in `vyapaar-labs` yet — it doesn't obviously match any existing crate's stated scope. Don't assume it's covered; flag it as a product-scope question if file-editing tools end up in scope for `AgentChat`. |
| `artha-tools` | **Done** (HLD+LLD) | Relevant to `vyapaar-agent-tools` (also a new crate — see §3), **but with a load-bearing mismatch**: `artha-tools` implements `artha-core`'s `Tool` trait, which is `async` and directly I/O-capable (it calls `Db::call`, HTTP clients, etc., straight from `execute`). `vyapaar-agent-tools`'s spec says *"pure `AgentTool` trait"* — "pure" strongly suggests zero-I/O, synchronous, in the same spirit as `vyapaar-ledger`'s "Zero-I/O Domain Purity" invariant. **Resolve this explicitly before porting anything** — if `vyapaar-agent-tools` really wants zero-I/O tools, the I/O has to live somewhere else (`vyapaar-agent-trader-loop`? `vyapaar-runtime`?) and `artha-tools`' design needs reshaping around that split, not a direct port. |
| `artha-finance` | **Not done** — subagent was killed mid-run, no docs exist | Not applicable here at all. `vyapaar-market` (all of it, including the would-be `artha-finance` equivalent, `vyapaar-market-finance`) is a straight port from `honba-me`, already fully crate-mapped in its own migration guide. There was never a finished `artha-finance` doc to begin with; don't go looking for one. |
| `artha-cli`, `artha-highlight`, `artha-jobs`, `artha-mcp`, `artha-plugins`, `artha-report` | **Never started** — empty `docs/` directories, no subagent was ever run for these | No design work exists for these at all. `vyapaar-cli` and `vyapaar-highlight` need fresh design now (this doc's main deliverable, §4). `artha-mcp`/`artha-plugins`/`artha-report` would have mapped to `vyapaar-agent-mcp` (ported from `honba-nou-mcp` — not new, skip), `vyapaar-agent-plugins` (new — see §3), `vyapaar-agent-report` (new — see §3). A hypothetical `artha-jobs` would have mapped to `vyapaar-runtime-jobs` (new — see §3), but since it was never written, there's nothing to carry over; design that fresh too, directly against the `vyapaar-runtime` spec. |

## 3. Pointers for adjacent new crates outside `vyapaar-ui` (not this doc's job to execute, but worth knowing)

These are new crates in *other* `vyapaar-labs` repos where `artha`'s design instincts are
relevant, listed here so whoever picks up those repos doesn't miss the connection. Each still
needs its own plan doc in its own repo if the `artha` docs are actually used for it — don't just
copy-paste across repos.

- **`vyapaar-agent/crates/vyapaar-agent-tools`** (new, per `vyapaar-agent/docs/migration-guide.md`):
  see the `artha-tools` row in §2 — useful, but only after the pure-vs-async `Tool` trait
  question is resolved.
- **`vyapaar-agent/crates/vyapaar-agent-plugins`** (new): spec is `PluginManifest`,
  `PluginCapability`, `PluginRegistry`. No `artha-plugins` doc exists (see §2), so this needs
  fresh design; the `artha` `ARCHITECTURE.md`'s plugin/marketplace notes (git-based fetch of a
  marketplace index) are a reasonable starting point but were never worked out in detail.
- **`vyapaar-agent/crates/vyapaar-agent-report`** (new): spec is a `TearsheetReport` +
  Typst/Markdown generator over `vyapaar-core` `Decimal` metrics. No `artha-report` doc exists
  either, but the one concrete idea from `artha`'s `ARCHITECTURE.md` worth carrying over: **embed
  the `typst` crate directly** rather than shelling out to a `typst` binary — `typst` is itself a
  native Rust library, and that's a real simplification regardless of which report schema ends
  up being generated.
- **`vyapaar-runtime/crates/vyapaar-runtime-jobs`** (new, per `vyapaar-runtime/docs/migration-guide.md`):
  spec is a deterministic `JobScheduler`/`CronSchedule`/`JobSpec` for domain-specific jobs
  (pre-market TOTP refresh, EOD settlement, snapshot jobs) — notably *not* the same shape as
  `artha`'s "background research tasks/monitors" framing, and no `artha-jobs` doc exists anyway.
  Design this fresh against the actual job types listed, not against `artha`'s more
  general-purpose automation idea.

**Explicitly do not** use `artha-core`, `artha-llm`, or `artha-data` as the primary design for
`vyapaar-agent-core`/`vyapaar-agent-llm`/`vyapaar-runtime-persistence` — those are straight ports
from existing, presumably-working source (`honba-nou`, `honba-ya`) per their own migration
guides. Same for all of `vyapaar-market` relative to `artha-finance` (which doesn't exist anyway).

## 4. The actual plan for `vyapaar-cli` / `vyapaar-tui` / `vyapaar-highlight`

### 4.1 `vyapaar-highlight`

1. Define `pub enum Language { Rust, Python, Json, Markdown, Shell, /* … whatever the chat and
   screener/studio code-block needs cover */ }` and `pub struct HighlightSpan { pub range:
   Range<usize>, pub class: HighlightClass }` with a small `HighlightClass` enum (keyword,
   string, comment, number, function, type, punctuation, plain — keep it small; this is for
   terminal/web styling, not an IDE).
2. `pub fn highlight_code(lang: Language, src: &str) -> Vec<HighlightSpan>` — a dumb,
   fast, regex-or-handwritten-lexer tokenizer per language. No tree-sitter, no WASM grammar
   blobs, no runtime dependency beyond maybe `regex` (confirm `regex` itself is
   `wasm32-unknown-unknown`-safe before taking it as a dependency — it should be, since it's
   pure Rust with no I/O, but verify rather than assume).
3. Zero `#[cfg(test)]` in `src/` — the migration guide's convention across every repo in this
   workspace is tests live in `tests/unit_tests.rs`.

### 4.2 `vyapaar-tui`

1. `TuiApp` owns a `ratatui` + `crossterm` event loop (this part of `artha-tui`'s design
   transfers directly — see its HLD §"concurrency model" for the sync-render/async-core bridge
   pattern) and an `ActiveTab` enum: `Screener | Portfolio | Orders | AgentChat`.
2. Build the `AgentChat` tab's content area directly from `artha-tui`'s LLD: the `AppEvent`
   type, the markdown+highlight rendering pipeline (now calling `vyapaar_highlight::highlight_code`
   instead of a tree-sitter-backed crate), the streaming-delta throttling strategy, and the
   permission-prompt modal flow — all of that logic is UI-only and doesn't care which agent
   backend it's talking to.
3. The `AgentChat` tab needs *something* implementing a `CoreEvent`-stream-and-`SessionHandle`-
   shaped interface to drive it. Until `vyapaar-agent`'s own port lands with a UI-facing API,
   treat `artha-core`'s HLD §3 (public API surface) as the **target shape to ask `vyapaar-agent`
   to expose**, not something to build inside `vyapaar-tui` itself. If `vyapaar-agent` already
   exposes something close, prefer wiring to the real thing over inventing a parallel contract.
4. `Screener`, `Portfolio`, `Orders` tabs: out of scope for this doc. They're driven by
   `vyapaar-market`/`vyapaar-ledger`, which are separate ported repos; build them against
   whatever public API those expose once available, as their own follow-up work.

### 4.3 `vyapaar-cli`

1. `CliArgs` via `clap` (derive API — matches `artha-cli`'s intended choice, and there's no
   reason to differ here).
2. `Command::Tui` → launches `vyapaar_tui::TuiApp`. `Command::Ask(prompt)` → one headless turn,
   printing the answer (`artha-core`'s `run_turn_headless` is exactly this shape; wire to
   whatever `vyapaar-agent` exposes once it lands). `Command::Health` → an `artha doctor`-style
   environment check (what's actually provisioned/reachable — adapt the specific checks to
   whatever this platform's runtime dependencies turn out to be, e.g. broker auth, DB
   connectivity, not `artha`'s Python-sidecar-specific checks). `Command::Scan` → new, no `artha`
   analog; coordinate with whoever owns the screener logic in `vyapaar-market`/`vyapaar-runtime`
   for what this should actually do.
3. Keep this crate thin — argument parsing and dispatch only, consistent with how `artha-cli`
   was scoped (a "zero-dep launcher" in spirit, even though this isn't a separate npm-launcher
   situation here).

## 5. Suggested order of work

1. `vyapaar-highlight` first (§4.1) — smallest, no dependencies on anything else, and both
   `vyapaar-tui` and (potentially) the WASM UI crates need it.
2. `vyapaar-tui`'s `TuiApp`/`ActiveTab` shell and the `AgentChat` tab's static rendering (no live
   agent yet — feed it canned/fake events first, matching `artha-tui`'s own testing-strategy
   idea of a `ratatui::TestBackend` + scripted event fixtures).
3. `vyapaar-cli`'s `CliArgs`/`Command` surface, wired to `Tui` launching what §2 built; stub
   `Ask`/`Health`/`Scan` with clear "not yet wired to a backend" errors rather than fake success.
4. Only once `vyapaar-agent` has something real to talk to: wire `AgentChat` and `Ask` to it for
   real, reconciling against `artha-core`'s HLD §3 contract as the UI-side wishlist, and fixing
   whatever doesn't match.

## 6. Open questions to resolve, not assume

- Does `vyapaar-agent-tools`' "pure" `AgentTool` mean strictly zero-I/O? If yes, where does tool
  I/O (market data lookups, order placement, memory/notes storage) actually happen? This blocks
  a clean answer for what `AgentChat`'s tool-call-result rendering should expect to receive.
- Is there any product requirement for an agent-editable-file + undo/checkpoint feature
  (`artha-data`'s git2 snapshot design) anywhere in this platform? If not, don't build it "just
  in case" — it has no stated home right now.
- Confirm `vyapaar-highlight`'s dependency choices (even something as small as `regex`) actually
  compile for `wasm32-unknown-unknown` before relying on them, per the WASM-portability
  constraint in `vyapaar-ui/docs/migration-guide.md` §2.
