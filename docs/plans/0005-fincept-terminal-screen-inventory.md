# 0005 — FinceptTerminal's Screen Inventory vs. vyapaar-ui's Roadmap

> **Status**: Implemented — Layers 4/5 of the bottoms-up build order. Screen
> inventory Phases 1–3 landed in `vyapaar-ui`:
> - **Watchlist / Dashboard**: `WatchlistFeature` in `vyapaar-ui-app` folds
>   canonical `Tick`/`Bar` events into `WatchlistViewModel` rows
>   (price/delta/sparkline) + `MarketStatus` + `AccountSummary`; TUI now opens
>   on this tab.
> - **Screener**: kept on the canonical `vyapaar_core::screener` wire contract
>   (`ScreenerFeature` + `vyapaar-ui-api-wasm::apply_scan`); the
>   two-screener-backend question is resolved below in §2 item 2.
> - **Backtest results viewer**: `BacktestSummary`/`BacktestResultView`
>   primitives in `vyapaar-ui-core` + `BacktestFeature` in `vyapaar-ui-app`
>   (run lifecycle, equity curve, peak/trough), a view over
>   `vyapaar-runtime-backtest`'s `BacktestResult`.
> - **Portfolio / positions**: `PortfolioViewModel` (positions + holdings +
>   totals) in `vyapaar-ui-core`, projected from the canonical
>   `vyapaar_core::portfolio::PortfolioSnapshot` wire contract; `PortfolioFeature`
>   in `vyapaar-ui-app` ingests `ClientEvent::Portfolio`.
> - **Algo / strategy management**: `AlgoRow`/`AlgoManagerViewModel` +
>   `AlgosFeature` for versioned strategy manifests and
>   `vyapaar-agent-genetic` synthesizers.
> - Router gained `Backtest`/`Portfolio`/`Algos` routes; `vyapaar-tui` is now
>   the seven-tab workstation (Watchlist, Screener, Portfolio, Orders,
>   Backtest, Algos, AgentChat).
> **Target Crates**: `vyapaar-ui-app`, `vyapaar-tui`, `vyapaar-ui-charts`, `vyapaar-ui-core`.
> **Depends on reading**:
> - `vyapaar-ui/docs/plans/0002-quantcept-web-ux-reference.md` — note explicitly: that doc is
>   about **quantcept.io's chat UX specifically** (persona settings, turn rendering, source chips)
>   and is a **different source product** from this doc, which is about **FinceptTerminal**, an
>   open-source Qt/C++ desktop trading terminal found at a separate repo path. The two should not
>   be conflated.

---

## 0. Executive Summary & Objective

A research pass inventoried FinceptTerminal's (a competing open-source desktop terminal,
AGPL-3.0, ~342K LOC) screen/feature surface — 54 lazy-instantiated screens across bounded contexts:
Markets, News, Economics, Geopolitics, Trading, Portfolio, Crypto, Derivatives, Predictions,
Agents, AI Chat, Workflow, Identity. `vyapaar-ui` currently has a UI shell
(`vyapaar-ui-charts`, `vyapaar-ui-app`, `vyapaar-ui-core`, `vyapaar-cli`, `vyapaar-tui`) but almost
none of the actual feature screens built. This is explicitly the single largest surface-area gap
found across the whole Fincept/Quantcept comparison — but also explicitly the lowest-risk one to
leave for last, since it's presentation, not domain logic: every screen below is a thin view over
a domain crate that either already exists or is covered by one of the other new plan docs written
in this same session.

---

## 1. Context

FinceptTerminal's 54 screens group into 13 bounded contexts. Rather than treat this as a single
undifferentiated backlog, this doc maps only the highest-value categories to the domain crates
that already exist (or are already planned elsewhere in this workspace) to view over, and
explicitly defers everything that would require building new domain logic first. No new domain
logic is proposed in this document — it is a presentation-layer mapping only.

---

## 2. Component Specifications

Highest-value screen categories to prioritize, each mapped to the domain crate(s) it would be a
thin view over:

1. **Watchlist / Dashboard** — a view over `vyapaar-market` live feeds. No new domain logic;
   consumes existing quote/candle streams.
2. **Screener** — a view over `vyapaar-market-indicators`'s existing `screener.rs` and
   `vyapaar-runtime-api-rest`'s `screener.rs`. **Flag, now resolved**: there ARE two screener
   backends in the workspace, and the UI must not be built against both. Resolution (verified by
   direct read at implementation time):
   - The canonical screener **wire contract** is `vyapaar_core::screener`
     (`ScreenerScanRequest`/`ScreenerScanResponse`/`ScreenerRow`), which is what the UI's
     `ScreenerFeature` already consumes and evaluates through the pure `apply_scan` in
     `vyapaar-ui-api-wasm`.
   - The production screener **HTTP backend** is `vyapaar-runtime-api`'s `/screener/scan`
     (`vyapaar-runtime-api-rest::screener`), which serves canonical rows over a bar universe.
   - `vyapaar-market-indicators::screener` is a standalone, simplified duplicate used only by its
     own `tests/` in that crate; no code in `vyapaar-ui` or `vyapaar-agent` imports it. The UI
     therefore builds only against `vyapaar_core::screener` and ignores the market-indicators
     duplicate. (Its crate-internal tests still pass; retirement is an optional cleanup outside
     this plan's scope.)
3. **Backtest results viewer** — a view over `vyapaar-runtime-backtest`.
4. **Portfolio / positions view** — a view over `vyapaar-ledger-portfolio`, and, once built, the
   target-weights output of `vyapaar-ledger-allocation` per
   `vyapaar-ledger/docs/plans/0001-portfolio-allocation-and-optimization.md`.
5. **Algo / strategy management** — a view over `vyapaar-agent-genetic` and `vyapaar-strategies`'
   versioned manifests, per
   `vyapaar-strategies/docs/plans/0001-versioned-strategy-manifests-and-baskets.md`.
6. **AI Chat / agent workbench** — already the subject of `vyapaar-ui/docs/plans/0001` and
   `vyapaar-ui/docs/plans/0002`; not duplicated here.

### Explicitly out of scope / not prioritized for now

News, Geopolitics, Crypto, Derivatives/options UI, and Predictions (Polymarket-style) screens are
not prioritized. These map to domain gaps this session's other new plan docs flagged as
low-priority or explicitly deferred elsewhere in the workspace — building UI for them first would
mean building ahead of the data those screens would need to show.

---

## 3. Implementation Phases

### Phase 1: Watchlist / Dashboard + Screener
- Build the Watchlist/Dashboard view over `vyapaar-market` live feeds.
- Resolve the two-screener-backend question (§2 item 2) before building the Screener view, then
  build it over whichever of `vyapaar-market-indicators`'s or `vyapaar-runtime-api-rest`'s
  `screener.rs` is determined canonical.

### Phase 2: Backtest results + Portfolio views
- Backtest results viewer over `vyapaar-runtime-backtest`.
- Portfolio/positions view over `vyapaar-ledger-portfolio`, extended to `vyapaar-ledger-allocation`
  target weights once that crate exists.

### Phase 3: Algo / strategy management view
- Algo/strategy management view over `vyapaar-agent-genetic` and `vyapaar-strategies` manifests.
