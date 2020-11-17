//! Sub module to contain the account position specific info.
//! By: Curtis Jones <mail@curtisjones.ca>
//! Started on: November 16, 2020

use super::super::include::{
    AccountPosition, Deserialize, HashMap, NaiveDate, NaiveTime, PositionSymbol, Serialize,
};

pub type DBInfoAccountPositionDay = HashMap<NaiveDate, Vec<DBInfoAccountPosition>>;
pub type DBInfoAccountPositionCollection = HashMap<PositionSymbol, DBInfoAccountPositionDay>;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
/// This is the wrapper for our positions.
pub struct DBInfoAccountPosition {
    pub symbol: PositionSymbol,
    pub open_quantity: f64,
    pub closed_quantity: f64,
    pub current_market_value: f64,
    pub current_price: f64,
    pub average_entry_price: f64,
    pub closed_pnl: f64,
    pub day_pnl: f64,
    pub open_pnl: f64,
    pub total_cost: f64,
    pub time_retrieved: NaiveTime,
}

impl DBInfoAccountPosition {
    pub fn new(position: AccountPosition, time_retrieved: NaiveTime) -> Self {
        Self {
            symbol: position.symbol,
            open_quantity: position.open_quantity.as_f64().unwrap(),
            closed_quantity: position.closed_quantity.as_f64().unwrap(),
            current_market_value: position.current_market_value.as_f64().unwrap(),
            current_price: position.current_price.as_f64().unwrap(),
            average_entry_price: position.average_entry_price.as_f64().unwrap(),
            closed_pnl: position.closed_profit_and_loss.as_f64().unwrap(),
            day_pnl: position.day_profit_and_loss.as_f64().unwrap(),
            open_pnl: position.open_profit_and_loss.as_f64().unwrap(),
            total_cost: position.total_cost.as_f64().unwrap(),
            time_retrieved,
        }
    }
}
