//! Error type and the public binding facade.

use serde::{Deserialize, Serialize};

/// Errors surfaced by the API-wasm bindings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ApiError {
    /// Not enough data points for the requested computation.
    InsufficientData,
    /// An argument violated a precondition.
    InvalidArgument(String),
}

impl core::fmt::Display for ApiError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InsufficientData => write!(f, "insufficient data"),
            Self::InvalidArgument(msg) => write!(f, "invalid argument: {msg}"),
        }
    }
}

impl std::error::Error for ApiError {}

/// Result alias for the bindings.
pub type ApiResult<T> = Result<T, ApiError>;

/// The public WASM-facing binding facade.
///
/// Every method is a thin wrapper over the pure computation modules — it
/// exists so the JS bindgen layer has a single, stable surface to bind to.
#[derive(Debug, Clone, Copy, Default)]
pub struct Bindings;

impl Bindings {
    /// Create a new bindings facade.
    pub fn new() -> Self {
        Self
    }

    /// Sharpe ratio of a return series.
    pub fn sharpe_ratio(
        &self,
        returns: &[rust_decimal::Decimal],
        periods_per_year: u32,
    ) -> ApiResult<rust_decimal::Decimal> {
        crate::analytics::sharpe_ratio(returns, periods_per_year)
    }

    /// Simple moving average of a series.
    pub fn sma(
        &self,
        values: &[rust_decimal::Decimal],
        period: usize,
    ) -> ApiResult<Vec<Option<rust_decimal::Decimal>>> {
        crate::indicators::sma(values, period)
    }

    /// Margin required for an order.
    pub fn margin_required(
        &self,
        instrument: &vyapaar_core::types::InstrumentId,
        price: vyapaar_core::types::Price,
        quantity: vyapaar_core::types::Quantity,
        leverage: rust_decimal::Decimal,
    ) -> vyapaar_core::types::Money {
        crate::margin::margin_required(instrument, price, quantity, leverage)
    }
}
