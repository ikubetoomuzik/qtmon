//! Questrade monitor stuct to manage authentication and requests.
//! By: Curtis Jones <mail@curtisjones.ca>
//! Started on: November 8, 2020

use super::config::{AuthInfo, Config};
use questrade::{AuthenticationInfo, Questrade};
use reqwest::Client;

pub struct QtradeAPIInterface {
    pub config: Config,
    qtrade: Questrade,
}

impl QtradeAPIInterface {
    pub async fn new(mut config: Config) -> Result<Self, Box<dyn std::error::Error>> {
        // Set up our qtrade variable to be set in the match statement.
        let qtrade: Questrade;
        match &config.auth {
            // If its just a token we use it to authenticate and then save our new info.
            AuthInfo::RefreshToken(rt) => {
                qtrade = Questrade::new();
                qtrade.authenticate(rt, false).await?;
                config.save_new_auth_info(qtrade.get_auth_info().unwrap())?;
            }
            // If we have full auth info then we just set that.
            AuthInfo::FullAuthInfo(sai) => {
                let auth_info = sai.convert_to_api();
                qtrade = Questrade::with_authentication(auth_info, Client::new());
            }
        }
        Ok(QtradeAPIInterface {
            config,
            qtrade,
        })
    }
}
