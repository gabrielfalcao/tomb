use crate::{colors, ioutils::absolute_path};

use console::style;
use serde::de::DeserializeOwned;
use serde::Serialize;

use std::{
    fs::{self, File},
    io::Write,
};

pub trait YamlFileError {
    fn with_message(message: String) -> Self;
}

pub trait YamlFile<Error: YamlFileError> {
    fn from_yaml<'a>(data: String) -> Result<Self, Error>
    where
        Self: DeserializeOwned,
        Self: Clone,
        Self: PartialEq,
    {
        let cfg: Self = match serde_yaml::from_str(&data) {
            Ok(config) => config,
            Err(error) => {
                return Err(Error::with_message(format!(
                    "failed to deserialize yaml config: {}",
                    error
                )))
            }
        };
        Ok(cfg)
    }
    /// Serialize YamlFile to String
    fn to_yaml(&self) -> Result<String, Error>
    where
        Self: Serialize,
    {
        match serde_yaml::to_string(&self) {
            Ok(val) => Ok(val),
            Err(e) => Err(Error::with_message(format!(
                "failed to encode key to yaml: {}",
                e
            ))),
        }
    }

    /// Loads the default config somehow
    fn default() -> Result<Self, Error>
    where
        Self: DeserializeOwned;

    /// Loads the default config from a yaml file
    fn import(filename: &str) -> Result<Self, Error>
    where
        Self: DeserializeOwned,
        Self: Clone,
        Self: PartialEq,
    {
        let filename = absolute_path(filename);
        match fs::read_to_string(filename.as_str()) {
            Ok(yaml) => YamlFile::from_yaml(yaml),
            Err(error) => {
                return Err(Error::with_message(format!(
                    "{}{}{}",
                    style("failed to read file ").color256(colors::ERR_MSG),
                    style(filename).color256(colors::ERR_VAR),
                    style(format!("\n\t{}", error)).color256(colors::ERR_HLT),
                )))
            }
        }
    }
    /// Store YAML-serialized key into a file
    fn export(&self, filename: &str) -> Result<String, Error>
    where
        Self: Serialize,
    {
        let filename = absolute_path(filename);

        let yaml = match self.to_yaml() {
            Ok(val) => val,
            Err(error) => return Err(error),
        };
        let mut file = match File::create(filename.as_str()) {
            Ok(file) => file,
            Err(error) => {
                return Err(Error::with_message(format!(
                    "{}{}{}",
                    style("failed to create file ").color256(colors::ERR_MSG),
                    style(filename).color256(colors::ERR_VAR),
                    style(format!("\n\t{}", error)).color256(colors::ERR_HLT),
                )))
            }
        };
        file.write(yaml.as_ref()).unwrap();
        Ok(filename)
    }
}
