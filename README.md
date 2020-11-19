# qtmon
Version: **0.1.0**

## TODO

* [ ] Fill out README more completely.
* [ ] Add lemonbar, unibar, and raw/account apis.
* [ ] Go back and comment throughout the project, most specifically in the *storage* module.

## DONE
* [x] REST API to actually get info.
* [x] Add license. Probably just MIT.
* [x] Fix the way that *Config* loading parses non-root paths provided.
* [x] Add a string encoded default config to produce on first run if needed.
* [x] Split *storage* module into seperate modules, file is over 400 lines of code..
* [x] Review dependencies to see if there is anything I can strip off of the dependency chart.
* [x] Implement real event loop with error handling so all these Results I'm making are useful.

## Usage

The program uses [Rusty-Object-Notation](https://github.com/ron-rs/ron) or RON for it's default data storage.
You may optionally enable a Yaml or Bincode encoding by disabling default and
including either feature.

The default config location is $XDG_CONFIG_DIR/qtmon/config.ron or 
$HOME/.config/qtmon/config.ron and you can override it with the *-c* option.
By default the program will generate a default config file the first time it is run.
An overview of the different config options can be found [here](#Configuration Options Overview).
To connect the first time you will need to provide a **Questrade API Refresh Token**
as the *-r* arguement to the program.
Instructions on enabling the API for your account and generating a new token can 
be found [here](https://www.questrade.com/api/documentation/getting-started).


Once you've run the program once with your initial token it should be able to
manage authentication from there. All you need to do is query your localhost at the 
port you selected in your config to get the up-to-date details on your account.
An overview of the API and how to pull information can be found [here](#REST API Overview).
An overview of the API and how to pull information can be found [here](./src/http_server/README.md).



Since RON is our default encoding Config examples will be written in rust:

### Configuration Options Overview

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
### REST API Overview


## Author

By: **Curtis Jones** <*mail@curtisjones.ca*>
