//! Questrade monitor stuct to manage authentication and requests.
//! By: Curtis Jones <mail@curtisjones.ca>
//! Started on: November 8, 2020

use super::{
    config::{AuthInfo, Config},
    http_server::HTTPServer,
    include::{AccountNumber, ApiError, Client, Local, Questrade, Result},
    storage::{DBRef, DB},
};

pub struct Monitor {
    config: Config,
    db: DBRef,
    qtrade: Questrade,
    _http: HTTPServer,
}

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
        let _http = HTTPServer::new(config.settings.http_port, db.clone());
        // Return the created Monitor.
        let mut result = Self {
            config,
            db,
            qtrade,
            _http,
        };
        // make sure we have valid tokens when we create it.
        if result.config.auth.is_expired() {
            result.renew_auth().await?;
        }
        result
            .config
            .save_new_auth_info(result.qtrade.get_auth_info().unwrap())?;
        // Returned the created interface.
        Ok(result)
    }

    async fn renew_auth(&mut self) -> Result<()> {
        // Here we make the request to the questrade server to get new auth info.
        self.qtrade
            .authenticate(self.config.auth.refresh_token(), false)
            .await?;
        // Here we save it to the config object and the local auth file.
        self.config
            .save_new_auth_info(self.qtrade.get_auth_info().unwrap())?;
        Ok(())
    }

    pub async fn sync_accounts(&mut self) -> Result<()> {
        let mut qtrade_result = match self.qtrade.accounts().await {
            Ok(accs) => accs,
            Err(e) => {
                let e = e.downcast::<ApiError>()?;
                match e.as_ref() {
                    ApiError::NotAuthenticatedError(_) => {
                        self.renew_auth().await?;
                        self.qtrade.accounts().await?
                    }
                    _ => return Err(e),
                }
            }
        };
        for acc in qtrade_result.drain(..) {
            match self
                .config
                .settings
                .accounts_to_sync
                .iter()
                .find(|ats| ats.check_account_match(&acc))
            {
                Some(an) => (*self.db).db.write(|db_info| -> Result<()> {
                    let name = if an.name_is_empty() {
                        acc.number.clone()
                    } else {
                        an.name()
                    };
                    db_info.insert_account(name, acc)?;
                    Ok(())
                })?,
                None => continue,
            }?;
        }
        Ok(())
    }

    pub async fn sync_account_balances(&mut self) -> Result<()> {
        for acct_num in (*self.db)
            .db
            .read(|db_info| {
                db_info
                    .iter_accounts()
                    .map(|dbi| dbi.number.clone())
                    .collect::<Vec<AccountNumber>>()
            })?
            .drain(..)
        {
            let balances = match self.qtrade.account_balance(&acct_num).await {
                Ok(acct_bals) => acct_bals,
                Err(e) => {
                    let e = e.downcast::<ApiError>()?;
                    match *e {
                        ApiError::NotAuthenticatedError(_) => {
                            self.qtrade.account_balance(&acct_num).await?
                        }
                        _ => return Err(e),
                    }
                }
            };
            (*self.db).db.write(|db_info| -> Result<()> {
                db_info.insert_account_balance(
                    Local::now(),
                    &acct_num,
                    balances
                        .per_currency_balances
                        .iter()
                        .find(|bl| bl.currency == self.config.settings.account_balance_currency)
                        .unwrap_or(&balances.per_currency_balances[0])
                        .clone(),
                    balances
                        .sod_per_currency_balances
                        .iter()
                        .find(|bl| bl.currency == self.config.settings.account_balance_currency)
                        .unwrap_or(&balances.per_currency_balances[0])
                        .clone(),
                )?;
                Ok(())
            })??;
        }
        Ok(())
    }

    pub async fn sync_account_positions(&mut self) -> Result<()> {
        for acct_num in (*self.db)
            .db
            .read(|db_info| {
                db_info
                    .iter_accounts()
                    .map(|dbi| dbi.number.clone())
                    .collect::<Vec<AccountNumber>>()
            })?
            .drain(..)
        {
            let positions = match self.qtrade.account_positions(&acct_num).await {
                Ok(acct_poss) => acct_poss,
                Err(e) => {
                    let e = e.downcast::<ApiError>()?;
                    match *e {
                        ApiError::NotAuthenticatedError(_) => {
                            self.qtrade.account_positions(&acct_num).await?
                        }
                        _ => return Err(e),
                    }
                }
            };
            for pos in positions {
                (*self.db).db.write(|db_info| -> Result<()> {
                    db_info.insert_account_position(Local::now(), &acct_num, pos)?;
                    Ok(())
                })??;
            }
        }
        Ok(())
    }
    pub fn save_db(&self) -> Result<()> {
        (*self.db).db.save()?;
        Ok(())
    }

    pub fn print_db(&self) {
        println!("DB:\n{:#?}", self.db);
    }
}
