//! Module to set up and wrap the Rustbreak db we are using.
//! By: Curtis Jones <mail@curtisjones.ca>
//! Started on: November 8, 2020

#[cfg(feature = "bincode")]
use super::util::Bincode;
#[cfg(feature = "default")]
use super::util::Ron;
#[cfg(feature = "yaml")]
use super::util::Yaml;
use super::{
    config::Config,
    util::{
        Account, AccountBalance, AccountName, AccountNumber, AccountPosition, Arc, DateTime,
        Deserialize, HashMap, NaiveDate, NaiveTime, PathBuf, PathDatabase, Result, Serialize, Utc,
        Weak,
    },
};

pub type DBRef = Arc<DB>;
pub type DBRefWeak = Weak<DB>;

#[derive(Debug)]
#[cfg(feature = "default")]
/// Only need to make one of these.
/// The big abstraction to store and retriveve Data.
/// Depending on the features it can have a Ron storage backend,
/// a Yaml storage backend, or a Bincode backend.
pub struct DB {
    db: PathDatabase<DBInfo, Ron>,
}
#[derive(Debug)]
#[cfg(feature = "yaml")]
/// To enable this you would have to disable default and enable 'yaml'.
pub struct DB {
    db: PathDatabase<DBInfo, Yaml>,
}
#[derive(Debug)]
#[cfg(feature = "bincode")]
/// To enable this you would have to disable default and enable 'bincode'.
pub struct DB {
    db: PathDatabase<DBInfo, Bincode>,
}

impl DB {
    /// Should only be called once. Loads a path database at the path provided.
    pub fn new(config: &Config) -> Result<Self> {
        Ok(DB {
            db: PathDatabase::load_from_path_or_default(PathBuf::from(
                &config.settings.db_file_path,
            ))?,
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum DBInsertType {
    Account,
    AccountBalance,
    AccountPosition,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
/// This is the struct that represents the actual database.
/// The abstraction above is what gives all the Read/Write protection.
pub struct DBInfo {
    accounts: HashMap<AccountName, Account>,
    account_balances: HashMap<AccountNumber, HashMap<NaiveDate, DBInfoAccountBalance>>,
    account_positions: HashMap<AccountNumber, HashMap<NaiveDate, DBInfoAccountPosition>>,
}

impl DBInfo {
    #[allow(dead_code)]
    /// Default function for Rustbreak to use if it can't find a file.
    /// I will never actually call it so we allow it to be "dead" per rust-analyzers opinion.
    fn default() -> Self {
        DBInfo {
            accounts: HashMap::new(),
            account_balances: HashMap::new(),
            account_positions: HashMap::new(),
        }
    }
    fn insert_account(&mut self, number: AccountNumber, account: Account) -> Result<()> {
        Ok(())
    }
    fn insert_account_balance(
        &mut self,
        datetime: DateTime<Utc>,
        balance: AccountBalance,
    ) -> Result<()> {
        Ok(())
    }
    fn insert_account_position(
        &mut self,
        datetime: DateTime<Utc>,
        position: AccountPosition,
    ) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
/// Basically the first table in our db is of Balances.
/// This is the wrapper for our balances.
struct DBInfoAccountBalance {
    account_balance: AccountBalance,
    time_retrieved: NaiveTime,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
/// Second item is positions.
/// This is the wrapper for our positions.
struct DBInfoAccountPosition {
    account_position: AccountPosition,
    time_retrieved: NaiveTime,
}
