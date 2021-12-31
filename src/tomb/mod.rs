pub mod logging;
use crate::aes256cbc::{Config as AesConfig, Digest, Key};

use crate::core::version;
use crate::{
    config::{YamlFile, YamlFileError},
    ioutils::{b64decode, b64encode},
    logger,
};
use chrono::prelude::*;
use console::style;
use fnmatch_regex::glob_to_regex;
use logging::*;
use md5;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::{borrow::Borrow, fmt};
pub const DEFAULT_TOMB_PATH: &'static str = "~/.tomb.yaml";

pub fn default_tomb_filename() -> String {
    match std::env::var("TOMB_FILE") {
        Ok(filename) => filename,
        Err(_err) => String::from(DEFAULT_TOMB_PATH),
    }
}
pub fn path_to_md5(path: &str) -> String {
    format!("{:x}", md5::compute(String::from(path).as_bytes()))
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

/// The Key struct
///
/// It contains the cycles for key, salt and iv used in key derivation.
#[derive(PartialEq, Clone, Serialize, Deserialize)]
pub struct AES256Secret {
    pub digest: Digest,
    pub path: String,
    pub value: String,
    pub notes: Option<String>,
    pub username: Option<String>,
    pub url: Option<String>,
    pub attributes: Option<BTreeMap<String, String>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
impl AES256Secret {
    /// Creates a new tomb based on a key
    pub fn new(path: String, value: Vec<u8>, key: Key) -> AES256Secret {
        AES256Secret {
            digest: key.digest(),
            path,
            value: b64encode(&value),
            notes: None,
            username: None,
            url: None,
            attributes: Some(BTreeMap::new()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
    pub fn key(&self) -> String {
        path_to_md5(self.path.as_str())
    }
    pub fn set_notes(&mut self, notes: Option<String>) {
        self.notes = notes;
    }
    pub fn with_notes(&mut self, notes: Option<String>) -> AES256Secret {
        self.set_notes(notes);
        self.clone()
    }
    pub fn set_username(&mut self, username: Option<String>) {
        self.username = username;
    }
    pub fn with_username(&mut self, username: Option<String>) -> AES256Secret {
        self.set_username(username);
        self.clone()
    }
    pub fn set_url(&mut self, url: Option<String>) {
        self.url = url;
    }
    pub fn with_url(&mut self, url: Option<String>) -> AES256Secret {
        self.set_url(url);
        self.clone()
    }
    pub fn value_bytes(&self) -> Vec<u8> {
        b64decode(&self.value.as_bytes()).unwrap()
    }
    pub fn update(&mut self, path: String, plaintext: Vec<u8>, key: Key) -> Result<(), Error> {
        self.digest = key.digest();
        self.path = path.clone();
        let cyphertext = match key.encrypt(&plaintext) {
            Ok(cypher) => cypher,
            Err(error) => {
                return Err(Error::with_message(format!(
                    "{}{}{}{}",
                    style("cannot encrypt data for path").color256(198),
                    style(path).color256(190),
                    style(" with the provided key.").color256(198),
                    style(format!("\n\t{:?}", error)).color256(197),
                )));
            }
        };
        self.value = b64encode(&cyphertext);
        self.updated_at = Utc::now();
        Ok(())
    }
    pub fn get_base64_string(&self, path: &str, key: Key) -> Result<String, Error> {
        match self.get_bytes(path, key) {
            Ok(bytes) => Ok(b64encode(&bytes)),
            Err(error) => Err(error),
        }
    }
    pub fn get_string(&self, path: &str, key: Key) -> Result<String, Error> {
        match self.get_bytes(path, key) {
            Ok(bytes) => match String::from_utf8(bytes) {
                Ok(value) => Ok(value),
                Err(error) => {
                    return Err(Error::with_message(format!(
                        "{}{}{}{}",
                        style("cannot convert value from key ").color256(198),
                        style(path).color256(190),
                        style(" to a valid utf-8 string.").color256(198),
                        style(format!("\n\t{:?}", error)).color256(197),
                    )));
                }
            },
            Err(error) => Err(error),
        }
    }
    pub fn get_bytes(&self, path: &str, key: Key) -> Result<Vec<u8>, Error> {
        if String::from(path) != self.path {
            return Err(Error::with_message(format!(
                "path {} does not match {}",
                path, self.path
            )));
        }
        match key.decrypt(&self.value_bytes()) {
            Ok(plaintext) => Ok(plaintext),
            Err(error) => {
                return Err(Error::with_message(format!(
                    "{}{}{}{}",
                    style("cannot decrypt value from secret ").color256(198),
                    style(path).color256(190),
                    style(" with the provided key.").color256(198),
                    style(format!("\n\t{:?}", error)).color256(197),
                )));
            }
        }
    }
}

#[derive(PartialEq, Clone, Serialize, Deserialize)]
pub struct AES256Tomb {
    pub digest: Digest,
    pub config: AesConfig,
    pub filepath: Option<String>,
    pub data: BTreeMap<String, AES256Secret>,
    pub version: Option<String>,
}
impl YamlFile<Error> for AES256Tomb {
    fn default() -> Result<AES256Tomb, Error> {
        let filename = shellexpand::tilde(DEFAULT_TOMB_PATH);
        AES256Tomb::import(filename.borrow())
    }
}

impl AES256Tomb {
    /// Creates a new tomb based on a key
    pub fn new(filepath: &str, key: Key, config: AesConfig) -> AES256Tomb {
        AES256Tomb {
            digest: key.digest(),
            data: BTreeMap::new(),
            filepath: Some(String::from(filepath)),
            version: Some(version()),
            config,
        }
    }
    pub fn set_filepath(&mut self, path: &str) {
        self.filepath = Some(String::from(path))
    }
    pub fn with_filepath(&self, path: &str) -> AES256Tomb {
        let mut dolly = self.clone();
        dolly.set_filepath(path);
        dolly
    }
    pub fn save(&mut self) -> Result<String, Error> {
        let filepath = match self.filepath.clone() {
            Some(filepath) => self.export(&filepath)?,
            None => {
                return Err(Error::with_message(format!("attempt to save tomb that does not have a filepath, falling back to DEFAULT_TOMB_PATH: {}", DEFAULT_TOMB_PATH)));
            }
        };
        let new = match AES256Tomb::import(&filepath) {
            Ok(fresh_tomb) => fresh_tomb,
            Err(error) => {
                return Err(Error::with_message(format!(
                    "failed to save tomb to path {}: {}",
                    filepath, error
                )))
            }
        };
        self.data = new.data.clone();
        Ok(filepath.clone())
    }
    pub fn reload(&mut self) -> Result<(), Error> {
        let filepath = match self.filepath.clone() {
            Some(filepath) => {
                // log_error(format!("tomb reloaded: {}", filepath));
                filepath
            }
            None => {
                log_error(format!("attempt to reload tomb that does not have a filepath, falling back to DEFAULT_TOMB_PATH: {}", DEFAULT_TOMB_PATH));
                String::from(DEFAULT_TOMB_PATH)
            }
        };
        let new = match AES256Tomb::import(&filepath) {
            Ok(fresh_tomb) => fresh_tomb,
            Err(error) => {
                return Err(Error::with_message(format!(
                    "failed to reload tomb from path {}: {}",
                    filepath, error
                )))
            }
        };
        self.data = new.data.clone();
        //log_error(format!("reloaded tomb: {}", filepath));
        Ok(())
    }
    pub fn list(&self, pattern: &str) -> Result<Vec<AES256Secret>, Error> {
        let regex = match glob_to_regex(pattern) {
            Ok(regex) => regex,
            Err(error) => {
                return Err(Error::with_message(format!(
                    "invalid pattern {}{}",
                    pattern,
                    format!("\n\t{}", error),
                )))
            }
        };
        let mut result = Vec::new();
        for (md5key, secret) in &self.data {
            let path = secret.path.clone();
            if regex.is_match(&path) && (md5key.eq(&secret.key()) || md5key.eq(&path)) {
                result.push(secret.clone());
            }
        }
        Ok(result)
    }

    pub fn delete_secret(&mut self, path: &str) -> Result<(), Error> {
        let key = path_to_md5(path);
        match self.data.remove(&key) {
            Some(_) => Ok(()),
            None => Err(Error::with_message(format!("key not found {}", path))),
        }
    }
    pub fn add_secret(&mut self, path: &str, plaintext: String, key: Key) -> Result<(), Error> {
        self.add_secret_from_bytes(path, Vec::from(plaintext), key)
    }
    pub fn add_secret_from_bytes(
        &mut self,
        path: &str,
        plaintext: Vec<u8>,
        key: Key,
    ) -> Result<(), Error> {
        let cyphertext = match key.encrypt(&plaintext) {
            Ok(cypher) => cypher,
            Err(error) => {
                return Err(Error::with_message(format!(
                    "cannot encrypt data for path '{}' with the provided key: {}",
                    path,
                    format!("\n\t{:?}", error),
                )));
            }
        };
        let secret = AES256Secret::new(String::from(path), cyphertext, key);
        self.data.insert(secret.key(), secret);
        Ok(())
    }
    pub fn derive_key(&self, password: &str) -> Key {
        Key::from_password(password.as_bytes(), &self.config)
    }

    pub fn get(&self, path: &str) -> Result<AES256Secret, Error> {
        let key = path_to_md5(path);
        match self.data.get(&key) {
            Some(secret) => Ok(secret.clone()),
            None => Err(Error::with_message(format!(
                "{}{}",
                style("key (path) not found: ").color256(198),
                style(path).color256(190),
            ))),
        }
    }
    pub fn get_by_md5(&self, key: &str) -> Result<AES256Secret, Error> {
        match self.data.get(key) {
            Some(secret) => Ok(secret.clone()),
            None => Err(Error::with_message(format!(
                "{}{}",
                style("key (md5) not found: ").color256(198),
                style(key).color256(190),
            ))),
        }
    }
    pub fn get_base64_string(&self, path: &str, key: Key) -> Result<String, Error> {
        self.get(path)?.get_base64_string(path, key)
    }
    pub fn get_string(&self, path: &str, key: Key) -> Result<String, Error> {
        self.get(path)?.get_string(path, key)
    }
    pub fn get_bytes(&self, path: &str, key: Key) -> Result<Vec<u8>, Error> {
        self.get(path)?.get_bytes(path, key)
    }
}

#[cfg(test)]
mod tests {
    use crate::aes256cbc::Config as AesConfig;
    use crate::aes256cbc::Key;
    use crate::tomb::AES256Tomb;
    use k9::assert_equal;

    fn generate_key() -> (Key, AesConfig) {
        let config = AesConfig::builtin(None);
        let password = String::from("123456");
        (Key::from_password(&password.as_bytes(), &config), config)
    }
    #[test]
    fn test_create_tomb_and_manage_secrets() {
        let (key, config) = generate_key();

        let mut tomb = AES256Tomb::new("test-create-tomb.yaml", key.clone(), config);
        tomb.add_secret_from_bytes(
            "my-secret",
            Vec::from("some bytes"),
            tomb.derive_key("123456"),
        )
        .expect("secret should be added");
        tomb.add_secret(
            "another-secret",
            String::from("more bytes"),
            tomb.derive_key("123456"),
        )
        .expect("secret should be added");

        let plaintext = tomb
            .get_bytes("my-secret", tomb.derive_key("123456"))
            .expect("secret should have been stored by previous statement(s)");

        assert_equal!(plaintext, Vec::from("some bytes"));

        let plaintext = tomb
            .get_string("another-secret", tomb.derive_key("123456"))
            .expect("secret should have been stored by previous statement(s)");

        assert_equal!(plaintext, String::from("more bytes"));

        let secrets = tomb.list("*").expect("failed to list *");
        assert_equal!(secrets.len(), 2);

        let first = tomb.list("my-*").expect("failed to list my-*");
        assert_equal!(first.len(), 1);

        let last = tomb.list("another-*").expect("failed to list another-*");
        assert_equal!(last.len(), 1);
    }
}
