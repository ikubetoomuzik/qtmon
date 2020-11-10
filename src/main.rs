// #![allow(dead_code, unused_variables, unused_imports)]
//! Multithreaded application to monitor a questrade account.
//! By: Curtis Jones <mail@curtisjones.ca>
//! Started on: November 8, 2020

mod config;
mod http_server;
mod monitor;
mod storage;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conf = config::Config::generate()?;
    let interface = monitor::QtradeAPIInterface::new(conf).await?;
    println!(
        "Work in progress..\n\nTest Output:\n{:#?}",
        interface.config
    );
    Ok(())
}
