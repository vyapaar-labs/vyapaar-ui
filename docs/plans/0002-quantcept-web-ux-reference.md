# 0002 — UX reference for the `AgentChat` tab, captured from a live product session

> **Status**: reference — written by one agent after a hands-on session against a live product
> the user has an account on, intended to be read by whoever builds out `vyapaar-tui`'s
> `AgentChat` tab, `vyapaar-cli`'s `Ask` command, and `vyapaar-agent-chat`.
> **Depends on reading**: [`0001-agent-chat-terminal-from-artha.md`](./0001-agent-chat-terminal-from-artha.md)
> (crate split and scope correction) and `vyapaar-agent/docs/design/0001-chat-assistant-and-superset-capabilities.md`
> (the `vyapaar-agent-chat` / `vyapaar-agent-chat-tools` design this doc's features ultimately
> have to be implemented against). **This doc inherits that design's implementation gate: none of
> this is to be built until the `honba-*` migrations (Milestones 3.1 and 3.2) are complete.**

## 0. What this is, in one paragraph

0001 mapped *crate boundaries* (which Rust crate owns what). This doc is narrower and more
concrete: a feature-by-feature account of how a finished product in this exact space —
"AI finance chat terminal, also shipped as a local CLI" — actually behaves for a logged-in user,
captured by directly using its web client (`quantcept.io/chat`) for one session: the dashboard,
the agent/persona settings, and one real turn end-to-end (question → tool calls → streamed
answer → embedded chart → follow-up). The goal is not to copy this product's branding or backend;
it's to have a concrete, observed reference for interaction patterns that are easy to under-spec
from architecture docs alone — turn-rendering sequencing, what a "tool call" looks like to the
user while it's running, how citations and generated charts are presented, and the knobs a power
user expects (effort level, auto-approve, persona switching). Where a pattern is directly
reusable in a TUI, this doc says so; where it's web-only and needs a terminal-native adaptation,
it says that too.

## 1. Two answer surfaces: `Answer` vs `Chat`

The product distinguishes two modes of asking something, and this distinction is worth keeping
rather than collapsing into "it's all just a chat thread":

- **`Answer`** — a single question asked from the dashboard (via the suggested-question list, the
  top search bar, or typing directly into the dashboard's composer) renders as a dedicated,
  single-turn page: the question as a heading, then the turn's execution trace, then the final
  answer. It is not initially part of any multi-turn thread. A **"Continue in chat"** affordance
  in the top-right promotes it into a full conversation if the user wants to follow up with
  context; otherwise an **"Ask a follow-up"** box at the bottom starts a *new* answer that is
  still independent, not implicitly chained.
- **`Chat`** — a named, persisted, multi-turn conversation, listed under "Chats" in the sidebar,
  with history navigable like any chat app.

Both are backed by the same turn-execution engine (same tool calls, same streaming format below)
— `Answer` is simply a chat thread of length one, rendered without the thread chrome, with a
promotion action instead of a reply box by default, and with the auto-generated title ("AAPL
Stock Drivers Today") doubling as the dashboard sidebar entry.

**Mapping**: `vyapaar-agent-core`'s session model already treats every exchange as a turn inside
a session (`SessionHandle`); nothing new is needed at that layer. What's missing from the existing
Artha-derived design is the *UI-level* distinction — `vyapaar-cli`'s `Ask` command producing a
plain one-shot answer (no persisted multi-turn session needed) is the CLI-native equivalent of
`Answer`, and `vyapaar-tui`'s `AgentChat` tab is the `Chat` equivalent. A TUI could still offer a
lightweight "quick ask" overlay (bound to a keystroke from any tab, not just inside `AgentChat`)
that mirrors `Answer`'s "ask without committing to a thread" ergonomics — flagged here as a nice-
to-have, not a requirement.

## 2. Turn-rendering sequence, observed live

This is the part most worth getting right, because it's the thing a user actually watches happen
in real time. For the test question "What is driving AAPL today?" (Medium effort, extended
thinking on), the turn rendered as this ordered sequence of blocks, each appearing and completing
before the next began:

1. **Preamble sentence** — a short, model-generated line of intent before any tool runs: *"I'll
   pull AAPL's live quote, recent price history, and the news flow around it now."* Not a tool
   call; just a plan statement so the user isn't staring at a blank spinner.
2. **Tool-call group, collapsed by default** — `✓ Ran 4 tools · Quote, Price history, News · search +1`
   with an elapsed-time badge (`3s`) and a chevron to expand. Expanding it (confirmed by clicking)
   lists each individual call as its own row: tool name, a short rendering of its key argument
   (e.g. `News · search  stocks selloff Nasdaq plunge VIX spike`), and a live status word
   (`running` → presumably a completed/success marker once done, matching the group's own `✓`).
   A `Read page` tool call rendered its argument as the actual URL being fetched. This is a
   genuine agentic tool loop, not a single tool-call round: this turn ran **three separate
   tool-call groups** back to back (`4 tools`, then `4 tools` again, then `2 tools`), each
   preceded by its own short narration sentence and followed by a collapsed `Reasoning` /
   `Thought for Ns` block.
3. **Narration between rounds** — a one-line sentence introducing *why* the next round of tools is
   running, e.g. *"Now the macro driver behind that broad index move, plus the AAPL-specific
   items."* This is what makes a 38-second, multi-round turn feel legible instead of like a stalled
   spinner — the user always knows what's being looked up and why.
4. **Reasoning / thinking, collapsed** — shown as `Thought for 6s` (collapsed) between tool-call
   groups, and also as a `Thinking` block right before the final answer starts streaming, which
   briefly showed partial draft text before settling into the final rendered answer. Collapsed by
   default; the raw deliberation is available but out of the way.
5. **Final answer** — streamed markdown with:
   - **Bold topic-sentence leads** on each paragraph (`**The dominant driver was market-wide, not
     Apple.**`, `**Read the relative performance, not the headline.**`) — a deliberate formatting
     convention, not incidental.
   - **Inline citations as parenthetical source+date**, e.g. `(BigGo Finance, 5 Nov)`,
     `(MarketBeat)` — not footnote markers, not hyperlinked superscripts; just prose-readable
     attribution inline.
   - **An embedded generated chart** — in this case an "AAPL daily candles (last 30 sessions to
     5 Nov 2026)" candlestick chart with a volume bar subplot underneath, rendered as its own card
     with a `...` menu, placed inline in the answer flow exactly where the text referenced price
     action. This is a first-class answer primitive, not an attachment — the model decided a
     chart was the right way to support this specific claim and the renderer honored it.
6. **Source-chip bar** — above the answer (observed at the very top, under the question heading,
   once the turn finished), a row of favicon + domain chips aggregating every distinct source used
   across *all* tool-call rounds in the turn (`seekingalpha.com`, `marketbeat.com`,
   `stocktitan.net`, `finance.yahoo.com`, …, with `+4 more` overflow). This is deduplicated and
   turn-scoped, not per-paragraph.
7. **Footer** — elapsed time and credit cost (`48s · 2 credits`), plus copy and regenerate icons.
8. **Follow-up composer** — `Ask a follow-up`, scoped to continue this specific thread.

**Why this sequencing matters for the TUI design**: `artha-tui`'s docs (per 0001, reusable for the
`AgentChat` tab's content area) already assumed a `CoreEvent` stream model and streaming markdown
re-parsed from the last safe block boundary. That's structurally correct for this. What's not yet
explicit in the existing design is that the event stream needs to carry, as distinct renderable
block types interleaved in order:

- plan/preamble text (plain assistant text, nothing special)
- a *tool-call group* that itself contains multiple named sub-calls with individual status, not
  just one tool call at a time — the existing `Tool` trait and `ToolPlan` model support this at
  the execution layer, but the TUI rendering needs a "group of N calls with M done" collapsed
  widget, expandable per-call
- a collapsed reasoning/thinking segment, distinguishable from plan text and from the final answer
- the final answer text, carrying structured citation references and optional chart data
  alongside the markdown, not citations baked into the markdown as plain text

A terminal can't render a live interactive candlestick card. The TUI-native adaptation: render
chart-bearing answer segments as either (a) a compact braille/unicode sparkline inline (good
enough for "down 1.5% over 5 days" at a glance) with a keybinding to open the full chart in the
system browser or a `vyapaar-market` screener tab, or (b) a placeholder card
(`[AAPL daily candles, last 30 sessions — press 'v' to view]`) that hands off to a plotting
surface outside the terminal. Either way, the *chart-as-a-citable-artifact-inline-in-the-answer*
pattern is worth keeping — don't downgrade it to "the model just mentions numbers in prose."

The source-chip bar is new relative to anything in the existing Artha/vyapaar design docs. A
terminal-native version is a one-line footer under the answer: `Sources: seekingalpha.com,
marketbeat.com, stocktitan.net +4 more (press 's' for list)`, expandable to a list view with the
specific query/URL each tool call used (recoverable from the already-planned per-call `ToolPlan`
metadata — this needs no new data model, only a renderer that aggregates it turn-wide).

## 3. Agents / personas, effort, and behavior settings

The product's "Agents" page is a single settings surface worth replicating in spirit (not
necessarily UI-for-UI) because it cleanly separates three independent knobs that the existing
`vyapaar-agent-chat` design doc left as open/deferred:

- **Persona selection** — eight built-in personas (`QuantceptAI` the generalist/default, `Analyst`,
  `Devil's Advocate`, `Macro Strategist`, `Portfolio Manager`, `Quant`, `Risk Manager`, `Trader`,
  `Value Investor`), each a one-line role description plus a default effort level, and a
  user-created-agent slot (capped at 20, "give it a role, a focus, a method, or describe it and
  let QuantceptAI draft it"). This is effectively a named, swappable **system-prompt preset** —
  nothing more exotic than that. It maps directly onto the `PromptBuilder`/`PromptContributor`
  model already in `artha-core`'s HLD and inherited by `vyapaar-agent-chat`: a persona is a
  contributor that supplies a different stable block, selected per-session or per-message.
- **Effort and thinking** — a single Low/Medium/High/Extra High/Max slider, each level showing its
  behavior in one line ("Balanced depth and cost. Thinks through complex questions. · 2 credits
  per answer") plus a separate `Extended thinking` toggle (step-by-step reasoning before
  answering; Low/Medium think only on complex questions, High/Max think on every question;
  thinking tokens billed like any other output). Picking a persona applies its default effort,
  overridable globally here or per-message from the composer. This is a clean, reusable
  three-variable model — **effort tier, extended-thinking on/off, and per-persona defaults** — and
  is worth adopting as-is for `vyapaar-agent-chat`'s request shape, independent of whether
  `vyapaar-labs` ever bills in credits.
- **"Your instructions"** — a free-text box ("What should QuantceptAI keep in mind about you?
  Your desk, the markets you cover, how you like numbers presented.") explicitly scoped as
  *"Added to every message, whichever agent answers"* and *"synced to your account · never include
  secrets or MNPI"*, with a separate link to **"Manage memory"** for what the assistant has learned
  from past chats. This is precisely the stable-profile / per-turn-notice split already resolved
  in `vyapaar-agent/docs/design/0001-chat-assistant-and-superset-capabilities.md` (§ memory via
  extended `PromptBuilder`) — this observation is a confirmation of that design choice, not a
  change to it. Worth lifting verbatim: the explicit **"never include secrets or MNPI"** inline
  warning text on the instructions box, and the account-level (not session-level) scope of these
  instructions.
- **Behavior toggles**:
  - `Auto-approve agent actions` — *"Run tools that change your data (monitors, schedules,
    portfolio edits) without asking first."* Off by default. This is the user-facing control for
    exactly the permission-prompt-as-event pattern already designed into `artha-core`/
    `vyapaar-agent-chat` (`ToolTraits::confirm`) — confirms that confirmation should be a
    per-account toggle the user sets once, not a per-call nag, for tools the account has decided
    it trusts.
  - `Next-step suggestions` — *"After an answer, offer follow-ups you can run with one click or the
    1–9 keys."* Numbered quick-replies after every answer, directly clickable or dispatched via a
    numeric keybinding. This is a genuinely new UI affordance not present in any existing
    Artha/vyapaar doc, and it's cheap to add to a TUI: the TUI already owns keybinding dispatch, so
    rendering the model's suggested follow-ups (if `vyapaar-agent-chat` starts returning them as
    structured output alongside the answer) as a numbered list the user can hit `1`-`9` on is a
    small, high-value addition to `AgentChat`'s keymap.

## 4. Cross-client sync, confirmed

The Chats list page states directly: *"Chats from the web, the Finance desk, and the Quantcept
terminal all land here."* The Portfolio page states: *"Synced with the Quantcept terminal."* Both
confirm, from the product side rather than just architectural inference, that sessions and
portfolios are account-scoped server/local-store state shared across every client surface
(web app, dashboard, and CLI/TUI) rather than siloed per-client. This matches — and is a useful
external confirmation of — the existing `vyapaar-agent-chat` `SessionStore` trait design
(owned by `vyapaar-agent-chat`, implemented by `vyapaar-runtime-persistence`) and
`PortfolioReader` (owned by chat-tools, implemented by `vyapaar-runtime-api`): nothing to change,
just a concrete product-level reason those seams were drawn as traits rather than assuming a
single process owns the data.

## 5. Explicitly out of scope for this doc

The dashboard's market-data surfaces — Top assets ticker strip, Market brief ("Write today's
brief" one-click macro summary), multi-market scope selector (US markets / Indices / Crypto /
Commodities / Bonds / Forex / ETFs / Funds / Futures), Watchlist panel, News feed — are **not**
part of this reference. They belong to `vyapaar-tui`'s `Screener`/`Portfolio` tabs and
`vyapaar-market`'s own data model, which already have their own migration guides and are
explicitly *not* the `artha`/chat lineage this doc and 0001 are tracing. Mentioned here only so
the next reader doesn't mistake their absence for an oversight.

## 6. Summary — what to actually carry forward

| Observed pattern | Already covered by existing design? | Action |
|---|---|---|
| `Answer` (one-shot) vs `Chat` (thread) | Session/turn model: yes. UI-level distinction: no | `vyapaar-cli Ask` = one-shot; `AgentChat` tab = thread; optional "quick ask" overlay is a nice-to-have |
| Preamble sentence before tools | Yes (plain assistant text in event stream) | No change needed |
| Grouped, multi-round tool calls with per-call live status | Tool execution: yes. Grouped-collapse rendering: no | Add a "tool-call group" block type/widget to the `AgentChat` rendering model |
| Collapsed reasoning/thinking between rounds | Yes (`ChatEvent`/thinking already modeled in LLM layer) | Ensure TUI renders it collapsed, distinct from plan text and final answer |
| Inline bold-lead, parenthetical-citation markdown style | Rendering: yes (markdown pipeline exists) | Adopt as the house answer style for `vyapaar-agent-chat`'s system prompt |
| Embedded chart inline in the answer | No | TUI-native adaptation: sparkline inline + handoff keybinding to full chart (§2) |
| Turn-scoped, deduplicated source-chip bar | No | One-line footer + expandable list, built from existing per-call `ToolPlan` metadata |
| Named personas as system-prompt presets | `PromptContributor` model: yes. Persona catalog/UI: no | Model as a `PromptContributor` variant selectable per session/message |
| Effort tier + extended-thinking toggle + per-persona defaults | Partially (effort mentioned in `artha-llm`/`artha-core`) | Adopt the three-variable shape as-is for `vyapaar-agent-chat` request params |
| Account-level "Your instructions" + "never include secrets/MNPI" | Design: yes (superset-capabilities doc) | Confirmed, no change; worth lifting the inline warning copy |
| `Auto-approve agent actions` toggle | `ToolTraits::confirm`: yes. Account-level toggle: no | Add an account/session preference gating the confirm-prompt path |
| Numbered (1–9) next-step suggestions | No | New: have `vyapaar-agent-chat` optionally return structured follow-up suggestions; TUI renders + keybinds them |
| Cross-client session/portfolio sync | Yes (`SessionStore`, `PortfolioReader` traits) | Confirmed, no change |

Nothing in this doc requires a new crate. It is entirely clarification and confirmation of
behavior inside crates `0001` and the `vyapaar-agent` superset-capabilities design doc already
assign: `vyapaar-agent-chat` (turn/event model, personas-as-contributors, effort/thinking params,
auto-approve preference, follow-up suggestions), `vyapaar-agent-chat-tools` (tool-call grouping
metadata, source aggregation), and `vyapaar-tui`'s `AgentChat` tab (all the rendering-side
adaptations in §2–§3). Build order is unchanged from 0001 — this doc only sharpens what "build the
`AgentChat` tab" concretely means once that work starts, which remains gated on the `honba-*`
migration completing per the superset-capabilities doc's own implementation gate.
