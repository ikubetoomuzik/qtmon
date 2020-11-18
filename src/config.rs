//! Config loading and saving module, gonna use Rust Object Notation (RON).
//! By: Curtis Jones <mail@curtisjones.ca>
//! Started on: November 8, 2020

use super::include::{
    clap_app, config_dir, default_format, from_str, io, read_to_string, to_string, Account,
    AccountNumber, AccountStatus, AccountType, AdaptiveFormat, AuthenticationInfo, Cleanup,
    ClientAccountType, ColoredHelp, Criterion, Currency, DateTime, Deserialize, Duplicate,
    Duration, Instant, LevelFilter, Local, LogSpecBuilder, Logger, Naming, OpenOptions, PathBuf,
    ReconfigurationHandle, Result, Serialize, Write,
};

static DEFAULT_CONFIG: &str = "ConfigFile(\n\
    \t// Where you want to store the auth info. If it's relative it will be measured relative to \n\
    \t// the directory of the config file.\n\
    \tauth_file_path: \"auth.ron\",\n\
    \t// Where you want to store the database. If it's relative it will be measured relative to \n\
    \t// the directory of the config file.\n\
    \tdb_file_path: \"db.ron\",\n\
    \t// The directory to store logs in, one active and up to 5 zipped archives. If it's relative\n\
    \t// it will be measured relative to the directory of the config file.\n\
    \tlog_file_dir: \"logs\",\n\
    \t// File & Stdout log level use the same enum to make a choice. The levels of log detail are:\n\
    \t// None => Print no logs at all. \n\
    \t// Error => Print only error logs. The most severe events.\n\
    \t// Warn => Print Error & Warn level logs. The most severe events and slight events that the \n\
    \t// program can easily handle. \n\
    \t// Info => Print all logs that this program has. Notifications for the start and completion \n\
    \t// events as well as all previously described error logs. \n\
    \tfile_log_level: Info,\n\
    \tstdout_log_level: Info,\n\
    \t// Local port for the REST Api server to listen on.\n\
    \thttp_port: 49494,\n\
    \t// List of AccountToSync objects that define which accounts you want to sync.\n\
    \t// They are of the form AccountToSync($StringRepresentingName, $ListofAccountSelectorObjects).\n\
    \t// And AccountSelector objects are an enum that can be any of:\n\
    \t// Number(String) => Where the string is your account number.\n\
    \t// IsPrimary(bool) => pretty simple, true or false whether the account you want is primary.\n\
    \t// IsBilling(bool) => read above but substitute billing for primary.\n\
    \t// AccountType(AccountType) => Where AccountType is one of: Cash, Margin, TFSA, RRSP, SRRSP,\n\
    \t// LRRSP, LIRA, LIF, RIF, SRIF, LRIF, RRIF, PRIF, RESP, or FRESP.\n\
    \t// ClientAccountType(ClientAccountType) => Where ClientAccountType is one of: Individual, Joint,\n\
    \t// InformalTrust, Corporation, InvestmentClub, FormalTrust, Partnership, SoleProprietorship,\n\
    \t// Family, JointAndInformalTrust, or Institution.\n\
    \t// Status(AccountStatus) => Where AccountStatus is one of: Active, SuspendedClosed, \n\
    \t// SuspendedViewOnly, Liquidate, or Closed.\n\
    \taccounts_to_sync: [AccountToSync(\"Primary\", [IsPrimary(true)])],\n\
    \t// Where the currency can be CAD or USD.\n\
    \taccount_balance_currency: CAD,\n\
    \t// The delay in seconds you want between Api syncs.\n\
    \tdelay: 300,\n\
)";

fn validate_pathbuf(to_validate: PathBuf, config_path_arg: &PathBuf) -> PathBuf {
    if to_validate.is_relative() && *config_path_arg == PathBuf::new() {
        let mut result = config_dir().unwrap_or_default();
        result.push("qtmon/");
        result.push(to_validate);
        result
    } else if to_validate.is_relative() && *config_path_arg != PathBuf::new() {
        let mut result = match config_path_arg.parent() {
            Some(cd) => (*cd).to_path_buf(),
            None => PathBuf::new(),
        };
        result.push(to_validate);
        result
    } else {
        to_validate
    }
}

/// Struct defining the Configuration that will be used by all other modules.
pub struct Config {
    pub settings: ConfigFile,
    pub auth: AuthInfo,
    _logger: ReconfigurationHandle,
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
        // check for X_DEFAULT_DIR.
        let default_config_path = match config_dir() {
            Some(mut cd) => {
                let def = "qtmon/config.ron".to_string();
                cd.push(def);
                cd
            }
            None => PathBuf::new(),
        };
        // check for a supplied config file path.
        let config_path_arg = match args.value_of("CONFIG") {
            Some(cfg) => {
                let mut res = PathBuf::new();
                res.push(cfg);
                res
            }
            None => PathBuf::new(),
        };
        // open the config file, if the user provided a specific file then we error out if its not
        // there. but if using the default file location and file not found then we write out the
        // default config file and use it.
        let mut settings = match ConfigFile::load(if config_path_arg == PathBuf::new() {
            &default_config_path
        } else {
            &config_path_arg
        }) {
            Ok(settings) => settings,
            Err(e) => {
                let e = e.downcast::<io::Error>()?;
                match e.as_ref().kind() {
                    io::ErrorKind::NotFound => {
                        if config_path_arg == PathBuf::new() {
                            // open and write out a default config at the default location
                            OpenOptions::new()
                                .write(true)
                                .create(true)
                                .open(default_config_path)?
                                .write_all(DEFAULT_CONFIG.as_bytes())?;
                            from_str::<ConfigFile>(&DEFAULT_CONFIG).unwrap()
                        } else {
                            return Err(e);
                        }
                    }
                    _ => return Err(e),
                }
            }
        };
        // start with setting up our logger. make a builder first.
        // check the file path and allow for relative paths if using default config dir
        settings.log_file_dir = validate_pathbuf(settings.log_file_dir, &config_path_arg);
        let (file_log_level, _) = settings.file_log_level.to_usable();
        let (_, stdout_log_level) = settings.stdout_log_level.to_usable();
        let mut builder = LogSpecBuilder::new();
        builder.default(file_log_level);
        let _logger = Logger::with(builder.build())
            .log_to_file()
            .suppress_timestamp()
            .append()
            .directory(&settings.log_file_dir)
            .suffix("log")
            .duplicate_to_stdout(stdout_log_level)
            .format_for_files(default_format)
            .adaptive_format_for_stdout(AdaptiveFormat::Default)
            .rotate(
                Criterion::Size(51200),
                Naming::Numbers,
                Cleanup::KeepCompressedFiles(5),
            )
            .start()?;
        // now that the logger is up and running.
        // check the file path and allow for relative paths if using default config dir
        settings.auth_file_path = validate_pathbuf(settings.auth_file_path, &config_path_arg);
        // load the saved auth info or use the supplied token if it is there.
        let auth = match args.value_of("REFRESH") {
            Some(rt) => AuthInfo::RefreshToken(rt.to_string()),
            None => AuthInfo::load(&settings.auth_file_path)?,
        };

        // validate the db file path and add the default path if using it
        settings.db_file_path = validate_pathbuf(settings.db_file_path, &config_path_arg);

        // return our generated config.
        Ok(Config {
            settings,
            auth,
            _logger,
        })
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
    pub db_file_path: PathBuf,
    pub auth_file_path: PathBuf,
    pub log_file_dir: PathBuf,
    pub file_log_level: LogLevel,
    pub stdout_log_level: LogLevel,
    pub http_port: u16,
    pub accounts_to_sync: Vec<AccountToSync>,
    pub account_balance_currency: Currency,
    pub delay: u64,
}

impl ConfigFile {
    fn load(file: &PathBuf) -> Result<Self> {
        let input = read_to_string(file)?;
        Ok(from_str::<Self>(&input)?)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum LogLevel {
    None,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    pub fn to_usable(&self) -> (LevelFilter, Duplicate) {
        match self {
            Self::None => (LevelFilter::Off, Duplicate::None),
            Self::Info => (LevelFilter::Info, Duplicate::Info),
            Self::Warn => (LevelFilter::Warn, Duplicate::Warn),
            Self::Error => (LevelFilter::Error, Duplicate::Error),
        }
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
    fn load(file: &PathBuf) -> Result<Self> {
        let input = read_to_string(file)?;
        Ok(from_str::<Self>(&input)?)
    }
    fn save(&self, file: &PathBuf) -> Result<()> {
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
