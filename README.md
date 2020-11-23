# qtmon
Version: **0.1.0**

## Installation
The project has been uploaded to crates.io and can be downloaded with:
```sh
cargo install qtmon
```
## Config
Instructions on configuration of the application can be found [here](./src/config).

## Usage
The program uses [Rusty-Object-Notation](https://github.com/ron-rs/ron) or RON for it's default data storage.
You may optionally enable a Yaml or Bincode encoding by disabling default and
including either feature.


The default config location is *$XDGCONFIGDIR/qtmon/config.ron* or 
*$HOME/.config/qtmon/config.ron* and you can override it with the *-c* option.
By default the program will generate a default config file the first time it is run.
To connect the first time you will need to provide a **Questrade API Refresh Token**
as the *-r* arguement to the program.
Instructions on enabling the API for your account and generating a new token can 
be found [here](https://www.questrade.com/api/documentation/getting-started).


Once you've run the program once with your initial token it should be able to
manage authentication from there. All you need to do is query your localhost at the 
port you selected in your config to get the up-to-date details on your account.

## API
Documentation for the REST API can be found [here](./src/http_server/README.md).

## TODO
* [ ] Go back and comment throughout the project, most specifically in the *storage* module.
* [ ] Add examples to the REST Api README.

## DONE
* [x] Add statusbar api.
* [x] REST Api to actually get info.
* [x] Add license. Probably just MIT.
* [x] Fill out config README more completely.
* [x] Fill out general README more completely.
* [x] Fill out REST Api README more completely.
* [x] Fix the way that *Config* loading parses non-root paths provided.
* [x] Add a string encoded default config to produce on first run if needed.
* [x] Split *storage* module into seperate modules, file is over 400 lines of code..
* [x] Review dependencies to see if there is anything I can strip off of the dependency chart.
* [x] Implement real event loop with error handling so all these Results I'm making are useful.


## Author

By: **Curtis Jones** <*mail@curtisjones.ca*>
