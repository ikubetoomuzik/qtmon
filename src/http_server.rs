//! Simple http server to provide styled output.
//! By: Curtis Jones <mail@curtisjones.ca>
//! Started on: November 8, 2020

use super::{
    include::{
        json, tokio,
        warp::{self, Filter},
        Deserialize, Json, Local, NaiveDate, NaiveTime, Result, Serialize,
    },
    storage::{DBInfoAccountBalance, DBRef},
};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorReply {
    message: String,
}
impl ErrorReply {
    pub fn new(message: String) -> Self {
        Self { message }
    }
}

#[derive(Debug)]
/// Struct to hold a reference to the thread that is runnning the http server.
pub struct HTTPServer {
    handle: tokio::task::JoinHandle<()>,
}

impl HTTPServer {
    pub fn new(port: u16, db: DBRef) -> Self {
        // for now the routes are just a simple hello..
        let any = warp::any().map(|| "Hello there...");
        // my first try at a basic api..
        let raw = warp::path("raw");
        let raw_balance = raw.and(warp::path("balance"));
        let raw_balance_latest = raw_balance
            .and(warp::path!(String / "latest"))
            .and(warp::path::end());
        let raw_balance_latest_date = raw_balance
            .and(warp::path!(String / String / "latest"))
            .and(warp::path::end());
        let raw_balance_date_time = raw_balance
            .and(warp::path!(String / String / String))
            .and(warp::path::end());

        // clone so we can move it to the new runtime
        let db_bbdt = db.clone();
        // and now we format our actual response.
        let raw_balance_date_time =
            raw_balance_date_time.map(move |a: String, b: String, c: String| -> Json {
                let date = match NaiveDate::parse_from_str(&b, "%Y-%m-%d") {
                    Ok(d) => d,
                    Err(e) => {
                        return json(&ErrorReply::new(format!(
                            "Could not parse date: {}. Error: {}",
                            b, e
                        )))
                    }
                };
                let time = match NaiveTime::parse_from_str(&c, "%H:%M") {
                    Ok(t) => t,
                    Err(e) => {
                        return json(&ErrorReply::new(format!(
                            "Could not parse date: {}. Error: {}",
                            b, e
                        )))
                    }
                };
                match (*db_bbdt).db.read(|db| -> Result<DBInfoAccountBalance> {
                    Ok(db.get_closest_balance(&a, date, time)?)
                }) {
                    Ok(Ok(val)) => json(&val),
                    Ok(Err(e)) => json(&ErrorReply::new(format!(
                        "Error getting latest balance. Error: {}",
                        e
                    ))),
                    Err(e) => json(&ErrorReply::new(format!(
                        "Error getting latest balance. Error: {}",
                        e
                    ))),
                }
            });

        // clone so we can move it to the new runtime
        let db_bbld = db.clone();
        // and now we format our actual response.
        let raw_balance_latest_date =
            raw_balance_latest_date.map(move |a: String, b: String| -> Json {
                let date = match NaiveDate::parse_from_str(&b, "%Y-%m-%d") {
                    Ok(d) => d,
                    Err(e) => {
                        return json(&ErrorReply::new(format!(
                            "Could not parse date: {}. Error: {}",
                            b, e
                        )))
                    }
                };
                match (*db_bbld).db.read(|db| -> Result<DBInfoAccountBalance> {
                    Ok(db.get_latest_balance(&a, date)?)
                }) {
                    Ok(Ok(val)) => json(&val),
                    Ok(Err(e)) => json(&ErrorReply::new(format!(
                        "Error getting latest balance. Error: {}",
                        e
                    ))),
                    Err(e) => json(&ErrorReply::new(format!(
                        "Error getting latest balance. Error: {}",
                        e
                    ))),
                }
            });

        // clone so we can move it to the new runtime
        let db_bbl = db.clone();
        // and now we format our actual response.
        let raw_balance_latest = raw_balance_latest.map(move |a: String| -> Json {
            match (*db_bbl).db.read(|db| -> Result<DBInfoAccountBalance> {
                Ok(db.get_latest_balance(&a, Local::today().naive_local())?)
            }) {
                Ok(Ok(val)) => json(&val),
                Ok(Err(e)) => json(&ErrorReply::new(format!(
                    "Error getting latest balance. Error: {}",
                    e
                ))),
                Err(e) => json(&ErrorReply::new(format!(
                    "Error getting latest balance. Error: {}",
                    e
                ))),
            }
        });

        // combine up the baic methods.
        let raw = raw_balance_latest
            .or(raw_balance_latest_date)
            .or(raw_balance_date_time);

        // combine her up.
        let routes = warp::get().and(raw.or(any));
        // print it out babyyy.
        println!("Starting HTTP server...");
        // here is the actual start of the server..
        let server = warp::serve(routes).bind(([127, 0, 0, 1], port));
        HTTPServer {
            handle: tokio::spawn(server),
        }
    }
}
