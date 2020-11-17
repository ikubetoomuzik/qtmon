//! Module containing the balance sub-objects.
//! By: Curtis Jones <mail@curtisjones.ca>
//! Started on: November 16, 2020

use super::super::{
    include::{AccountBalance, Currency, Deserialize, NaiveDate, NaiveTime, Result, Serialize},
    myerrors::DBInsertError,
};

/// pub type def for the vector of saved day info
pub type DBInfoAccountBalanceCollection = Vec<DBInfoAccountBalanceDay>;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct DBInfoAccountBalance {
    pub currency: Currency,
    pub cash: f64,
    pub market_value: f64,
    pub total_equity: f64,
    pub buying_power: f64,
    pub maitenance_excess: f64,
    pub time_retrieved: NaiveTime,
}

impl DBInfoAccountBalance {
    pub fn new(balance: AccountBalance, time_retrieved: NaiveTime) -> Self {
        Self {
            currency: balance.currency,
            cash: balance.cash.as_f64().unwrap(),
            market_value: balance.market_value.as_f64().unwrap(),
            total_equity: balance.total_equity.as_f64().unwrap(),
            buying_power: balance.buying_power.as_f64().unwrap(),
            maitenance_excess: balance.maintenance_excess.as_f64().unwrap(),
            time_retrieved,
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
/// Struct to contain a whole day's balances.
pub struct DBInfoAccountBalanceDay {
    pub date: NaiveDate,
    pub start_of_day_bal: DBInfoAccountBalance,
    pub over_day_balances: Vec<DBInfoAccountBalance>,
}

impl DBInfoAccountBalanceDay {
    pub fn new(date: NaiveDate, start_of_day_bal: DBInfoAccountBalance) -> Self {
        Self {
            date,
            start_of_day_bal,
            over_day_balances: Vec::new(),
        }
    }

    pub fn insert_bal(&mut self, balance: DBInfoAccountBalance) -> Result<()> {
        if self.over_day_balances.iter().any(|odb| *odb == balance) {
            Err(Box::new(DBInsertError::InsertAccountBalanceDuplicateError))
        } else {
            self.over_day_balances.push(balance);
            self.sort();
            Ok(())
        }
    }

    fn sort(&mut self) {
        self.over_day_balances
            .sort_unstable_by(|a, b| a.time_retrieved.cmp(&b.time_retrieved));
    }

    pub fn get_most_recent(&self) -> &DBInfoAccountBalance {
        match self.over_day_balances.last() {
            Some(b) => b,
            None => &self.start_of_day_bal,
        }
    }

    pub fn get_first_bal(&self) -> &DBInfoAccountBalance {
        match self.over_day_balances.first() {
            Some(b) => b,
            None => &self.start_of_day_bal,
        }
    }
}
