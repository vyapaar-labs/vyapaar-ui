//! Watchlist / dashboard feature: reactive state over live market data.
//!
//! Consumes canonical `vyapaar-core` `Tick`/`Bar` events (as delivered by
//! `vyapaar-ui-client`'s `ClientEvent::Tick` / `ClientEvent::Bar`) and folds
//! them into the presentational [`WatchlistViewModel`] (per
//! `docs/plans/0005-fincept-terminal-screen-inventory.md` §2 item 1 — a thin
//! view over `vyapaar-market` live feeds). WASM-safe: no I/O, no tokio, all
//! math exact `Decimal`.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use vyapaar_core::market::{Bar, Tick};
use vyapaar_core::types::{InstrumentId, Price};
use vyapaar_ui_core::atoms::Delta;
use vyapaar_ui_core::home::{MarketStatus, WatchlistRow, WatchlistViewModel};

/// The watchlist feature state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WatchlistFeature {
    /// Display rows, keyed by instrument (stable insertion order preserved by
    /// the view model's `upsert`).
    pub view: WatchlistViewModel,
    /// Per-instrument previous close, for delta computation.
    previous_close: std::collections::BTreeMap<InstrumentId, Decimal>,
}

impl WatchlistFeature {
    /// Create an empty watchlist.
    pub fn new() -> Self {
        Self {
            view: WatchlistViewModel::empty(),
            previous_close: std::collections::BTreeMap::new(),
        }
    }

    /// Set the market session status shown on the dashboard.
    pub fn set_market_status(&mut self, status: MarketStatus) {
        self.view.status = Some(status);
    }

    /// Fold a live tick into the matching row.
    ///
    /// Rows for instruments not yet in the watchlist are ignored (the host
    /// subscribes explicitly via [`Self::upsert_row`]).
    pub fn ingest_tick(&mut self, tick: &Tick) {
        let Some(price) = tick.last_price() else {
            return;
        };
        let instrument = tick.instrument().clone();
        self.update_price(&instrument, price.value());
    }

    /// Fold a completed bar into the matching row, updating last price and
    /// appending the close to the row's sparkline.
    pub fn ingest_bar(&mut self, bar: &Bar) {
        let instrument = bar.instrument.clone();
        let Some(row) = self
            .view
            .rows
            .iter_mut()
            .find(|r| r.instrument == instrument)
        else {
            return;
        };
        let previous = self
            .previous_close
            .get(&instrument)
            .copied()
            .unwrap_or(bar.close.value());
        row.last_price = bar.close;
        row.delta = delta_from(bar.close.value(), previous);
        row.spark.push(bar.close.value());
        if row.spark.len() > 64 {
            row.spark.remove(0);
        }
        self.previous_close.insert(instrument, bar.close.value());
    }

    /// Insert or replace a watchlist row.
    ///
    /// Returns `true` when an existing row was replaced.
    pub fn upsert_row(&mut self, row: WatchlistRow) -> bool {
        self.previous_close
            .entry(row.instrument.clone())
            .or_insert(row.last_price.value());
        self.view.upsert(row)
    }

    /// Remove an instrument from the watchlist.
    pub fn remove(&mut self, instrument: &InstrumentId) -> bool {
        self.previous_close.remove(instrument);
        self.view.remove(instrument)
    }

    /// The number of rows currently shown.
    pub fn row_count(&self) -> usize {
        self.view.row_count()
    }

    fn update_price(&mut self, instrument: &InstrumentId, price: Decimal) {
        let Some(row) = self
            .view
            .rows
            .iter_mut()
            .find(|r| &r.instrument == instrument)
        else {
            return;
        };
        let previous = self
            .previous_close
            .get(instrument)
            .copied()
            .unwrap_or(price);
        row.delta = delta_from(price, previous);
        row.last_price = Price::new(price);
        self.previous_close.insert(instrument.clone(), price);
    }
}

impl Default for WatchlistFeature {
    fn default() -> Self {
        Self::new()
    }
}

/// Compute a delta from `current` vs `previous`.
fn delta_from(current: Decimal, previous: Decimal) -> Delta {
    if previous.is_zero() {
        return Delta::new(Decimal::ZERO, Decimal::ZERO);
    }
    let absolute = current - previous;
    let pct = absolute / previous * Decimal::from(100);
    Delta::new(absolute, pct)
}
