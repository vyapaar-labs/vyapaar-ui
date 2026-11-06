//! Technical indicators over exact Decimal series.

use rust_decimal::Decimal;

use crate::bindings::ApiError;

/// Simple moving average. Returns `None` for positions where fewer than
/// `period` values precede (inclusive) the window.
pub fn sma(values: &[Decimal], period: usize) -> Result<Vec<Option<Decimal>>, ApiError> {
    if period == 0 {
        return Err(ApiError::InvalidArgument("period must be > 0".to_string()));
    }
    let mut out = Vec::with_capacity(values.len());
    let mut window_sum = Decimal::ZERO;
    for (i, &v) in values.iter().enumerate() {
        window_sum += v;
        if i >= period {
            window_sum -= values[i - period];
        }
        if i + 1 >= period {
            out.push(Some(window_sum / Decimal::from(period)));
        } else {
            out.push(None);
        }
    }
    Ok(out)
}

/// Exponential moving average with smoothing factor `2 / (period + 1)`.
///
/// Output has the same length as the input. The first value seeds the EMA;
/// every subsequent value applies the recursion.
pub fn ema(values: &[Decimal], period: usize) -> Result<Vec<Decimal>, ApiError> {
    if period == 0 {
        return Err(ApiError::InvalidArgument("period must be > 0".to_string()));
    }
    if values.is_empty() {
        return Ok(Vec::new());
    }
    let k = Decimal::TWO / Decimal::from(period + 1);
    let mut out = Vec::with_capacity(values.len());
    out.push(values[0]);
    for &v in &values[1..] {
        let prev = *out.last().unwrap();
        out.push((v - prev) * k + prev);
    }
    Ok(out)
}

/// Wilder Relative Strength Index (period-based, default smoothing).
pub fn rsi(values: &[Decimal], period: usize) -> Result<Vec<Option<Decimal>>, ApiError> {
    if period == 0 {
        return Err(ApiError::InvalidArgument("period must be > 0".to_string()));
    }
    if values.len() < period + 1 {
        return Ok(values.iter().map(|_| None).collect());
    }
    let mut out = vec![None; values.len()];
    // First average gain/loss over the initial window.
    let mut gains = Decimal::ZERO;
    let mut losses = Decimal::ZERO;
    for w in values[1..=period].windows(2) {
        let d = w[1] - w[0];
        if d >= Decimal::ZERO {
            gains += d;
        } else {
            losses -= d;
        }
    }
    let period_d = Decimal::from(period);
    let mut avg_gain = gains / period_d;
    let mut avg_loss = losses / period_d;
    out[period] = Some(rsi_from_averages(avg_gain, avg_loss));
    // Wilder smoothing for the rest.
    for i in period + 1..values.len() {
        let d = values[i] - values[i - 1];
        let gain = if d > Decimal::ZERO { d } else { Decimal::ZERO };
        let loss = if d < Decimal::ZERO { -d } else { Decimal::ZERO };
        avg_gain = (avg_gain * (period_d - Decimal::ONE) + gain) / period_d;
        avg_loss = (avg_loss * (period_d - Decimal::ONE) + loss) / period_d;
        out[i] = Some(rsi_from_averages(avg_gain, avg_loss));
    }
    Ok(out)
}

fn rsi_from_averages(avg_gain: Decimal, avg_loss: Decimal) -> Decimal {
    if avg_loss.is_zero() {
        return Decimal::from(100);
    }
    let rs = avg_gain / avg_loss;
    Decimal::from(100) - (Decimal::from(100) / (Decimal::ONE + rs))
}
