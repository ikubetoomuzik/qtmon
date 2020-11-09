//! Config loading and saving module, gonna use Rust Object Notation (RON).
//! By: Curtis Jones <mail@curtisjones.ca>
//! Started on: November 8, 2020

use super::http_server::RestApiFeatures;
use chrono::{DateTime, Utc};
use clap::clap_app;
use questrade::AuthenticationInfo;
use ron::{from_str, to_string};
use serde::{Deserialize, Serialize};
use std::{
    fs::{read_to_string, File},
    io::Write,
};

#[derive(Debug)]
struct Config {
    path: String,
    file: ConfigFile,
    auth: AuthInfo,
}

#[derive(Debug, Serialize, Deserialize)]
struct ConfigFile {
    auth_file_path: String,
    web_interface_enabled: bool,
    rest_api_enabled: bool,
    rest_api_features: Vec<RestApiFeatures>,
}

impl ConfigFile {
    fn load(file: &str) -> ron::Result<Self> {
        let input = read_to_string(file)?;
        Ok(from_str::<Self>(&input)?)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct SavedAuthInfo {
    refresh_token: String,
    access_token: String,
    expires_at: DateTime<Utc>,
    api_server: String,
    is_demo: bool,
}

impl SavedAuthInfo {
    fn convert_to_api(self) -> AuthenticationInfo {
        AuthenticationInfo {
            refresh_token: self.refresh_token,
            access_token: self.access_token,
            expires_at: std::time::Instant::now(),
            api_server: self.api_server,
            is_demo: self.is_demo,
        }
    }
    fn convert_from_api(api_auth: AuthenticationInfo) -> Result<Self, Box<dyn std::error::Error>> {
        let expires_at = api_auth.expires_at;
        let duration = std::time::Instant::now() - expires_at;
        let duration = chrono::Duration::from_std(duration)?;
        let expires_at = Utc::now() - duration;
        Ok(SavedAuthInfo {
            refresh_token: api_auth.refresh_token,
            access_token: api_auth.access_token,
            expires_at,
            api_server: api_auth.api_server,
            is_demo: api_auth.is_demo,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
enum AuthInfo {
    RefreshToken(String),
    FullAuthInfo(SavedAuthInfo),
}

impl AuthInfo {
    // *** Basic I/O Functions ***
    fn load(file: &str) -> ron::Result<Self> {
        let input = read_to_string(file)?;
        Ok(from_str::<Self>(&input)?)
    }
    fn save(&self, file: &str) -> ron::Result<()> {
        let mut file = File::create(file)?;
        let output = to_string::<Self>(self)?;
        file.write_all(output.as_bytes())?;
        Ok(())
    }
    // *** Conversion Functions ***
    fn convert_to_api_auth(self) -> Result<AuthenticationInfo, String> {
        match self {
            Self::RefreshToken(s) => Err(s),
            Self::FullAuthInfo(sai) => Ok(sai.convert_to_api()),
        }
    }
    fn convert_from_api_auth(
        api_auth: AuthenticationInfo,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self::FullAuthInfo(SavedAuthInfo::convert_from_api(
            api_auth,
        )?))
    }
}
