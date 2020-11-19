//! Simple http server to provide styled output.
//! By: Curtis Jones <mail@curtisjones.ca>
//! Started on: November 8, 2020

use super::{
    include::{
        error, info, json, tokio, warn,
        warp::{self, Filter},
        with_status, Deserialize, Duration, Ipv4Addr, Json, Local, NaiveDate, NaiveTime, Result,
        Serialize, SocketAddr, SocketAddrV4, StatusCode,
    },
    storage::{DBInfoAccountBalance, DBInfoAccountPosition, DBRef},
};

// Helper functions
fn parse_date(date_str: String) -> std::result::Result<NaiveDate, Json> {
    match NaiveDate::parse_from_str(&date_str, "%Y-%m-%d") {
        Ok(d) => Ok(d),
        Err(e) => Err(json(&ErrorReply::new(format!(
            "Could not parse date: {}. Error: {}",
            date_str, e
        )))),
    }
}
fn parse_time(time_str: String) -> std::result::Result<NaiveTime, Json> {
    match NaiveTime::parse_from_str(&time_str, "%H:%M") {
        Ok(t) => Ok(t),
        Err(e) => Err(json(&ErrorReply::new(format!(
            "Could not parse date: {}. Error: {}",
            time_str, e
        )))),
    }
}

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
    pub fn new(addr: Ipv4Addr, port: u16, db: DBRef) -> Self {
        // gen the any filter.
        let any = warp::any().map(|| with_status(format!("Not implemented."), StatusCode::OK));
        // gen the log filters
        let log = warp::filters::log::custom(|info| {
            let log_str = format!(
                "[{} FROM {} {}] Path: {}. Time for response: {}µs.",
                info.method(),
                info.remote_addr()
                    .unwrap_or(SocketAddr::V4(SocketAddrV4::new(
                        Ipv4Addr::new(u8::MAX, u8::MAX, u8::MAX, u8::MAX),
                        u16::MAX
                    ))),
                info.user_agent().unwrap_or_default(),
                info.path(),
                Duration::from_std(info.elapsed())
                    .unwrap_or(Duration::zero())
                    .num_microseconds()
                    .unwrap_or_default(),
            );
            match info.status() {
                StatusCode::OK => info!("{}", log_str),
                StatusCode::INTERNAL_SERVER_ERROR => error!("{}", log_str),
                _ => warn!("{}", log_str),
            }
        });
        //  the raw json api
        let raw = warp::path("raw");
        // ** /raw/position paths
        let raw_position = raw.and(warp::path("position"));
        let raw_position_list = raw_position
            .and(warp::path!(String / "list"))
            .and(warp::path::end());
        let raw_position_latest = raw_position
            .and(warp::path!(String / String / "latest"))
            .and(warp::path::end());
        let raw_position_date_latest = raw_position
            .and(warp::path!(String / String / String / "latest"))
            .and(warp::path::end());
        let raw_position_date_time = raw_position
            .and(warp::path!(String / String / String / String))
            .and(warp::path::end());
        // ** /raw/balance paths
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
        let db_rpl = db.clone();
        let raw_position_list = raw_position_list.map(move |a: String| -> Json {
            match (*db_rpl)
                .db
                .read(|db| -> Result<Vec<String>> { Ok(db.get_position_symbols(a)?) })
            {
                Ok(Ok(val)) => json(&val),
                Ok(Err(e)) => json(&ErrorReply::new(format!(
                    "Error getting position list. Error: {}",
                    e
                ))),
                Err(e) => json(&ErrorReply::new(format!(
                    "Error getting position list. Error: {}",
                    e
                ))),
            }
        });

        // clone so we can move it to the new runtime
        let db_rplatest = db.clone();
        let raw_position_latest = raw_position_latest.map(move |a: String, b: String| -> Json {
            match (*db_rplatest)
                .db
                .read(|db| -> Result<DBInfoAccountPosition> {
                    Ok(db.get_latest_position(a, b, Local::today().naive_local())?)
                }) {
                Ok(Ok(val)) => json(&val),
                Ok(Err(e)) => json(&ErrorReply::new(format!(
                    "Error getting position list. Error: {}",
                    e
                ))),
                Err(e) => json(&ErrorReply::new(format!(
                    "Error getting position list. Error: {}",
                    e
                ))),
            }
        });

        // clone so we can move it to the new runtime
        let db_rpdlatest = db.clone();
        let raw_position_date_latest =
            raw_position_date_latest.map(move |a: String, b: String, c: String| -> Json {
                let date = match parse_date(c) {
                    Ok(d) => d,
                    Err(e) => return e,
                };
                match (*db_rpdlatest)
                    .db
                    .read(|db| -> Result<DBInfoAccountPosition> {
                        Ok(db.get_latest_position(a, b, date)?)
                    }) {
                    Ok(Ok(val)) => json(&val),
                    Ok(Err(e)) => json(&ErrorReply::new(format!(
                        "Error getting position list. Error: {}",
                        e
                    ))),
                    Err(e) => json(&ErrorReply::new(format!(
                        "Error getting position list. Error: {}",
                        e
                    ))),
                }
            });

        // clone so we can move it to the new runtime
        let db_rpdtime = db.clone();
        let raw_position_date_time =
            raw_position_date_time.map(move |a: String, b: String, c: String, d: String| -> Json {
                let date = match parse_date(c) {
                    Ok(d) => d,
                    Err(e) => return e,
                };
                let time = match parse_time(d) {
                    Ok(t) => t,
                    Err(e) => return e,
                };
                match (*db_rpdtime)
                    .db
                    .read(|db| -> Result<DBInfoAccountPosition> {
                        Ok(db.get_closest_position(a, b, date, time)?)
                    }) {
                    Ok(Ok(val)) => json(&val),
                    Ok(Err(e)) => json(&ErrorReply::new(format!(
                        "Error getting position list. Error: {}",
                        e
                    ))),
                    Err(e) => json(&ErrorReply::new(format!(
                        "Error getting position list. Error: {}",
                        e
                    ))),
                }
            });

        // clone so we can move it to the new runtime
        let db_rbdt = db.clone();
        // and now we format our actual response.
        let raw_balance_date_time =
            raw_balance_date_time.map(move |a: String, b: String, c: String| -> Json {
                let date = match parse_date(b) {
                    Ok(d) => d,
                    Err(e) => return e,
                };
                let time = match parse_time(c) {
                    Ok(t) => t,
                    Err(e) => return e,
                };
                match (*db_rbdt).db.read(|db| -> Result<DBInfoAccountBalance> {
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
        let db_rbld = db.clone();
        // and now we format our actual response.
        let raw_balance_latest_date =
            raw_balance_latest_date.map(move |a: String, b: String| -> Json {
                let date = match parse_date(b) {
                    Ok(d) => d,
                    Err(e) => return e,
                };
                match (*db_rbld).db.read(|db| -> Result<DBInfoAccountBalance> {
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
        let db_rbl = db.clone();
        // and now we format our actual response.
        let raw_balance_latest =
            raw_balance_latest.map(move |a: String| -> warp::reply::WithStatus<Json> {
                match (*db_rbl).db.read(|db| -> Result<DBInfoAccountBalance> {
                    Ok(db.get_latest_balance(&a, Local::today().naive_local())?)
                }) {
                    Ok(Ok(val)) => with_status(json(&val), StatusCode::OK),
                    Ok(Err(e)) => with_status(
                        json(&ErrorReply::new(format!(
                            "Error getting latest balance. Error: {}",
                            e
                        ))),
                        StatusCode::NOT_FOUND,
                    ),
                    Err(e) => with_status(
                        json(&ErrorReply::new(format!(
                            "Error getting latest balance. Error: {}",
                            e
                        ))),
                        StatusCode::NOT_FOUND,
                    ),
                }
            });

        // combine up the baic methods.
        let raw = raw_balance_latest
            .or(raw_balance_latest_date)
            .or(raw_balance_date_time)
            .or(raw_position_list)
            .or(raw_position_latest)
            .or(raw_position_date_latest)
            .or(raw_position_date_time);

        // combine her up.
        let routes = warp::get().and(raw.or(any)).with(log);
        // print it out babyyy.
        info!("Starting HTTP server @ [{}:{}]...", addr, port);
        // here is the actual start of the server..
        // split the addr arg.
        let server = warp::serve(routes).bind((addr, port));
        HTTPServer {
            handle: tokio::spawn(server),
        }
    }
}
