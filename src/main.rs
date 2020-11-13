//! Multithreaded application to monitor a questrade account.
//! By: Curtis Jones <mail@curtisjones.ca>
//! Started on: November 8, 2020
// #![allow(dead_code, unused_variables, unused_imports)]

// Local modules to store the real workhorse code.
mod config;
mod http_server;
mod monitor;
mod storage;
mod util;

// Local use statements.
use config::Config;
use monitor::Monitor;
use util::{io, tokio, Result};

// temp delay_until_input function
fn delay_until_input() -> Result<()> {
    for _ in 0..20 {
        let mut buffer = String::new();
        let stdin = io::stdin();
        stdin.read_line(&mut buffer)?;
    }
    Ok(())
}

#[tokio::main]
// And now we do the main function, wrapped with tokio so it can be async.
async fn main() -> Result<()> {
    // Reads CLI args and a config file encoded in Ron to generate config.
    let conf = Config::generate()?;
    // This creates a new interface to use for the app, it also makes sure that all auth info is
    // valid.
    let interface = Monitor::new(conf).await?;
    delay_until_input()?;
    Ok(())
}
