//! Unit tests for `vyapaar-ui-assets` fixtures.

use rust_decimal::Decimal;
use vyapaar_core::market::Tick;
use vyapaar_ui_assets::fixtures::{
    filter_chips, portfolio_snapshot, reliance, reliance_bars, reliance_quote_tick, screener_request,
    tcs_depth_tick,
};

#[test]
fn reliance_bars_are_deterministic() {
    let bars = reliance_bars(5);
    assert_eq!(bars.len(), 5);
    assert_eq!(bars[0].instrument, reliance());
    for b in &bars {
        assert!(b.high >= b.open);
        assert!(b.low <= b.open);
        assert!(!b.volume.value().is_sign_negative());
    }
    // Same call twice → identical series.
    let again = reliance_bars(5);
    assert_eq!(bars, again);
}

#[test]
fn quote_tick_carries_price() {
    let tick = reliance_quote_tick();
    assert_eq!(tick.instrument(), &reliance());
    match tick {
        Tick::Quote { bid, ask, .. } => {
            assert!(bid < ask);
        }
        _ => panic!("expected a quote tick"),
    }
}

#[test]
fn depth_tick_has_bids_and_asks() {
    let tick = tcs_depth_tick();
    match tick {
        Tick::Depth { bids, asks, .. } => {
            assert!(!bids.is_empty());
            assert!(!asks.is_empty());
            // bids sorted best-first: best bid price >= second best
            assert!(bids[0].price >= bids[1].price);
            assert!(asks[0].price <= asks[1].price);
        }
        _ => panic!("expected a depth tick"),
    }
}

#[test]
fn portfolio_snapshot_totals_balance() {
    let snap = portfolio_snapshot();
    assert_eq!(snap.positions.len(), 2);
    let totals = &snap.totals;
    assert!(totals.margin_available.amount > Decimal::ZERO);
    // margin used + available should not exceed cash + net worth sanity check
    assert!(totals.cash.amount >= totals.margin_used.amount);
}

#[test]
fn screener_fixtures_build_valid_request() {
    let req = screener_request();
    assert_eq!(req.universe.as_ref().map(Vec::len), Some(2));
    assert_eq!(req.filters.predicates.len(), 2);
    assert!(req.filters.match_all);

    let chips = filter_chips();
    assert_eq!(chips.len(), 2);
    assert_eq!(chips[0].metric, "roe");
    assert_eq!(chips[1].value_upper, Some(Decimal::from(30)));
}
