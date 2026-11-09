# 0002 — ui-core engine exposes no per-trade rows (job journal `trades.json` writes aggregates only)

**Status**: open

## What's stubbed

`vyapaar-ui` (PLAN-0001 Stage 4 in `vyapaar-cli/src/jobs_runtime.rs`) journals a
completed job run's `trades.json` as `{"total_trades": <u64>}` because
`vyapaar-ui-core::strategy::BacktestReport` (`crates/vyapaar-ui-core/src/strategy.rs`)
only carries scalar aggregates — no per-trade rows:

```rust
pub struct BacktestReport {
    pub initial_capital: Decimal,
    pub final_equity: Decimal,
    pub net_pnl: Decimal,
    pub total_return_pct: Decimal,
    pub sharpe_ratio: Decimal,
    pub max_drawdown_pct: Decimal,
    pub total_trades: u64,          // the only trade signal the report owns
    pub win_rate_pct: Decimal,
    pub profit_factor: Decimal,
}
```

The deterministic simulator backing it (`simulate_strategy`) tracks its synthetic
position internally and discards the individual fills, so the journal can only
honestly record the aggregate count. `metrics.json` (the full report) and
`events.ndjson` (`run.result` event) carry the same report verbatim.

## Cited-by

- `vyapaar-ui/crates/vyapaar-cli/src/jobs_runtime.rs` (`execute_in_journal`,
  `trades.json` write stands on this gap).

## How to close

Teach `vyapaar-ui-core::strategy` to emit trade rows (entry/exit timestamps,
prices as `Decimal`, quantity, fees per fill — the same shapes as
`vyapaar_runtime_api::responses::FillEntry`) from `simulate_strategy`, add a
`trades: Vec<TradeRow>` field to `BacktestReport`, and switch the job-run
journal's `trades.json` from `{"total_trades": N}` to the serialized row list
(planning the FIX/DOT integration note: keep rows `Decimal` — zero floats).
Then `trades.json`, `metrics.json`, and `events.ndjson` become replayable
evidence for a tax-lot ledger rebuild.
