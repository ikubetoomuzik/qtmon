//! Module to set up and wrap the Rustbreak db we are using.
//! By: Curtis Jones <mail@curtisjones.ca>
//! Started on: November 8, 2020

#[cfg(feature = "bincode")]
use super::include::Bincode;
#[cfg(feature = "default")]
use super::include::Ron;
#[cfg(feature = "yaml")]
use super::include::Yaml;
use super::{
    config::Config,
    include::{
        hash_map, Account, AccountBalance, AccountName, AccountNumber, AccountPosition, Arc,
        Currency, DateTime, Deserialize, Duration, HashMap, Local, NaiveDate, NaiveTime, PathBuf,
        PathDatabase, PositionSymbol, Result, Serialize,
    },
    myerrors::{DBInsertError, DBRetrieveError},
};

/// Helper functions
fn make_dateime_naive(datetime: DateTime<Local>) -> (NaiveDate, NaiveTime) {
    let datetime = datetime.naive_local();
    (datetime.date(), datetime.time())
}
fn duration_abs(dur: Duration) -> Duration {
    if dur < Duration::zero() {
        -dur
    } else {
        dur
    }
}

pub type DBRef = Arc<DB>;

#[derive(Debug)]
#[cfg(feature = "default")]
/// Only need to make one of these.
/// The big abstraction to store and retriveve Data.
/// Depending on the features it can have a Ron storage backend,
/// a Yaml storage backend, or a Bincode backend.
pub struct DB {
    pub db: PathDatabase<DBInfo, Ron>,
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

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
/// This is the struct that represents the actual database.
/// The abstraction above is what gives all the Read/Write protection.
pub struct DBInfo {
    accounts: HashMap<AccountName, Account>,
    account_balances: HashMap<AccountNumber, DBInfoAccountBalanceCollection>,
    account_positions: HashMap<AccountNumber, DBInfoAccountPositionCollection>,
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

    // *** Insert Functions ***
    pub fn insert_account(&mut self, name: AccountName, account: Account) -> Result<()> {
        if self.accounts.keys().any(|k| *k == name) {
            Err(Box::new(DBInsertError::InsertAccountDuplicateNameError))
        } else if self.accounts.values().any(|v| *v == account) {
            Err(Box::new(DBInsertError::InsertAccountDuplicateInfoError))
        } else {
            self.accounts.insert(name, account);
            Ok(())
        }
    }
    pub fn insert_account_balance(
        &mut self,
        datetime: DateTime<Local>,
        number: &AccountNumber,
        balance: AccountBalance,
        sod: AccountBalance,
    ) -> Result<()> {
        // Seperate the date and time into their easily serializable parts.
        let (date, time) = make_dateime_naive(datetime);
        // Get the info related to the account we are trying to insert for. If it's not there
        // something is definitely wrong so we send up an error.
        if None == self.accounts.values().find(|val| val.number == *number) {
            return Err(Box::new(DBInsertError::InsertAccountBalanceNoAccountError));
        }
        let acct_bal = match self.account_balances.get_mut(number) {
            Some(abs) => abs,
            None => {
                self.account_balances
                    .insert(number.clone(), DBInfoAccountBalanceCollection::new());
                self.account_balances.get_mut(number).unwrap()
            }
        };
        match acct_bal.iter_mut().find(|ab| ab.date == date) {
            // If we already have a trace of the balances going then we insert the current we were
            // given.
            Some(abd) => abd.insert_bal(DBInfoAccountBalance::new(balance, time))?,
            // If we do not have a trace going yet then we insert the sod balance we have and
            // create a new trace.
            None => {
                let mut new_day =
                    DBInfoAccountBalanceDay::new(date, DBInfoAccountBalance::new(sod, time));
                new_day.insert_bal(DBInfoAccountBalance::new(balance, time))?;
                acct_bal.push(new_day);
            }
        }
        Ok(())
    }
    pub fn insert_account_position(
        &mut self,
        datetime: DateTime<Local>,
        number: &AccountNumber,
        position: AccountPosition,
    ) -> Result<()> {
        let (date, time) = make_dateime_naive(datetime);
        if None == self.accounts.values().find(|val| val.number == *number) {
            return Err(Box::new(DBInsertError::InsertAccountPositionNoAccountError));
        }
        let acct_map = match self.account_positions.get_mut(number) {
            Some(am) => am,
            None => {
                self.account_positions
                    .insert(number.clone(), DBInfoAccountPositionCollection::new());
                self.account_positions.get_mut(number).unwrap()
            }
        };
        match acct_map.get_mut(&position.symbol) {
            Some(pm) => match pm.get_mut(&date) {
                Some(pl) => {
                    let temp = DBInfoAccountPosition::new(position, time);
                    if !pl.iter().any(|p| *p == temp) {
                        pl.push(temp);
                        // make sure the list stays sorted from earliest to latest.
                        pl.sort_unstable_by(|a, b| a.time_retrieved.cmp(&b.time_retrieved));
                        Ok(())
                    } else {
                        Err(Box::new(DBInsertError::InsertAccountPositionDuplicateError))
                    }
                }
                None => {
                    pm.insert(date, vec![DBInfoAccountPosition::new(position, time)]);
                    Ok(())
                }
            },
            None => {
                let symbol = position.symbol.clone();
                let mut day = DBInfoAccountPositionDay::new();
                day.insert(date, vec![DBInfoAccountPosition::new(position, time)]);
                acct_map.insert(symbol, day);
                Ok(())
            }
        }
    }

    // *** Reading functions. ***
    pub fn iter_accounts(&self) -> hash_map::Values<'_, String, Account> {
        self.accounts.values()
    }

    // ** get balance info **
    pub fn get_latest_balance(
        &self,
        account_identifier: &str,
        date: NaiveDate,
    ) -> Result<DBInfoAccountBalance> {
        // first we verify that we have a valid account identifier and reduce it to just a number.
        let account_number = match self.accounts.get(account_identifier) {
            // if we find an account by name then we just get the number and return it
            Some(acct) => &acct.number,
            None => {
                // next we check if the value matches any of the numbers for the accounts we have
                // synced.
                if None
                    == self
                        .accounts
                        .values()
                        .find(|acct| acct.number == account_identifier)
                {
                    // if we find nothing then we return an error explaining that
                    return Err(Box::new(
                        DBRetrieveError::RetrieveAccountBalanceNoAccountError,
                    ));
                } else {
                    // If it matched anything we just return that number.
                    account_identifier
                }
            }
        };
        // now that we have a valid account number we use it to pull the balance collection.
        match self.account_balances.get(account_number) {
            // in most cases we expect to find a collection.
            Some(acct_bal_collection) => {
                // now we go to find the daybalance info for today
                match acct_bal_collection.iter().find(|abd| abd.date == date) {
                    // the list stays sorted its sorted every insert, so we just grab last val
                    Some(todays_bal) => Ok(todays_bal.get_most_recent()),
                    // if we can't find anything then we raise an error and attach a date
                    None => Err(Box::new(
                        DBRetrieveError::RetrieveAccountBalanceNotSyncedDayError(date),
                    )),
                }
            }
            // if we find no collection then we raise an error to indicate so
            None => Err(Box::new(
                DBRetrieveError::RetrieveAccountBalanceNotSyncedError,
            )),
        }
    }

    // ** get balance info **
    pub fn get_closest_balance(
        &self,
        account_identifier: &str,
        date: NaiveDate,
        time: NaiveTime,
    ) -> Result<DBInfoAccountBalance> {
        // first we verify that we have a valid account identifier and reduce it to just a number.
        let account_number = match self.accounts.get(account_identifier) {
            // if we find an account by name then we just get the number and return it
            Some(acct) => &acct.number,
            None => {
                // next we check if the value matches any of the numbers for the accounts we have
                // synced.
                if None
                    == self
                        .accounts
                        .values()
                        .find(|acct| acct.number == account_identifier)
                {
                    // if we find nothing then we return an error explaining that
                    return Err(Box::new(
                        DBRetrieveError::RetrieveAccountBalanceNoAccountError,
                    ));
                } else {
                    // If it matched anything we just return that number.
                    account_identifier
                }
            }
        };
        // now that we have a valid account number we use it to pull the balance collection.
        match self.account_balances.get(account_number) {
            // in most cases we expect to find a collection.
            Some(acct_bal_collection) => {
                // now we go to find the daybalance info for today
                match acct_bal_collection.iter().find(|abd| abd.date == date) {
                    // the list stays sorted its sorted every insert, so we just grab last val
                    Some(todays_bal) => {
                        // we start with the earliest bal available.
                        let mut return_bal = todays_bal.get_first_bal();
                        // then we iterate over the day's balances and stop when the absolute value
                        // of the difference starts increasing, we've already gone too far.
                        for bal in todays_bal.over_day_balances.iter().skip(1) {
                            if duration_abs(time - bal.time_retrieved)
                                <= duration_abs(time - return_bal.time_retrieved)
                            {
                                return_bal = bal.clone();
                                continue;
                            } else {
                                break;
                            }
                        }
                        Ok(return_bal)
                    }
                    // if we can't find anything then we raise an error and attach a date
                    None => Err(Box::new(
                        DBRetrieveError::RetrieveAccountBalanceNotSyncedDayError(date),
                    )),
                }
            }
            // if we find no collection then we raise an error to indicate so
            None => Err(Box::new(
                DBRetrieveError::RetrieveAccountBalanceNotSyncedError,
            )),
        }
    }
}

/// pub type def for the vector of saved day info
type DBInfoAccountBalanceCollection = Vec<DBInfoAccountBalanceDay>;

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
    fn new(balance: AccountBalance, time_retrieved: NaiveTime) -> Self {
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
struct DBInfoAccountBalanceDay {
    date: NaiveDate,
    start_of_day_bal: DBInfoAccountBalance,
    over_day_balances: Vec<DBInfoAccountBalance>,
}

impl DBInfoAccountBalanceDay {
    fn new(date: NaiveDate, start_of_day_bal: DBInfoAccountBalance) -> Self {
        Self {
            date,
            start_of_day_bal,
            over_day_balances: Vec::new(),
        }
    }
    fn insert_bal(&mut self, balance: DBInfoAccountBalance) -> Result<()> {
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

    fn get_most_recent(&self) -> DBInfoAccountBalance {
        match self.over_day_balances.last() {
            Some(b) => b.clone(),
            None => self.start_of_day_bal.clone(),
        }
    }

    fn get_first_bal(&self) -> DBInfoAccountBalance {
        match self.over_day_balances.first() {
            Some(b) => b.clone(),
            None => self.start_of_day_bal.clone(),
        }
    }
}

type DBInfoAccountPositionDay = HashMap<NaiveDate, Vec<DBInfoAccountPosition>>;
type DBInfoAccountPositionCollection = HashMap<PositionSymbol, DBInfoAccountPositionDay>;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
/// Second item is positions.
/// This is the wrapper for our positions.
struct DBInfoAccountPosition {
    symbol: PositionSymbol,
    open_quantity: f64,
    closed_quantity: f64,
    current_market_value: f64,
    current_price: f64,
    average_entry_price: f64,
    closed_pnl: f64,
    day_pnl: f64,
    open_pnl: f64,
    total_cost: f64,
    time_retrieved: NaiveTime,
}

impl DBInfoAccountPosition {
    fn new(position: AccountPosition, time_retrieved: NaiveTime) -> Self {
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
