#![allow(dead_code, unused_variables, unused_imports)]
//! Multithreaded application to monitor a questrade account.
//! By: Curtis Jones <mail@curtisjones.ca>
//! Started on: November 8, 2020

// Global general typedef.
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

mod config;
mod http_server;
mod monitor;
mod storage;

#[tokio::main]
async fn main() -> Result<()> {
    let conf = config::Config::generate()?;
    let mut interface = monitor::Monitor::new(conf).await?;
    interface.validate_auth().await?;
    println!(
        "Work in progress..\n\nTest Output:\nConfig:\n{:#?}\nDB:\n{:#?}",
        interface.config,
        interface.db,
    );
    Ok(())
}
