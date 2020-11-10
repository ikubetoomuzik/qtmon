//! Module to set up and wrap the Rustbreak db we are using.
//! By: Curtis Jones <mail@curtisjones.ca>
//! Started on: November 8, 2020

use super::{config::Config, Result};
use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use rustbreak::{self, deser::Ron};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};

type PathDatabase = rustbreak::PathDatabase<DBInfo, Ron>;

#[derive(Debug)]
pub struct DB {
    db: PathDatabase,
}

impl DB {
    pub fn new(config: &Config) -> Result<Self> {
        Ok(DB {
            db: PathDatabase::load_from_path_or_default(PathBuf::from(
                &config.settings.db_file_path,
            ))?,
        })
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct DBInfo {
    account_balances: HashMap<u16, HashMap<NaiveDate, DBInfoBalance>>,
}

impl DBInfo {
    fn default() -> Self {
        DBInfo {
            account_balances: HashMap::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct DBInfoBalance {
    bal: f64,
    start_of_day_bal: f64,
    time_retrieved: NaiveTime,
}
