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
use include::{io, tokio, Result};
use monitor::Monitor;

// temp delay_until_input function
fn delay_until_input() -> Result<()> {
    let mut buffer = String::new();
    let stdin = io::stdin();
    stdin.read_line(&mut buffer)?;
    Ok(())
}

#[tokio::main]
// And now we do the main function, wrapped with tokio so it can be async.
async fn main() -> Result<()> {
    // Reads CLI args and a config file encoded in Ron to generate config.
    let conf = Config::generate()?;
    // This creates a new interface to use for the app,
    // it also makes sure that all auth info is valid.
    let mut mon = Monitor::new(conf).await?;
    // sync the accounts and for now we print errors.
    if let Err(e) = mon.sync_accounts().await {
        eprintln!("Error1:\n{}", e);
    }
    // sync the balances and for now we print errors.
    if let Err(e) = mon.sync_account_balances().await {
        eprintln!("Error2:\n{}", e);
    }
    // sync the positions and for now we print errors.
    if let Err(e) = mon.sync_account_positions().await {
        eprintln!("Error3:\n{}", e);
    }
    delay_until_input()?;
    mon.save_db()?;
    Ok(())
}
