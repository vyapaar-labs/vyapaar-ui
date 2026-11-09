//! Headless `vyapaar scan` wiring (gap `0001` closed; gap doc removed).
//!
//! Parses the CLI query into a canonical [`vyapaar_core::screener::ScreenerScanRequest`],
//! evaluates it against an in-memory fixture universe through
//! [`vyapaar_ui_api_wasm::screener::apply_scan`], and returns the matching row count.
//! The fixture stands in for a live `vyapaar-runtime-api-rest` `GET /screener/scan`
//! universe until a headless data source is plumbed in.

use std::collections::BTreeMap;

use rust_decimal::Decimal;
use vyapaar_core::screener::{
    FilterOp, MetricKeySpec, MetricRef, ScreenerFilterGroup, ScreenerFilterPredicate,
    ScreenerScanRequest,
};
use vyapaar_core::types::{Exchange, InstrumentId};
use vyapaar_ui_api_wasm::screener::apply_scan;

use crate::args::ScanArgs;
use crate::runner::{CliError, CommandOutcome};

/// Fixture universe of NSE instruments with scalar metrics.
///
/// Mirrors the shape of a `vyapaar_runtime_api_rest` screener response row
/// (`instrument` + metric name → `Decimal`). Kept `Decimal`-only: zero floats.
/// `pub` so integration tests can assert against a known universe.
pub fn scan_universe() -> Vec<vyapaar_core::screener::ScreenerRow> {
    let row = |symbol: &str, pe: u64, roe: u64, market_cr: u64| {
        let instrument = InstrumentId::new(Exchange::Nse, symbol);
        let mut metrics = BTreeMap::new();
        metrics.insert("pe".to_string(), Decimal::from(pe));
        metrics.insert("roe".to_string(), Decimal::from(roe));
        metrics.insert("market_cap".to_string(), Decimal::from(market_cr));
        vyapaar_core::screener::ScreenerRow {
            instrument,
            metrics,
        }
    };

    vec![
        row("RELIANCE", 25, 15, 200_000_000),
        row("TCS", 30, 20, 150_000_000),
        row("INFY", 28, 18, 80_000_000),
        row("HDFCBANK", 22, 12, 120_000_000),
    ]
}

/// Build a [`MetricRef`] for a metric name at the daily granularity.
fn metric_ref(name: &str) -> MetricRef {
    MetricRef(MetricKeySpec {
        metric: name.to_string(),
        period: vyapaar_core::signal::Timeframe::D1,
    })
}

/// Parse a screener query into a [`ScreenerScanRequest`].
///
/// Supported syntax:
/// - `metric<op>value` — single predicate, `op` in `{>, <, >=, <=, ==, !=}`
/// - `metric between <low> <high>` — inclusive range predicate
/// - `pred1 and pred2` — AND-combined group (`match_all = true`)
///
/// Metric values are [`Decimal`] — zero floats.
pub fn parse_query(query: &str) -> Result<ScreenerScanRequest, String> {
    let query = query.trim();
    if query.is_empty() {
        return Err("query cannot be empty".to_string());
    }

    let parts: Vec<&str> = query
        .split(" and ")
        .map(|p| p.trim())
        .filter(|p| !p.is_empty())
        .collect();
    if parts.is_empty() {
        return Err("query cannot be empty".to_string());
    }

    let mut predicates = Vec::with_capacity(parts.len());
    for part in parts {
        let part = part.trim();
        if let Some((metric, rest)) = part.split_once(" between ") {
            let metric = metric.trim();
            let mut bounds = rest.split_whitespace();
            let low = bounds
                .next()
                .ok_or_else(|| format!("missing lower bound for 'between' on '{metric}'"))?;
            let high = bounds
                .next()
                .ok_or_else(|| format!("missing upper bound for 'between' on '{metric}'"))?;
            if bounds.next().is_some() {
                return Err(format!("too many bounds for 'between' on '{metric}'"));
            }
            let value: Decimal = low
                .parse()
                .map_err(|_| format!("invalid lower bound '{low}' for '{metric}'"))?;
            let value_upper: Decimal = high
                .parse()
                .map_err(|_| format!("invalid upper bound '{high}' for '{metric}'"))?;
            predicates.push(ScreenerFilterPredicate {
                metric: metric_ref(metric),
                op: FilterOp::Between,
                value,
                value_upper: Some(value_upper),
            });
            continue;
        }

        // Two-char operators first so `>=`/`<=`/`==`/`!=` win over `>`/`<`.
        let (metric, value_str, op) = if let Some((m, v)) = part.split_once(">=") {
            (m.trim(), v.trim(), FilterOp::Gte)
        } else if let Some((m, v)) = part.split_once("<=") {
            (m.trim(), v.trim(), FilterOp::Lte)
        } else if let Some((m, v)) = part.split_once("==") {
            (m.trim(), v.trim(), FilterOp::Eq)
        } else if let Some((m, v)) = part.split_once("!=") {
            (m.trim(), v.trim(), FilterOp::Ne)
        } else if let Some((m, v)) = part.split_once(">") {
            (m.trim(), v.trim(), FilterOp::Gt)
        } else if let Some((m, v)) = part.split_once("<") {
            (m.trim(), v.trim(), FilterOp::Lt)
        } else {
            return Err(format!("no comparison operator in predicate '{part}'"));
        };
        let value: Decimal = value_str
            .parse()
            .map_err(|_| format!("invalid value '{value_str}' for '{metric}'"))?;
        predicates.push(ScreenerFilterPredicate {
            metric: metric_ref(metric),
            op,
            value,
            value_upper: None,
        });
    }

    Ok(ScreenerScanRequest {
        universe: None, // full fixture universe
        filters: ScreenerFilterGroup {
            predicates,
            match_all: true,
        },
        sort: None,
        limit: None,
    })
}

/// Run a `vyapaar scan` against the fixture universe.
pub fn run_scan(query: &str) -> Result<CommandOutcome, CliError> {
    let request = parse_query(query).map_err(CliError::Execution)?;
    let universe = scan_universe();
    let response = apply_scan(&universe, &request)
        .map_err(|e| CliError::Execution(format!("screener evaluation failed: {e:?}")))?;
    Ok(CommandOutcome::ScanResults {
        rows: response.rows.len(),
    })
}

/// Convenience wrapper over [`ScanArgs`].
pub fn run_scan_args(args: &ScanArgs) -> Result<CommandOutcome, CliError> {
    run_scan(args.query.trim())
}
