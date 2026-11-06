//! Portfolio / positions view primitives.
//!
//! A thin presentational mapping of the canonical `vyapaar_core::portfolio`
//! wire contract (`PortfolioSnapshot`) into rows a screen can render. No
//! domain logic lives here (per
//! `docs/plans/0005-fincept-terminal-screen-inventory.md` §2, the portfolio
//! screen is a view over `vyapaar-ledger-portfolio`'s wire output). All
//! values are exact `Decimal`s.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use vyapaar_core::instrument::PositionSide;
use vyapaar_core::portfolio::PortfolioSnapshot;
use vyapaar_core::types::{InstrumentId, Money, Price, Quantity};

/// One rendered open position row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortfolioPositionRow {
    /// The instrument held.
    pub instrument: InstrumentId,
    /// Net direction.
    pub side: PositionSide,
    /// Net quantity held.
    pub quantity: Quantity,
    /// Volume-weighted average entry price.
    pub avg_price: Price,
    /// Unrealized mark-to-market PnL.
    pub unrealized_pnl: Money,
    /// Realized PnL today.
    pub realized_pnl: Money,
}

impl PortfolioPositionRow {
    /// Construct a new position row.
    pub fn new(
        instrument: InstrumentId,
        side: PositionSide,
        quantity: Quantity,
        avg_price: Price,
        unrealized_pnl: Money,
        realized_pnl: Money,
    ) -> Self {
        Self {
            instrument,
            side,
            quantity,
            avg_price,
            unrealized_pnl,
            realized_pnl,
        }
    }
}

/// One rendered T+1 delivery holding row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortfolioHoldingRow {
    /// The instrument held.
    pub instrument: InstrumentId,
    /// Quantity held.
    pub quantity: Quantity,
    /// Volume-weighted average acquisition price.
    pub avg_price: Price,
    /// Current mark-to-market value of the holding.
    pub current_value: Money,
}

impl PortfolioHoldingRow {
    /// Construct a new holding row.
    pub fn new(
        instrument: InstrumentId,
        quantity: Quantity,
        avg_price: Price,
        current_value: Money,
    ) -> Self {
        Self {
            instrument,
            quantity,
            avg_price,
            current_value,
        }
    }
}

/// Account-level totals shown on the portfolio screen.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortfolioTotals {
    /// Free cash balance.
    pub cash: Money,
    /// Margin blocked by open positions/orders.
    pub margin_used: Money,
    /// Margin still available.
    pub margin_available: Money,
    /// Total PnL (realized + unrealized).
    pub total_pnl: Money,
    /// Net worth (cash + mark-to-market value).
    pub net_worth: Money,
}

impl PortfolioTotals {
    /// Zero totals in INR.
    pub fn zero() -> Self {
        Self {
            cash: Money::zero_inr(),
            margin_used: Money::zero_inr(),
            margin_available: Money::zero_inr(),
            total_pnl: Money::zero_inr(),
            net_worth: Money::zero_inr(),
        }
    }
}

/// The full presentational state of a portfolio / positions screen.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortfolioViewModel {
    /// Open positions.
    pub positions: Vec<PortfolioPositionRow>,
    /// T+1 delivery holdings.
    pub holdings: Vec<PortfolioHoldingRow>,
    /// Account-level totals.
    pub totals: PortfolioTotals,
}

impl PortfolioViewModel {
    /// An empty portfolio view.
    pub fn empty() -> Self {
        Self {
            positions: Vec::new(),
            holdings: Vec::new(),
            totals: PortfolioTotals::zero(),
        }
    }

    /// Project a canonical [`PortfolioSnapshot`] into presentational rows.
    ///
    /// This is a pure, structural mapping — it performs no arithmetic, it only
    /// re-lays the wire contract's fields into render-ready shapes.
    pub fn from_snapshot(snapshot: &PortfolioSnapshot) -> Self {
        let positions = snapshot
            .positions
            .iter()
            .map(|p| {
                PortfolioPositionRow::new(
                    p.instrument.clone(),
                    p.side,
                    p.quantity,
                    p.avg_price,
                    p.unrealized_pnl,
                    p.realized_pnl,
                )
            })
            .collect();
        let holdings = snapshot
            .holdings
            .iter()
            .map(|h| {
                PortfolioHoldingRow::new(
                    h.instrument.clone(),
                    h.quantity,
                    h.avg_price,
                    h.current_value,
                )
            })
            .collect();
        Self {
            positions,
            holdings,
            totals: PortfolioTotals {
                cash: snapshot.totals.cash,
                margin_used: snapshot.totals.margin_used,
                margin_available: snapshot.totals.margin_available,
                total_pnl: snapshot.totals.total_pnl,
                net_worth: snapshot.totals.net_worth,
            },
        }
    }

    /// Whether there is anything to render.
    pub fn is_empty(&self) -> bool {
        self.positions.is_empty() && self.holdings.is_empty()
    }

    /// Sum of unrealized PnL across all positions, as an INR amount.
    pub fn unrealized_pnl_total(&self) -> Decimal {
        self.positions
            .iter()
            .fold(Decimal::ZERO, |acc, p| acc + p.unrealized_pnl.amount)
    }

    /// Sum of realized PnL across all positions, as an INR amount.
    pub fn realized_pnl_total(&self) -> Decimal {
        self.positions
            .iter()
            .fold(Decimal::ZERO, |acc, p| acc + p.realized_pnl.amount)
    }
}
