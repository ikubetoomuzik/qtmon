//! Simple utility module to import and then re-export libs to use in project.
//! By: Curtis Jones <mail@curtisjones.ca>
//! Started on: November 12, 2020

// Use statements.
pub use chrono::{DateTime, Duration, Local, NaiveDate, NaiveTime};
pub use clap::{clap_app, AppSettings::ColoredHelp};
pub use dirs::config_dir;
pub use flexi_logger::{
    colored_default_format, default_format, AdaptiveFormat, Cleanup, Criterion, Duplicate,
    LevelFilter, LogSpecBuilder, Logger, Naming, ReconfigurationHandle,
};
pub use futures::{future::FutureExt, pin_mut, select, try_join};
pub use log::{error, info, warn};
pub use questrade_rs::{
    Account, AccountBalance, AccountPosition, AccountStatus, AccountType, ApiError,
    AuthenticationInfo, ClientAccountType, Currency, Questrade,
};
pub use reqwest::Client;
pub use ron::{from_str, to_string};
pub use rustbreak::PathDatabase;
pub use serde::{Deserialize, Serialize};
pub use std::{
    cell::RefCell,
    collections::{hash_map, HashMap},
    error, fmt,
    fs::{read_to_string, DirBuilder, OpenOptions},
    io::{self, Read, Write},
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    path::{Path, PathBuf},
    sync::{mpsc, Arc, Mutex},
    time::Instant,
};
pub use tokio;
pub use warp::{
    self,
    http::StatusCode,
    reply::{json, with_status, Json},
};

// Feature specfic use statements.
#[cfg(feature = "bincode")]
pub use rustbreak::deser::Bincode;
#[cfg(feature = "default")]
pub use rustbreak::deser::Ron;
#[cfg(feature = "yaml")]
pub use rustbreak::deser::Yaml;

// Typedefs.
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
pub type AccountName = String;
pub type AccountNumber = String;
pub type PositionSymbol = String;
