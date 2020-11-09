//! Simple http server to provide styled output.
//! By: Curtis Jones <mail@curtisjones.ca>
//! Started on: November 8, 2020

use serde::{Deserialize, Serialize};
use warp;

#[derive(Debug, Serialize, Deserialize)]
pub enum RestApiFeatures {}
