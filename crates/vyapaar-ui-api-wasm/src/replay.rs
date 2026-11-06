//! Tick replay into a position view (for the simulator/backtest workbench).

use rust_decimal::Decimal;
use vyapaar_core::market::Tick;
use vyapaar_core::types::{InstrumentId, Price, Quantity};

/// A summary view of a position built by replaying ticks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PositionView {
    /// The instrument whose ticks were replayed.
    pub instrument: InstrumentId,
    /// Last seen price.
    pub last_price: Price,
    /// Total tick volume observed.
    pub total_volume: Quantity,
    /// Highest price seen.
    pub high: Price,
    /// Lowest price seen.
    pub low: Price,
}

/// Replay ticks into a [`PositionView`], tracking high/low/last and volume.
///
/// Ticks without a price (e.g. depth-only) contribute volume only when the
/// volume field carries it, otherwise they are skipped for price tracking.
pub fn replay_ticks<'a>(ticks: impl IntoIterator<Item = &'a Tick>) -> Option<PositionView> {
    let mut instrument = None;
    let mut last = None;
    let mut total_volume = Decimal::ZERO;
    let mut high = None;
    let mut low = None;

    for tick in ticks {
        let inst = tick.instrument().clone();
        instrument.get_or_insert(inst);
        let price = tick.last_price();
        if let Some(p) = price {
            let v = p.value();
            high = Some(high.map_or(v, |h: Decimal| h.max(v)));
            low = Some(low.map_or(v, |l: Decimal| l.min(v)));
            last = Some(p);
        }
        // Volume contribution varies by tick variant; the canonical helper
        // exposes LTP/quote volume through the variant fields.
        if let Some(q) = tick_volume(tick) {
            total_volume += q.value();
        }
    }

    let instrument = instrument?;
    let last_price = last?;
    Some(PositionView {
        instrument,
        last_price,
        total_volume: Quantity::new(total_volume),
        high: Price::new(high.unwrap_or(last_price.value())),
        low: Price::new(low.unwrap_or(last_price.value())),
    })
}

fn tick_volume(tick: &Tick) -> Option<Quantity> {
    match tick {
        Tick::Ltp { volume, .. } => Some(*volume),
        Tick::Quote { bid_qty, ask_qty, .. } => Some(Quantity::new(bid_qty.value() + ask_qty.value())),
        Tick::Depth { bids, asks, .. } => {
            let bid_vol = bids.iter().map(|b| b.quantity.value()).sum::<Decimal>();
            let ask_vol = asks.iter().map(|a| a.quantity.value()).sum::<Decimal>();
            Some(Quantity::new(bid_vol + ask_vol))
        }
    }
}
