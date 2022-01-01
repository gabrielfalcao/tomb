use super::logging::*;
use crate::aes256cbc::default_key_filename;
use crate::core::version;
use crate::tomb::default_tomb_filename;
use crate::{
    config::{YamlFile, YamlFileError},
    logger,
};
use serde::{Deserialize, Serialize};
use shellexpand;
use std::fmt;

pub const TOMB_CONFIG: &'static str = "~/.tomb.config.yaml";
pub const TOMB_LOG: &'static str = "~/.tomb.log";

pub fn default_tomb_config_filename() -> String {
    match std::env::var("TOMB_CONFIG") {
        Ok(filename) => String::from(shellexpand::tilde(&filename)),
        Err(_error) => String::from(TOMB_CONFIG),
    }
}
pub fn default_log_filename() -> String {
    match std::env::var("TOMB_LOG") {
        Ok(filename) => String::from(shellexpand::tilde(&filename)),
        Err(_error) => String::from(TOMB_LOG),
    }
}
#[derive(Debug, Clone)]
pub struct Error {
    pub message: String,
}
impl Error {
    pub fn with_message(message: String) -> Error {
        Error {
            message: logger::paint::error(format!("{}", message)),
        }
    }
}

impl YamlFileError for Error {
    fn with_message(message: String) -> Error {
        Error {
            message: logger::paint::error(format!("{}", message)),
        }
    }
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

#[derive(PartialEq, Clone, Serialize, Deserialize)]
pub struct TombConfig {
    pub color_default: String,
    pub color_light: String,
    pub key_filename: String,
    pub tomb_filename: String,
    pub log_filename: String,
    pub version: Option<String>,
}

impl YamlFile<Error> for TombConfig {
    fn default() -> Result<TombConfig, Error> {
        TombConfig::import(default_tomb_config_filename().as_str())
    }
}

impl TombConfig {
    /// Creates a new tomb config in memory
    pub fn new(
        color_default: &str,
        color_light: &str,
        key_filename: &str,
        tomb_filename: &str,
        log_filename: &str,
    ) -> TombConfig {
        TombConfig {
            version: Some(version()),
            color_default: color_default.to_string(),
            color_light: color_light.to_string(),
            key_filename: key_filename.to_string(),
            tomb_filename: tomb_filename.to_string(),
            log_filename: log_filename.to_string(),
        }
    }
    pub fn builtin() -> TombConfig {
        TombConfig::new(
            "D740B3",
            "FF6FFC",
            &default_key_filename(),
            &default_tomb_filename(),
            &default_log_filename(),
        )
    }
    pub fn load() -> TombConfig {
        TombConfig::default().unwrap_or(TombConfig::builtin())
    }
    pub fn save(&mut self) -> Result<(), Error> {
        let filename = default_tomb_config_filename();
        match self.export(&filename) {
            Ok(_) => {
                log_error(format!("config saved: {}", filename));
                Ok(())
            }
            Err(error) => Err(Error::with_message(format!(
                "cannot save config {}: {}",
                filename, error
            ))),
        }
    }
}
