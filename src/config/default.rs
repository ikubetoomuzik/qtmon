//! Simple submodule to just hold the &str of the default config file.
//! By: Curtis Jones <mail@curtisjones.ca>
//! Started on: November 18, 2020

pub static DEFAULT_CONFIG: &str = "ConfigFile(\n\
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
