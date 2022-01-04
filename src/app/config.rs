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

pub fn default_tomb_config_filename() -> String {
    match std::env::var("TOMB_CONFIG") {
        Ok(filename) => String::from(shellexpand::tilde(&filename)),
        Err(_error) => String::from(TOMB_CONFIG),
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
pub struct ColorTheme {
    pub default: String,
    pub light: String,
    pub blurred: String,
    pub default_fg: String,
    pub default_bg: String,
    pub error_fg: String,
    pub error_bg: String,
}
impl ColorTheme {
    pub fn builtin() -> ColorTheme {
        ColorTheme {
            // https://coolors.co/palletes/trending :)
            default: "#4f5d75".to_string(),
            light: "#ffd400".to_string(),
            blurred: "#998a63".to_string(),
            default_fg: "#f5cb5c".to_string(),
            default_bg: "#001219".to_string(),
            error_fg: "#ff7f51".to_string(),
            error_bg: "#242423".to_string(),
        }
    }
}
#[derive(PartialEq, Clone, Serialize, Deserialize)]
pub struct TombConfig {
    pub colors: ColorTheme,
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
        key_filename: &str,
        tomb_filename: &str,
        log_filename: &str,
        colors: ColorTheme,
    ) -> TombConfig {
        TombConfig {
            version: Some(version()),
            key_filename: key_filename.to_string(),
            tomb_filename: tomb_filename.to_string(),
            log_filename: log_filename.to_string(),
            colors,
        }
    }
    pub fn builtin() -> TombConfig {
        TombConfig::new(
            &default_key_filename(),
            &default_tomb_filename(),
            &default_log_filename(),
            ColorTheme::builtin(),
        )
    }
    pub fn load() -> TombConfig {
        TombConfig::default().unwrap_or(TombConfig::builtin())
    }
    pub fn set_colors(&mut self, colors: ColorTheme) {
        self.colors = colors.clone();
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
