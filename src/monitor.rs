//! Questrade monitor stuct to manage authentication and requests.
//! By: Curtis Jones <mail@curtisjones.ca>
//! Started on: November 8, 2020

use super::{
    config::{AuthInfo, Config},
    http_server::HTTPServer,
    storage::{DBRef, DB},
    util::{Arc, Client, Questrade, Result},
};

pub struct Monitor {
    pub config: Config,
    pub db: DBRef,
    qtrade: Questrade,
    pub http: HTTPServer,
}

// impl Drop for Monitor {
//     fn drop(&mut self) {
//         println!("Shutting down...");
//         self.http.kill();
//     }
// }

impl Monitor {
    pub async fn new(mut config: Config) -> Result<Self> {
        // Set up our qtrade variable to be set in the match statement.
        let qtrade: Questrade = match &config.auth {
            // If its just a token we use it to authenticate and then save our new info.
            AuthInfo::RefreshToken(rt) => {
                let qtrade = Questrade::new();
                qtrade.authenticate(rt, false).await?;
                config.save_new_auth_info(qtrade.get_auth_info().unwrap())?;
                qtrade
            }
            // If we have full auth info then we just set that.
            AuthInfo::FullAuthInfo(sai) => {
                let auth_info = sai.convert_to_api();
                let client = Client::new();
                let qtrade = Questrade::with_authentication(auth_info, client);
                qtrade
            }
        };
        // Set up the Rustbreak DB. Depending on enabled features it can have a Ron, Yaml, or
        // Bincode backend for data storage. No matter what it is a Path database.
        let db = DBRef::new(DB::new(&config)?);
        // Start the http server.
        let http = HTTPServer::new(Arc::downgrade(&db.clone()), Vec::new());
        // Return the created Monitor.
        let mut result = Self {
            config,
            db,
            qtrade,
            http,
        };
        // make sure we have valid tokens when we create it.
        result.validate_auth().await?;
        // Returned the created interface.
        Ok(result)
    }

    pub async fn validate_auth(&mut self) -> Result<()> {
        // The only time we need to take action is if things are expired.
        if self.config.auth.is_expired() {
            // Here we make the request to the questrade server to get new auth info.
            self.qtrade
                .authenticate(self.config.auth.refresh_token(), false)
                .await?;
            // Here we save it to the config object and the local auth file.
            self.config
                .save_new_auth_info(self.qtrade.get_auth_info().unwrap())?;
        }
        // Indicate that everything is ok
        Ok(())
    }
}
