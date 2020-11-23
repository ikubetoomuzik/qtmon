# qtmon
Version: **0.1.0**


## Quick Ref
* [Statusbar](#Statusbar)
* [Statusbar/Variables](#Variables)
* [Raw](#Raw)
* [Raw/Account](#Account)
* [Raw/Balance](#Balance)
* [Raw/Position](#Position)


## Statusbar

Only one path is available with this part of the api.


__/statusbar/__*$accountIdentifier*__/__*$inputString*


Where the api will respond with a string equal to the *$inputString* with variables substituted.

### Variables

| Identifier                                      | Description                                                          |
|-------------------------------------------------|----------------------------------------------------------------------|
| **%slash**                                      | Just replaces with the *"/"* character.                              |
| **%dollar**                                     | Just replaces with the *"$"* character.                              |
| **underscore**                                  | Just replaces with the *"space"* character.                          |
| **$accountIdentifier**                          | Account name or number.                                              |
| **$inputString**                                | String to replace variables in and return..                          |
| **%sod.cash**                                   | Cash balance for the account @ start of day.                         |
| **%sod.marketValue**                            | Market value for the account @ start of day.                         |
| **%sod.totalEquity**                            | Cash + bal @ start of day.                                           |
| **%sod.maitenanceExcess**                       | Usually equal to cash but sometimes different.                       |
| **%bal.cash**                                   | Most recent known cash bal for acct.                                 |
| **%bal.cashPNL**                                | Change in cash from sod as a % of sod                                |
| **%bal.marketValue**                            | Most recent known market value for acct.                             |
| **%bal.marketValuePNL**                         | Change in market value from sod as a % of sod.                       |
| **%bal.totalEquity**                            | Most recent cash + bal.                                              |
| **%bal.totalEquityPNL**                         | Change in total equity from sod as a % of sod.                       |
| **%bal.maitenanceExcess**                       | Most recent value, usually same as cash bal.                         |
| **%bal.maitenanceExcessPNL**                    | Change in maitenance excess from sod as a % of sod.                  |
| **%***[Position Symbol]***.openQuantity**       | Total quantity of position currently owned.                          |
| **%***[Position Symbol]***.closedQuantity**     | Total quantity of position sold.                                     |
| **%***[Position Symbol]***.currentMarketValue** | Market value at most recent sync.                                    |
| **%***[Position Symbol]***.sodMarketValue**     | Market value @ sod.                                                  |
| **%***[Position Symbol]***.currentPrice**       | Current price of the position.                                       |
| **%***[Position Symbol]***.averageEntryPrice**  | Average price you purchased the position at.                         |
| **%***[Position Symbol]***.totalCost**          | Dollar value of cash spent purchasing this position.                 |
| **%***[Position Symbol]***.openPNL**            | Open (not sold) PNL from start of invest, as a % of totalCost.       |
| **%***[Position Symbol]***.closedPNL**          | Closed (or sold) PNL from start of invest, as a % of totalCost.      |
| **%***[Position Symbol]***.dayPNL**             | PNL from sod as a % of sod.                                          |
| **%***[Position Symbol]***.{any}PNLABS**        | All of the PNL position apis have this option, returns absolute val. |

## Raw 


Returns JSON representing the requested info.


| Variable        | Description                                |
|-----------------|--------------------------------------------|
| **$position**   | The symbol for a position held on account. |
| **$identifier** | An account *name* or *number*.             |
| **$date**       | A date of the format: *YYYY-MM-DD*.        |
| **$time**       | A time of the format: *HH:MM*.             |

### Account

| Path                        | Description                  |
|-----------------------------|------------------------------|
| `/raw/account/list`         | Json array of account names. |
| `/raw/account/$indentifier` | Json object of account info. |

### Balance

| Path                                    | Description                                                    |
|-----------------------------------------|----------------------------------------------------------------|
| `/raw/balance/$identifier/sod`          | The balance at the start of day today.                         |
| `/raw/balance/$identifier/latest`       | The most recently synced balance.                              |
| `/raw/balance/$identifier/$date/sod`    | The balance at the start of day for **$date**.                 |
| `/raw/balance/$identifier/$date/latest` | The most recently synced balance for **$date**.                |
| `/raw/balance/$identifier/$date/$time`  | The balance closest to **$date** & **$time**.                  |

### Position

| Path                                               | Description                                                              |
|----------------------------------------------------|--------------------------------------------------------------------------|
| `/raw/position/$identifier/list`                   | List of position symbols (**$position**).                                |
| `/raw/position/$identifier/$position/latest`       | The latest synced info for the **$position**.                            |
| `/raw/position/$identifier/$position/$date/latest` | The latest synced info for the **$position** on **$date**.               |
| `/raw/position/$identifier/$position/$date/$time`  | The closest synced info for the **$position** on **$date** at **$time**. |

## Author

By: **Curtis Jones** <*mail@curtisjones.ca*>
