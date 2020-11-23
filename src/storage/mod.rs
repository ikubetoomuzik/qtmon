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
        error, hash_map, warn, Account, AccountBalance, AccountName, AccountNumber,
        AccountPosition, Arc, DateTime, Deserialize, Duration, HashMap, Local, NaiveDate,
        NaiveTime, PathBuf, PathDatabase, Result, Serialize,
    },
};

/// Sub modules
mod balance;
mod errors;
mod position;

/// Re-export sub-modules so we can read from them in other modules.
pub use balance::*;
pub use errors::*;
pub use position::*;

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
    pub db: PathDatabase<DBInfo, Yaml>,
}
#[derive(Debug)]
#[cfg(feature = "bincode")]
/// To enable this you would have to disable default and enable 'bincode'.
pub struct DB {
    pub db: PathDatabase<DBInfo, Bincode>,
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
        if self
            .accounts
            .iter()
            .any(|(k, v)| *k == name && *v == account)
        {
            Ok(())
        } else if self
            .accounts
            .iter()
            .any(|(k, v)| *k == name && (*v).number != account.number)
        {
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
        // check to ensure that the number in args is a valid account number.
        if None == self.accounts.values().find(|val| val.number == *number) {
            return Err(Box::new(DBInsertError::InsertAccountBalanceNoAccountError));
        }
        // Get the info related to the account we are trying to insert for. If it's not there
        // something is definitely wrong so we send up an error.
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

    // ** get account info **
    pub fn get_account_list(&self) -> Result<Vec<String>> {
        if self.accounts.is_empty() {
            Err(Box::new(DBRetrieveError::RetrieveAccountsNotSyncedError))
        } else {
            Ok(self.accounts.keys().map(|k| k.clone()).collect())
        }
    }

    // ** get account info **
    pub fn get_account_info(&self, account_identifier: &str) -> Result<Account> {
        // first we verify that we have a valid account identifier and reduce it to just a number.
        let account_number = match self.acct_identifier_to_number(account_identifier) {
            Ok(an) => an,
            Err(e) => {
                error!("Could not retrieve balance, with error: {}", e);
                return Err(e);
            }
        };
        Ok(self
            .accounts
            .values()
            .find(|v| v.number == account_number)
            .unwrap()
            .clone())
    }

    // ** get balance info **

    pub fn get_start_of_day_balance(
        &self,
        account_identifier: &str,
        date: NaiveDate,
    ) -> Result<DBInfoAccountBalance> {
        // first we verify that we have a valid account identifier and reduce it to just a number.
        let account_number = match self.acct_identifier_to_number(account_identifier) {
            Ok(an) => an,
            Err(e) => {
                error!("Could not retrieve balance, with error: {}", e);
                return Err(e);
            }
        };
        // now that we have a valid account number we use it to pull the balance collection.
        let todays_bal = match self.list_balances_of_date(&account_number, &date) {
            Ok(tb) => tb,
            Err(e) => {
                warn!("Could not retrieve balance, with error: {}", e);
                return Err(e);
            }
        };
        // Then if all goes well we return the most recent bal.
        Ok(todays_bal.get_start_of_day().clone())
    }

    pub fn get_latest_balance(
        &self,
        account_identifier: &str,
        date: NaiveDate,
    ) -> Result<DBInfoAccountBalance> {
        // first we verify that we have a valid account identifier and reduce it to just a number.
        let account_number = match self.acct_identifier_to_number(account_identifier) {
            Ok(an) => an,
            Err(e) => {
                error!("Could not retrieve balance, with error: {}", e);
                return Err(e);
            }
        };
        // now that we have a valid account number we use it to pull the balance collection.
        let todays_bal = match self.list_balances_of_date(&account_number, &date) {
            Ok(tb) => tb,
            Err(e) => {
                warn!("Could not retrieve balance, with error: {}", e);
                return Err(e);
            }
        };
        // Then if all goes well we return the most recent bal.
        Ok(todays_bal.get_most_recent().clone())
    }

    // function to find the balance closest to a date & time.
    pub fn get_closest_balance(
        &self,
        account_identifier: &str,
        date: NaiveDate,
        time: NaiveTime,
    ) -> Result<DBInfoAccountBalance> {
        // first we verify that we have a valid account identifier and reduce it to just a number.
        let account_number = match self.acct_identifier_to_number(account_identifier) {
            Ok(an) => an,
            Err(e) => {
                error!("Could not retrieve balance, with error: {}", e);
                return Err(e);
            }
        };
        // now that we have a valid account number we use it to pull the balance collection.
        let todays_bal = match self.list_balances_of_date(&account_number, &date) {
            Ok(tb) => tb,
            Err(e) => {
                warn!("Could not retrieve balance, with error: {}", e);
                return Err(e);
            }
        };
        // we start with the first bal and then check from there, but to avoid having to iterate
        // over everything we use a for loop. So we can leave whenever.
        let mut return_bal = todays_bal.get_first_bal();
        // then we iterate over the day's balances and stop when the absolute value
        // of the difference starts increasing, we've already gone too far.
        for bal in todays_bal.over_day_balances.iter().skip(1) {
            if duration_abs(time - bal.time_retrieved)
                <= duration_abs(time - return_bal.time_retrieved)
            {
                return_bal = bal;
                continue;
            } else {
                break;
            }
        }
        // Once we get to the end or break early we return whatever is in the var.
        Ok(return_bal.clone())
    }
    // function to get a list of position symbols.
    pub fn get_position_symbols(&self, acct_ident: &str) -> Result<Vec<String>> {
        // first we verify that we have a valid account identifier and reduce it to just a number.
        let account_number = match self.acct_identifier_to_number(acct_ident) {
            Ok(an) => an,
            Err(e) => {
                error!("Could not retrieve position, with error: {}", e);
                return Err(e);
            }
        };
        match self.account_positions.get(&account_number) {
            Some(ap) => Ok(ap.keys().map(|k| k.to_string()).collect()),
            None => {
                let e = Box::new(DBRetrieveError::RetrieveAccountPositionAllNotSyncedError);
                warn!("Could not retrieve position, with error: {}", e);
                Err(e)
            }
        }
    }
    pub fn get_latest_position(
        &self,
        acct_ident: &str,
        position_symbol: &str,
        date: NaiveDate,
    ) -> Result<DBInfoAccountPosition> {
        // first we verify that we have a valid account identifier and reduce it to just a number.
        let acc_num = match self.acct_identifier_to_number(acct_ident) {
            Ok(an) => an,
            Err(e) => {
                error!("Could not retrieve position, with error: {}", e);
                return Err(e);
            }
        };
        let day_list_of_positions =
            match self.list_positions_of_date(&acc_num, position_symbol, &date) {
                Ok(dlop) => dlop,
                Err(e) => {
                    warn!("Could not retrieve position, with error: {}", e);
                    return Err(e);
                }
            };
        Ok(day_list_of_positions.last().unwrap().clone())
    }
    pub fn get_closest_position(
        &self,
        acct_ident: &str,
        position_symbol: &str,
        date: NaiveDate,
        time: NaiveTime,
    ) -> Result<DBInfoAccountPosition> {
        // first we verify that we have a valid account identifier and reduce it to just a number.
        let acc_num = match self.acct_identifier_to_number(&acct_ident) {
            Ok(an) => an,
            Err(e) => {
                error!("Could not retrieve position, with error: {}", e);
                return Err(e);
            }
        };
        let day_list_of_positions =
            match self.list_positions_of_date(&acc_num, position_symbol, &date) {
                Ok(dlop) => dlop,
                Err(e) => {
                    warn!("Could not retrieve position, with error: {}", e);
                    return Err(e);
                }
            };
        let mut result = day_list_of_positions.first().unwrap();
        for pos in day_list_of_positions.iter().skip(1) {
            if duration_abs(time - pos.time_retrieved) <= duration_abs(time - result.time_retrieved)
            {
                result = pos;
                continue;
            } else {
                break;
            }
        }
        Ok(result.clone())
    }
    // ** Helper methods. **
    fn acct_identifier_to_number(&self, acct_ident: &str) -> Result<String> {
        match self.accounts.get(acct_ident) {
            Some(acct) => Ok(acct.number.clone()),
            None => {
                if self.iter_accounts().any(|ac| ac.number == acct_ident) {
                    Ok(acct_ident.to_string())
                } else {
                    Err(Box::new(DBRetrieveError::RetrieveAccountNoAccountError(
                        acct_ident.to_string(),
                    )))
                }
            }
        }
    }
    fn list_positions_of_date(
        &self,
        acct_num: &str,
        symbol: &str,
        date: &NaiveDate,
    ) -> Result<&Vec<DBInfoAccountPosition>> {
        match self.account_positions.get(acct_num) {
            Some(pos_collection) => match pos_collection.get(symbol) {
                Some(pos_days) => match pos_days.get(date) {
                    Some(pos_day) => Ok(pos_day),
                    None => Err(Box::new(
                        DBRetrieveError::RetrieveAccountPositionNotSyncedDayError(
                            symbol.to_string(),
                            date.clone(),
                        ),
                    )),
                },
                None => Err(Box::new(
                    DBRetrieveError::RetrieveAccountPositionNotSyncedError(symbol.to_string()),
                )),
            },
            None => Err(Box::new(
                DBRetrieveError::RetrieveAccountPositionAllNotSyncedError,
            )),
        }
    }
    fn list_balances_of_date(
        &self,
        acct_num: &str,
        date: &NaiveDate,
    ) -> Result<&DBInfoAccountBalanceDay> {
        match self.account_balances.get(acct_num) {
            Some(balances_days) => match balances_days.iter().find(|bd| bd.date == *date) {
                Some(bal_day) => Ok(bal_day),
                None => Err(Box::new(
                    DBRetrieveError::RetrieveAccountBalanceNotSyncedDayError(date.clone()),
                )),
            },
            None => Err(Box::new(
                DBRetrieveError::RetrieveAccountBalanceNotSyncedError,
            )),
        }
    }
}
