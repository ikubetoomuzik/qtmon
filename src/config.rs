//! Config loading and saving module, gonna use Rust Object Notation (RON).
//! By: Curtis Jones <mail@curtisjones.ca>
//! Started on: November 8, 2020

use super::include::{
    clap_app, config_dir, from_str, read_to_string, to_string, Account, AccountNumber,
    AccountStatus, AccountType, AuthenticationInfo, ClientAccountType, ColoredHelp, Currency,
    DateTime, Deserialize, Duration, Instant, Local, OpenOptions, Result, Serialize, Write,
};

#[derive(Debug)]
/// Struct defining the Configuration that will be used by all other modules.
pub struct Config {
    pub settings: ConfigFile,
    pub auth: AuthInfo,
}

impl Config {
    /// Basic starting function to generate the main config.
    /// Parses command line args and then uses those to load the ConfigFile.
    pub fn generate() -> Result<Self> {
        // get cli args.
        let args = clap_app!(qtmon =>
        (version: env!("CARGO_PKG_VERSION"))
        (author: "Curtis Jones <mail@curtisjones.ca>")
        (about: "program to monitor questrade account using official api")
        (@arg CONFIG:  -c --config        +takes_value "Sets a custom config file.")
        (@arg REFRESH: -r --refreshtoken  +takes_value "If running for the first time this can provide the starting refresh token."))
        .setting(ColoredHelp)
        .get_matches();
        // check for a supplied refresh token.
        let refresh_token_arg = args.value_of("REFRESH");
        // check for a supplied config file path.
        let config_path_arg = args.value_of("CONFIG");
        // check for X_DEFAULT_DIR.
        let default_config_dir = match config_dir() {
            Some(mut cd) => {
                let def = "qtmon/config.ron".to_string();
                cd.push(def);
                cd.to_str().unwrap().to_string()
            }
            None => String::new(),
        };
        // load the settings (ConfigFile struct).
        let settings = ConfigFile::load(match config_path_arg {
            Some(path) => path.trim(),
            None => &default_config_dir,
        })?;
        // load the saved auth info or use the supplied token if it is there.
        let auth = match refresh_token_arg {
            Some(rt) => AuthInfo::RefreshToken(rt.to_string()),
            None => AuthInfo::load(&settings.auth_file_path)?,
        };

        // return our generated config.
        Ok(Config { settings, auth })
    }
    // Save a new set of authentication info.
    pub fn save_new_auth_info(&mut self, auth_info: AuthenticationInfo) -> Result<()> {
        // Convert from api version to the version I can save.
        let auth = AuthInfo::convert_from_api_auth(auth_info)?;
        // Set the auth to our variable.
        self.auth = auth;
        // Use the AuthInfo save() method to store the value.
        self.auth.save(&self.settings.auth_file_path)?;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigFile {
    pub db_file_path: String,
    auth_file_path: String,
    pub http_port: u16,
    pub accounts_to_sync: Vec<AccountToSync>,
    pub account_balance_currency: Currency,
}

impl ConfigFile {
    fn load(file: &str) -> Result<Self> {
        let input = read_to_string(file)?;
        Ok(from_str::<Self>(&input)?)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum AccountSelector {
    Number(AccountNumber),
    IsPrimary(bool),
    IsBilling(bool),
    AccountType(AccountType),
    ClientAccountType(ClientAccountType),
    Status(AccountStatus),
}

impl AccountSelector {
    fn check_account_match(&self, acct: &Account) -> bool {
        match self {
            Self::Number(acc_num) => *acc_num == acct.number,
            Self::IsPrimary(ip) => *ip == acct.is_primary,
            Self::IsBilling(ib) => *ib == acct.is_billing,
            Self::AccountType(acc_type) => *acc_type == acct.account_type,
            Self::ClientAccountType(cli_acc_type) => *cli_acc_type == acct.client_account_type,
            Self::Status(acc_stat) => *acc_stat == acct.status,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountToSync(String, Vec<AccountSelector>);

impl AccountToSync {
    pub fn check_account_match(&self, acct: &Account) -> bool {
        self.1
            .iter()
            .all(|selector| selector.check_account_match(acct))
    }
    pub fn name(&self) -> String {
        self.0.clone()
    }
    pub fn name_is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SavedAuthInfo {
    refresh_token: String,
    access_token: String,
    expires_at: DateTime<Local>,
    api_server: String,
    is_demo: bool,
}

impl SavedAuthInfo {
    pub fn convert_to_api(&self) -> AuthenticationInfo {
        // Chrono durations can be negative so we can just do the subtraction. If its positive then
        // the token isn't expired and we are doing addition. And visa versa.
        let time_to_expiry = self.expires_at - Local::now();
        // Here is where we make the actual decision and calculation.
        let expires_at = if time_to_expiry >= Duration::zero() {
            // Since we have already checked the sign of the duration we know we can just convert
            // and call unwrap with no risk of panic.
            let time_to_expiry = time_to_expiry.to_std().unwrap();
            // Sign was positive so we add.
            Instant::now() + time_to_expiry
        } else {
            let time_to_expiry = time_to_expiry * -1;
            let time_to_expiry = time_to_expiry.to_std().unwrap();
            Instant::now() - time_to_expiry
        };

        AuthenticationInfo {
            refresh_token: self.refresh_token.to_string(),
            access_token: self.access_token.to_string(),
            api_server: self.api_server.to_string(),
            is_demo: false,
            expires_at,
        }
    }
    fn convert_from_api(api_auth: AuthenticationInfo) -> Result<Self> {
        // pull the expires at time from the arg
        let expires_at = api_auth.expires_at;
        // get the time
        let now = Instant::now();
        // have to do the check here because std:instants do not like negative numbers.
        let duration = if now > expires_at {
            now - expires_at
        } else {
            expires_at - now
        };
        // convert std:duration to chrono:duration
        let duration = Duration::from_std(duration)?;
        // apply the duration and we have to do the sign ourselves because of std:time not liking
        // negative numbers.
        let expires_at = if now > expires_at {
            Local::now() - duration
        } else {
            Local::now() + duration
        };
        // and finally we return the calculated val.
        Ok(SavedAuthInfo {
            refresh_token: api_auth.refresh_token,
            access_token: api_auth.access_token,
            expires_at,
            api_server: api_auth.api_server,
            is_demo: api_auth.is_demo,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum AuthInfo {
    RefreshToken(String),
    FullAuthInfo(SavedAuthInfo),
}

impl AuthInfo {
    // *** Basic I/O Functions ***
    fn load(file: &str) -> Result<Self> {
        let input = read_to_string(file)?;
        Ok(from_str::<Self>(&input)?)
    }
    fn save(&self, file: &str) -> Result<()> {
        let mut file = OpenOptions::new().write(true).create(true).open(file)?;
        let output = to_string::<Self>(self)?;
        file.write_all(output.as_bytes())?;
        Ok(())
    }
    // *** Conversion Functions ***
    fn convert_from_api_auth(api_auth: AuthenticationInfo) -> Result<Self> {
        Ok(Self::FullAuthInfo(SavedAuthInfo::convert_from_api(
            api_auth,
        )?))
    }
    // *** Helper Functions ***
    pub fn refresh_token(&self) -> &str {
        match self {
            Self::RefreshToken(rt) => rt,
            Self::FullAuthInfo(sai) => &sai.refresh_token,
        }
    }
    pub fn is_expired(&self) -> bool {
        match self {
            Self::RefreshToken(_) => true,
            Self::FullAuthInfo(sai) => {
                if sai.expires_at - Local::now() <= Duration::minutes(5) {
                    true
                } else {
                    false
                }
            }
        }
    }
}
