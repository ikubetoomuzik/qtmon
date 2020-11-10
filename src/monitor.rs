//! Questrade monitor stuct to manage authentication and requests.
//! By: Curtis Jones <mail@curtisjones.ca>
//! Started on: November 8, 2020

use super::{Result, storage::DB,config::{AuthInfo, Config}};
use questrade::Questrade;
use reqwest::Client;

pub struct Monitor {
    pub config: Config,
    pub db: DB,
    qtrade: Questrade,
}


impl Monitor {
    pub async fn new(mut config: Config) -> Result<Self> {
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
        // Set up the Rustbreak DB.
        let db = DB::new(&config)?;
        Ok(Self {
            config,
            db,
            qtrade,
        })
    }
    pub async fn validate_auth(&mut self) -> Result<()> {
        if self.config.auth.is_expired() {
            self.refresh_auth().await?;
        } 
        Ok(())
    }
    async fn refresh_auth(&mut self) -> Result<()> {
        self.qtrade.authenticate(self.config.auth.refresh_token(), false).await?;
        self.config.save_new_auth_info(self.qtrade.get_auth_info().unwrap())?;
        Ok(())
    }
}
