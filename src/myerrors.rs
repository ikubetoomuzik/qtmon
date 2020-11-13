//! Module containing the specific errors to be defined for my project.
//! By: Curtis Jones <mail@curtisjones.ca>
//! Started on: November 12, 2020

use super::include::{error, fmt};

#[derive(Debug)]
// Enum representing errors that are possible during inserts into my database.
pub enum DBInsertError {
    InsertAccountDuplicateNumberError,
    InsertAccountDuplicateInfoError,
    InsertAccountBalanceError,
    InsertAccountPositionError,
}

impl error::Error for DBInsertError {}

impl fmt::Display for DBInsertError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DBInsertError::InsertAccountDuplicateNumberError => write!(
                f,
                "Could not insert Account into DataBase! Duplicate account number already in DB."
            ),
            DBInsertError::InsertAccountDuplicateInfoError => write!(
                f,
                "Could not insert Account into DataBase! Duplicate account info already in DB."
            ),
            DBInsertError::InsertAccountBalanceError => write!(
                f,
                "Could not insert Account Balance into DataBase! Duplicate already in DB."
            ),
            DBInsertError::InsertAccountPositionError => write!(
                f,
                "Could not insert Account Position into DataBase! Duplicate already in DB."
            ),
        }
    }
}
