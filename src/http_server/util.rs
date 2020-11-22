//! Module for HTTP helper functions
//! By: Curtis Jones <mail@curtisjones.ca>
//! Started on: November 22, 2020

use super::{
    super::{
        include::{json, Json, NaiveDate, NaiveTime},
        storage::{DBInfoAccountBalance, DBInfoAccountPosition},
    },
    ErrorReply,
};

// Funtion for parsing dates from strings.
pub fn parse_date(date_str: String) -> Result<NaiveDate, Json> {
    match NaiveDate::parse_from_str(&date_str, "%Y-%m-%d") {
        Ok(d) => Ok(d),
        Err(e) => Err(json(&ErrorReply::new(format!(
            "Could not parse date: {}. Error: {}",
            date_str, e
        )))),
    }
}

// Funtion for parsing time from strings.
pub fn parse_time(time_str: String) -> Result<NaiveTime, Json> {
    match NaiveTime::parse_from_str(&time_str, "%H:%M") {
        Ok(t) => Ok(t),
        Err(e) => Err(json(&ErrorReply::new(format!(
            "Could not parse date: {}. Error: {}",
            time_str, e
        )))),
    }
}

// function for parsing the input string on the /statusbar/$account_id/$input_str api
pub fn api_string_replacement(
    positions: Vec<(String, DBInfoAccountPosition)>,
    sod_balance: DBInfoAccountBalance,
    latest_balance: DBInfoAccountBalance,
    mut input_string: String,
) -> String {
    let sod_balance_replace_pairs = [
        ("%sod.cash".to_string(), sod_balance.cash.to_string()),
        (
            "%sod.marketValue".to_string(),
            format!("{:.2}", sod_balance.market_value),
        ),
        (
            "%sod.totalEquity".to_string(),
            format!("{:.2}", sod_balance.total_equity),
        ),
        (
            "%sod.maitenanceExcess".to_string(),
            format!("{:.2}", sod_balance.maitenance_excess),
        ),
    ];
    let latest_balance_replace_pairs = [
        (
            "%bal.cash".to_string(),
            format!("{:.2}", latest_balance.cash),
        ),
        (
            "%bal.cashPNL".to_string(),
            format!(
                "{:.2}",
                (latest_balance.cash - sod_balance.cash) / sod_balance.cash * 100f64
            ),
        ),
        (
            "%bal.marketValue".to_string(),
            format!("{:.2}", latest_balance.market_value),
        ),
        (
            "%bal.marketValuePNL".to_string(),
            format!(
                "{:.2}",
                (latest_balance.market_value - sod_balance.market_value) / sod_balance.market_value
                    * 100f64
            ),
        ),
        (
            "%bal.totalEquity".to_string(),
            format!("{:.2}", latest_balance.total_equity),
        ),
        (
            "%bal.totalEquityPNL".to_string(),
            format!(
                "{:.2}",
                (latest_balance.total_equity - sod_balance.total_equity) / sod_balance.total_equity
                    * 100f64
            ),
        ),
        (
            "%bal.maitenanceExcess".to_string(),
            format!("{:.2}", latest_balance.maitenance_excess),
        ),
        (
            "%bal.maitenanceExcessPNL".to_string(),
            format!(
                "{:.2}",
                (latest_balance.maitenance_excess - sod_balance.maitenance_excess)
                    / sod_balance.maitenance_excess
                    * 100f64
            ),
        ),
    ];
    // filter throught the start of day keys.
    input_string = sod_balance_replace_pairs
        .iter()
        .fold(input_string, |acc, (key, val)| acc.replace(key, val));
    // filter throught the latest keys.
    input_string = latest_balance_replace_pairs
        .iter()
        .fold(input_string, |acc, (key, val)| acc.replace(key, val));
    // generate keys for the positions and filter over the string.
    input_string = positions.iter().fold(input_string, |acc, (name, info)| {
        let sod_market_value = info.current_market_value - info.day_pnl;
        let replacement_pairs = [
            (
                format!("%{}.openQuantity", name),
                format!("{:.2}", info.open_quantity),
            ),
            (
                format!("%{}.closedQuantity", name),
                format!("{:.2}", info.closed_quantity),
            ),
            (
                format!("%{}.currentMarketValue", name),
                format!("{:.2}", info.current_market_value),
            ),
            (
                format!("%{}.sodMarketValue", name),
                format!("{:.2}", sod_market_value),
            ),
            (
                format!("%{}.currentPrice", name),
                format!("{:.2}", info.current_price),
            ),
            (
                format!("%{}.averageEntryPrice", name),
                format!("{:.2}", info.average_entry_price),
            ),
            (
                format!("%{}.openPNL", name),
                format!("{:.2}", info.open_pnl / info.total_cost * 100f64),
            ),
            (
                format!("%{}.closedPNL", name),
                format!("{:.2}", info.closed_pnl / info.total_cost * 100f64),
            ),
            (
                format!("%{}.dayPNL", name),
                format!("{:.2}", info.day_pnl / sod_market_value * 100f64),
            ),
            (
                format!("%{}.openPNLABS", name),
                format!("{:.2}", (info.open_pnl / info.total_cost * 100f64).abs()),
            ),
            (
                format!("%{}.closedPNLABS", name),
                format!("{:.2}", (info.closed_pnl / info.total_cost * 100f64).abs()),
            ),
            (
                format!("%{}.dayPNLABS", name),
                format!("{:.2}", (info.day_pnl / sod_market_value * 100f64).abs()),
            ),
            (
                format!("%{}.totalCost", name),
                format!("{:.2}", info.total_cost),
            ),
        ];
        // filter using the generated pairs
        replacement_pairs
            .iter()
            .fold(acc, |ac, (k, v)| ac.replace(k, v))
    });
    // final replacements we do are the specific char escapes
    input_string = input_string.replace('_', " ");
    input_string = input_string.replace("%dollar", "$");
    input_string = input_string.replace("%slash", "/");
    // return whatver the input string was.
    input_string
}
