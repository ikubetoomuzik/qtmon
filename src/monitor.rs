//! Questrade monitor stuct to manage authentication and requests.
//! By: Curtis Jones <mail@curtisjones.ca>
//! Started on: November 8, 2020

use super::{
    config::{AuthInfo, Config},
    http_server::HTTPServer,
    include::{tokio, try_join, AccountNumber, ApiError, Client, Local, Questrade, Result},
    storage::{DBRef, DB},
};

pub struct Monitor {
    config: Config,
    db: DBRef,
    qtrade: Questrade,
    _http: HTTPServer,
}

impl Monitor {
    // *** public functions **
    /// Constructor function for the main struct of the project.
    /// If this function errors out then something is wrong.
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
    /// Main event loop of the program. Runs our different async functions
    /// with timeouts to make sure that we retry on the delay given by user.
    pub async fn execute_runtime(&mut self) -> Result<()> {
        loop {
            // announce beginning of the loop
            println!("Beginning exectution loop:");
            // calculate the next timeout based on the delay set by user
            let timeout = tokio::time::Instant::now()
                + tokio::time::Duration::from_secs(self.config.settings.delay);
            // announce start of account sync
            println!("Starting account sync...");
            // if the timeout triggers we get and Err so we announce that the timeout triggered,
            // if not the we get Ok. either way we just announce what happened and move on
            if let Err(_) = tokio::time::timeout_at(timeout, self.sync_accounts()).await {
                println!("Account sync was not completed within 5 minutes.");
            } else {
                println!("Account sync successful.");
            }
            // announce the start of next syncs.
            println!("Starting balance and position sync...");
            // run our balance and position syncs together so if there is a delay in either we use
            // that time to start the next request.
            match try_join!(
                tokio::time::timeout_at(timeout, self.sync_account_balances()),
                tokio::time::timeout_at(timeout, self.sync_account_positions())
            ) {
                Ok((Ok(_), Ok(_))) => println!("Balance and position sync successful."),
                Ok((Err(e), Ok(_))) => println!("Error during balance sync: {}.", e),
                Ok((Ok(_), Err(e))) => println!("Error during position sync: {}.", e),
                Ok((Err(e1), Err(e2))) => println!(
                    "Error during balance and position sync.\n\
                    Balance error: {}. Position error: {}.",
                    e1, e2
                ),
                Err(_) => println!("Balance and position sync timeout."),
            }
            // once we are done all of the syncing we save the info,
            // currently the only way to exit this loop is this function failing
            println!("Saving DB...");
            self.save_db()?;
            println!("DB save successful.");
            // if we still have time to wait we announce it
            if tokio::time::Instant::now() < timeout {
                println!("Waiting for next execution..");
            }
            // finally we delay here until there is something to do
            tokio::time::delay_until(timeout).await;
            print!("\n");
        }
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

    async fn sync_accounts(&mut self) -> Result<()> {
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

    async fn sync_account_balances(&self) -> Result<()> {
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
                Err(e) => return Err(e),
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

    async fn sync_account_positions(&self) -> Result<()> {
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
                Err(e) => return Err(e),
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

    fn save_db(&self) -> Result<()> {
        (*self.db).db.save()?;
        Ok(())
    }
}
