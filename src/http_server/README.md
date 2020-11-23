# qtmon
Version: **0.1.0**


## Statusbar

Only one path is available with this part of the api.


**/statusbar/$***accountIdentifier***/$***inputString***


Where the api will respond with a string equal to the *inputString* with variables substituted.
To indicate you want spaces in the output a *underscore* char can be used. 
All *underscores* will be replaced with spaces.

### Variables available to use:

| Identifier                                      | Description                                                          |
|-------------------------------------------------|----------------------------------------------------------------------|
| **%slash**                                      | Just replaces with the *"/"* character.                              |
| **%dollar**                                     | Just replaces with the *"$"* character.                              |
| **%sod.cash**                                   | Cash balance for the account @ start of day.                         |
| **%sod.marketValue**                            | Market value for the account @ start of day.                         |
| **%sod.totalEquity**                            | Cash + bal @ start of day.                                           |
| **%sod.maitenanceExcess**                       | Usually equal to cash but sometimes different.                       |
| **%bal.cash**                                   | Most recent known cash bal for acct.                                 |
| **%bal.cashPNL**                                | Change in cash from sod as a % of sod                                |
| **%bal.marketValue**                            | Most recent known market value for acct.                             |
| **%bal.marketValuePNL**                         | Change in market value from sod as a % of sod                        |
| **%bal.totalEquity**                            | Most recent cash + bal                                               |
| **%bal.totalEquityPNL**                         | Change in total equity from sod as a % of sod                        |
| **%bal.maitenanceExcess**                       | Most recent value, usually same as cash bal                          |
| **%bal.maitenanceExcessPNL**                    | Change in maitenance excess from sod as a % of sod                   |
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

### Accounts

| Path                        | Description                                              |
|-----------------------------|----------------------------------------------------------|
| `/raw/account/list`         | Json array of account names.                             |
| `/raw/account/$indentifier` | Json object of account info.                             |
|                             | Where **$identifier** is the account *name* or *number*. |


## Author

By: **Curtis Jones** <*mail@curtisjones.ca*>
