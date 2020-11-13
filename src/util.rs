//! Simple utility module to import and then re-export libs to use in project.
//! By: Curtis Jones <mail@curtisjones.ca>
//! Started on: November 12, 2020

// Use statements.
pub use chrono::{DateTime, Duration, NaiveDate, NaiveTime, Utc};
pub use clap::{clap_app, AppSettings::ColoredHelp};
pub use dirs::config_dir;
pub use questrade::{
    Account, AccountBalance, AccountPosition, AccountType, AuthenticationInfo, ClientAccountType,
    Currency, Questrade,
};
pub use reqwest::Client;
pub use ron::{from_str, to_string};
#[cfg(feature = "bincode")]
pub use rustbreak::deser::Bincode;
#[cfg(feature = "default")]
pub use rustbreak::deser::Ron;
#[cfg(feature = "yaml")]
pub use rustbreak::deser::Yaml;
pub use rustbreak::PathDatabase;
pub use serde::{Deserialize, Serialize};
pub use std::{
    cell::RefCell,
    collections::HashMap,
    fs::{read_to_string, OpenOptions},
    io::{self, Read, Write},
    path::PathBuf,
    sync::{mpsc, Arc, Mutex, Weak},
    time::Instant,
};
pub use tokio;
pub use warp;

// Typedefs.
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
pub type AccountName = String;
pub type AccountNumber = String;
