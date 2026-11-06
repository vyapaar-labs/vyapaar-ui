//! Portfolio analytics: Sharpe, Sortino, and drawdown over exact Decimal
//! return series.

use rust_decimal::Decimal;
use rust_decimal::MathematicalOps;

use crate::bindings::ApiError;

/// Compute the annualized Sharpe ratio of a per-period return series.
///
/// Uses the sample (n−1) standard deviation. Returns [`ApiError::InsufficientData`]
/// when there are fewer than 2 returns or the standard deviation is zero.
pub fn sharpe_ratio(returns: &[Decimal], periods_per_year: u32) -> Result<Decimal, ApiError> {
    if returns.len() < 2 {
        return Err(ApiError::InsufficientData);
    }
    if periods_per_year == 0 {
        return Err(ApiError::InvalidArgument(
            "periods_per_year must be > 0".to_string(),
        ));
    }
    let mean = mean(returns);
    let variance = variance(returns, mean)?;
    let Some(std) = variance.sqrt() else {
        return Err(ApiError::InsufficientData);
    };
    if std.is_zero() {
        return Err(ApiError::InsufficientData);
    }
    let scale = Decimal::from(periods_per_year)
        .sqrt()
        .unwrap_or_default();
    Ok((mean / std) * scale)
}

/// Compute the Sortino ratio using only downside deviation.
pub fn sortino_ratio(returns: &[Decimal], periods_per_year: u32) -> Result<Decimal, ApiError> {
    if returns.len() < 2 {
        return Err(ApiError::InsufficientData);
    }
    if periods_per_year == 0 {
        return Err(ApiError::InvalidArgument(
            "periods_per_year must be > 0".to_string(),
        ));
    }
    let mean = mean(returns);
    let downside = downside_deviation(returns, mean)?;
    if downside.is_zero() {
        return Err(ApiError::InsufficientData);
    }
    let scale = Decimal::from(periods_per_year)
        .sqrt()
        .unwrap_or_default();
    Ok((mean / downside) * scale)
}

/// Maximum drawdown of an equity/price series (as a positive fraction, e.g.
/// `0.25` for a 25% drawdown). Returns 0 for a monotonic (never declining)
/// series.
pub fn drawdown(equity: &[Decimal]) -> Result<Decimal, ApiError> {
    if equity.is_empty() {
        return Err(ApiError::InsufficientData);
    }
    let mut peak = equity[0];
    let mut max_dd = Decimal::ZERO;
    for &v in equity {
        if v > peak {
            peak = v;
        }
        if peak.is_zero() {
            continue;
        }
        let dd = (peak - v) / peak;
        if dd > max_dd {
            max_dd = dd;
        }
    }
    Ok(max_dd)
}

fn mean(values: &[Decimal]) -> Decimal {
    values.iter().sum::<Decimal>() / Decimal::from(values.len())
}

fn variance(values: &[Decimal], mean: Decimal) -> Result<Decimal, ApiError> {
    let sum_sq = values
        .iter()
        .map(|v| {
            let d = *v - mean;
            d * d
        })
        .sum::<Decimal>();
    Ok(sum_sq / Decimal::from(values.len().saturating_sub(1)))
}

fn downside_deviation(values: &[Decimal], mean: Decimal) -> Result<Decimal, ApiError> {
    let mut sum_sq = Decimal::ZERO;
    let mut count = 0u32;
    for v in values {
        if *v < mean {
            let d = mean - *v;
            sum_sq += d * d;
            count += 1;
        }
    }
    if count == 0 {
        return Err(ApiError::InsufficientData);
    }
    Ok((sum_sq / Decimal::from(count)).sqrt().unwrap_or_default())
}
