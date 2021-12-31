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

use std::{borrow::Borrow, fmt};
pub const DEFAULT_TOMB_CONFIG_PATH: &'static str = "~/.tombconfig";
pub fn default_tomb_config_filename() -> String {
    match std::env::var("TOMB_CONFIG") {
        Ok(filename) => String::from(shellexpand::tilde(&filename)),
        Err(_error) => String::from(DEFAULT_TOMB_CONFIG_PATH),
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
    pub ui_color: String,
    pub key_filename: String,
    pub tomb_filename: String,
    pub version: Option<String>,
}
impl YamlFile<Error> for TombConfig {
    fn default() -> Result<TombConfig, Error> {
        let filename = shellexpand::tilde(DEFAULT_TOMB_CONFIG_PATH);
        TombConfig::import(filename.borrow())
    }
}

impl TombConfig {
    /// Creates a new tomb config in memory
    pub fn new(ui_color: &str, _key_filename: &str, tomb_filename: &str) -> TombConfig {
        TombConfig {
            version: Some(version()),
            ui_color: ui_color.to_string(),
            key_filename: tomb_filename.to_string(),
            tomb_filename: tomb_filename.to_string(),
        }
    }
    pub fn default() -> TombConfig {
        TombConfig::new("cyan", &default_key_filename(), &default_tomb_filename())
    }
    pub fn set_ui_color(&mut self, color: &str) {
        self.ui_color = color.to_string();
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
