//! Simple http server to provide styled output.
//! By: Curtis Jones <mail@curtisjones.ca>
//! Started on: November 8, 2020

use super::{
    storage::DBRefWeak,
    util::{
        tokio,
        warp::{self, Filter},
        Deserialize, Serialize,
    },
};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RestApiFeature {
    Lemonbar,
    Polybar,
    Unibar,
    // Custom(String),
}

#[derive(Debug)]
/// Struct to hold a reference to the thread that is runnning the http server.
pub struct HTTPServer {
    handle: tokio::task::JoinHandle<()>,
}

impl HTTPServer {
    pub fn new(db: DBRefWeak, rest_api_features: Vec<RestApiFeature>) -> Self {
        // for now the routes are just a simple hello..
        let routes = warp::any().map(|| "Hello there...");
        // print it out babyyy.
        println!("Starting HTTP server...");
        // here is the actual start of the server..
        let server = warp::serve(routes).bind(([127, 0, 0, 1], 8080));
        HTTPServer {
            handle: tokio::spawn(server),
        }
    }
}
