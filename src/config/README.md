Configuration Options Overview

Since RON is our default encoding Config examples will be written in rust:
```rust
struct ConfigFile {
    // Location of the file the DB will use for long-term storage.
    // If location is relative then it will be used relative to the 
    // parent dir of the config file.
    db_file_path: PathBuf,
    // Location of the file the program will use for auth storage.
    // If location is relative then it will be used relative to the 
    // parent dir of the config file.
    auth_file_path: PathBuf,
    // Location of the directory the program will use for log storage.
    // If location is relative then it will be used relative to the 
    // parent dir of the config file.
    log_file_dir: PathBuf,
    // Minimum log level you want output to file or stdout.
    // In order of most to least detail it would be:
    // None, Error, Warn, Info
    file_log_level: LogLevel,
    stdout_log_level: LogLevel,
    // Port to listen on for REST API.
    http_port: u16,
    // List of AccountToSync objects that will be used to determine what
    // information will be requested and saved with API.
    accounts_to_sync: Vec<AccountToSync>,
    // Currency to use when retrieving Cash and Combined balances.
    account_balance_currency: Currency,
    // Delay in seconds between sync attempts.
    delay: u64,
}

// Struct defining specific accounts to sync, with a Name(String) and a
// list of selector objects (Vec<AccountSelector>). See below for more
// info on the AccountSelector enum.
struct AccountToSync(String, Vec<AccountSelector>);

// Enum to describe the different possible ways to select an account.
enum AccountSelector {
    // compare the number contained to see if it matches.
    Number(String),
    // compare the boolean contained to see if it matches.
    IsPrimary(bool),
    // compare the boolean contained to see if it matches.
    IsBilling(bool),
    // The account type is something like TFSA, RRSP, RRIP, etc.
    AccountType(AccountType),
    // The account type is something like Individual, Joint, etc.
    ClientAccountType(ClientAccountType),
    // Basically boils down to Active or Closed, maybe Suspended if
    // you got something weird going on with yourself.
    Status(AccountStatus),
}
```

## Author

By: **Curtis Jones** <*mail@curtisjones.ca*>
