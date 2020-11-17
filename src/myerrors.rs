//! Module containing the specific errors to be defined for my project.
//! By: Curtis Jones <mail@curtisjones.ca>
//! Started on: November 12, 2020

use super::include::{error, fmt, NaiveDate};

#[derive(Debug)]
// Enum representing errors that are possible during inserts into my database.
pub enum DBInsertError {
    InsertAccountDuplicateNameError,
    InsertAccountDuplicateInfoError,
    InsertAccountBalanceDuplicateError,
    InsertAccountBalanceNoAccountError,
    InsertAccountPositionDuplicateError,
    InsertAccountPositionNoAccountError,
}

impl error::Error for DBInsertError {}

impl fmt::Display for DBInsertError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InsertAccountDuplicateNameError => write!(
                f,
                "Could not insert Account into DataBase! Duplicate account name already in DB."
            ),
            Self::InsertAccountDuplicateInfoError => write!(
                f,
                "Could not insert Account into DataBase! Duplicate account info already in DB."
            ),
            Self::InsertAccountBalanceDuplicateError => write!(
                f,
                "Could not insert Account Balance into DataBase! Duplicate already in DB."
            ),
            Self::InsertAccountBalanceNoAccountError => write!(
                f,
                "Could not insert Account Balance into DataBase! Account for balance does not exist."
            ),
            Self::InsertAccountPositionDuplicateError => write!(
                f,
                "Could not insert Account Position into DataBase! Duplicate already in DB."
            ),
            Self::InsertAccountPositionNoAccountError => write!(
                f,
                "Could not insert Account Position into DataBase! Account that position belongs to does not exist."
            ),
        }
    }
}

#[derive(Debug)]
// Enum representing errors that are possible during inserts into my database.
pub enum DBRetrieveError {
    RetrieveAccountNoAccountError(String),
    RetrieveAccountPositionNotSyncedError(String),
    RetrieveAccountPositionAllNotSyncedError,
    RetrieveAccountPositionNoAccountError,
    RetrieveAccountBalanceNoAccountError,
    RetrieveAccountBalanceNotSyncedError,
    RetrieveAccountBalanceNotSyncedDayError(NaiveDate),
    RetrieveAccountPositionNotSyncedDayError(String, NaiveDate),
}

impl error::Error for DBRetrieveError {}

impl fmt::Display for DBRetrieveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RetrieveAccountNoAccountError(s) => write!(
                f,
                "Error looking up account with: {}! No such account found by number or name.",
                s
            ),
            Self::RetrieveAccountPositionNoAccountError => write!(
                f,
                "Could not find position! No account matching identifier in DB."
            ),
            Self::RetrieveAccountPositionAllNotSyncedError => write!(
                f,
                "Could not find position! No positions at all synced on account."
            ),
            Self::RetrieveAccountPositionNotSyncedError(s) => {
                write!(f, "Could not find position: {}! Not synced for account.", s)
            }
            Self::RetrieveAccountPositionNotSyncedDayError(s, date) => write!(
                f,
                "Could not find position! No {} positions synced for account date: {}.",
                s, date
            ),
            Self::RetrieveAccountBalanceNoAccountError => write!(
                f,
                "Could not find balance! No account matching identifier in DB."
            ),
            Self::RetrieveAccountBalanceNotSyncedError => write!(
                f,
                "Could not find balance! No balances at all synced for account."
            ),
            Self::RetrieveAccountBalanceNotSyncedDayError(date) => write!(
                f,
                "Could not find balance! No balances synced for account date: {}.",
                date
            ),
        }
    }
}
