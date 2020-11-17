// #![allow(dead_code, unused_variables, unused_imports)]
//! Multithreaded application to monitor a questrade account.
//! By: Curtis Jones <mail@curtisjones.ca>
//! Started on: November 8, 2020

// Local modules to store the real workhorse code.
mod config;
mod http_server;
mod include;
mod monitor;
mod myerrors;
mod storage;

// Local use statements.
use config::Config;
use include::{tokio, Result};
use monitor::Monitor;

#[tokio::main]
// And now we do the main function, wrapped with tokio so it can be async.
async fn main() -> Result<()> {
    // Reads CLI args and a config file encoded in Ron to generate config.
    let conf = Config::generate()?;
    // This creates a new interface to use for the app,
    // it also makes sure that all auth info is valid.
    let mut mon = Monitor::new(conf).await?;
    // sync the accounts and for now we print errors.
    if let Err(e) = mon.execute_runtime().await {
        eprintln!("Error1:\n{}", e);
    }
    Ok(())
}
