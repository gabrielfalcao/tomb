use crate::colors;
use crate::logger;
use chrono::prelude::*;
use console::style;

use std::collections::BTreeMap;

use std::fs::{File, OpenOptions};
use std::io::{BufReader, Read, Write};

use shellexpand;
use std::path::Path;
use std::{env, fmt, fs};

pub fn absolute_path(src: &str) -> String {
    String::from(shellexpand::tilde(src))
}

pub fn homedir() -> String {
    absolute_path("~")
}

#[derive(Debug, Clone)]
pub struct Error {
    pub message: String,
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}
impl Error {
    pub fn with_message(message: String) -> Error {
        Error {
            message: logger::paint::error(format!("{}", message)),
        }
    }
}

pub fn open_read(filename: &str) -> Result<File, Error> {
    let filename = absolute_path(filename);
    match File::open(filename.as_str()) {
        Ok(file) => Ok(file),
        Err(error) => {
            return Err(Error::with_message(format!(
                "{}{}{}",
                style("cannot open file ").color256(colors::ERR_MSG),
                style(filename).color256(colors::ERR_VAR),
                style(format!("\n\t{}", error)).color256(colors::ERR_HLT),
            )));
        }
    }
}
pub fn open_write(target: &str) -> Result<std::fs::File, Error> {
    let target = absolute_path(target);
    match OpenOptions::new()
        .create(true)
        .write(true)
        .open(target.as_str())
    {
        Ok(file) => Ok(file),
        Err(error) => {
            return Err(Error::with_message(format!(
                "{}{}{}{}",
                style("failed to open file ").color256(colors::ERR_MSG),
                style(target).color256(colors::ERR_VAR),
                style("in write mode").color256(colors::ERR_MSG),
                style(format!("\n\t{}", error)).color256(colors::ERR_HLT),
            )));
        }
    }
}
pub fn open_append(target: &str) -> Result<std::fs::File, Error> {
    let target = absolute_path(target);
    match OpenOptions::new()
        .create(true)
        .append(true)
        .open(target.as_str())
    {
        Ok(file) => Ok(file),
        Err(error) => {
            return Err(Error::with_message(format!(
                "{}{}{}{}",
                style("failed to open file ").color256(colors::ERR_MSG),
                style(target).color256(colors::ERR_VAR),
                style("in append mode").color256(colors::ERR_MSG),
                style(format!("\n\t{}", error)).color256(colors::ERR_HLT),
            )));
        }
    }
}
pub fn create_file(filename: &str) -> Result<std::fs::File, Error> {
    let filename = absolute_path(filename);
    match File::create(filename.as_str()) {
        Ok(file) => Ok(file),
        Err(error) => {
            return Err(Error::with_message(format!(
                "{}{}{}{}",
                style("failed to create file ").color256(colors::ERR_MSG),
                style(filename).color256(colors::ERR_VAR),
                style("in write mode").color256(colors::ERR_MSG),
                style(format!("\n\t{}", error)).color256(colors::ERR_HLT),
            )));
        }
    }
}
pub fn read_file(filename: &str) -> Result<String, Error> {
    let mut file = match open_read(filename) {
        Ok(file) => file,
        Err(e) => return Err(e),
    };
    let mut text = String::new();
    match file.read_to_string(&mut text) {
        Ok(_) => {}
        Err(error) => {
            return Err(Error::with_message(format!(
                "{}{}{}",
                style("reading file ").color256(colors::ERR_MSG),
                style(filename).color256(colors::ERR_VAR),
                style(format!("\n\t{}", error)).color256(colors::ERR_HLT),
            )));
        }
    }
    Ok(text)
}

/// Encodes &[u8] to a base64 string
///
/// # Example
///
/// ```
/// use tomb::ioutils::b64encode;
/// assert_eq!("SGVsbG8=", b64encode(b"Hello"));
/// ```
pub fn b64encode(bytes: &[u8]) -> String {
    base64::encode(bytes)
}

/// Encodes base64 string into a Vec<8>
///
/// # Example
///
/// ```
/// use tomb::ioutils::b64decode;
/// assert_eq!(b"Hello".to_vec(), b64decode(b"SGVsbG8=").unwrap());
/// ```
pub fn b64decode(bytes: &[u8]) -> Result<Vec<u8>, Error> {
    let bytes = match base64::decode(&bytes) {
        Ok(bytes) => Ok(bytes),
        Err(error) => {
            return Err(Error::with_message(format!(
                "{}{}",
                style("base64 decode failed").color256(colors::ERR_MSG),
                style(format!("\n\t{}", error)).color256(colors::ERR_HLT),
            )));
        }
    };
    bytes
}

/// Reads the given filename as Vec<u8>
pub fn read_bytes(filename: &str) -> Result<Vec<u8>, Error> {
    let f = match open_read(filename) {
        Ok(file) => file,
        Err(error) => return Err(error),
    };

    let mut reader = BufReader::new(f);
    let mut buffer = Vec::new();
    match reader.read_to_end(&mut buffer) {
        Ok(bytecount) => {
            logger::err::ok(format!(
                "{}{}",
                style(format!("read {:?} bytes from file ", bytecount)).color256(colors::ERR_MSG),
                style(filename).color256(colors::ERR_VAR),
            ));
        }
        Err(error) => {
            return Err(Error::with_message(format!(
                "{}{}{}",
                style("reading file ").color256(colors::ERR_MSG),
                style(filename).color256(colors::ERR_VAR),
                style(format!("\n\t{}", error)).color256(colors::ERR_HLT),
            )));
        }
    };
    Ok(buffer)
}

pub fn delete_file(target: &str) -> Result<(), Error> {
    match fs::remove_file(target) {
        Ok(_) => {
            logger::err::warning(format!(
                "{}{}",
                style("deleted index ").color256(241),
                style(target).color256(246),
            ));
            Ok(())
        }
        Err(error) => {
            return Err(Error::with_message(format!(
                "{}{}{}",
                style("deleting ").color256(colors::ERR_MSG),
                style(target).color256(colors::ERR_VAR),
                style(format!("\n\t{}", error)).color256(colors::ERR_HLT),
            )));
        }
    }
}
pub fn delete_directory(target: &str) -> Result<(), Error> {
    match fs::remove_dir_all(target) {
        Ok(_) => Ok(()),
        Err(error) => {
            return Err(Error::with_message(format!(
                "{}{}{}",
                style("deleting ").color256(colors::ERR_MSG),
                style(target).color256(colors::ERR_VAR),
                style(format!("\n\t{}", error)).color256(colors::ERR_HLT),
            )));
        }
    }
}
pub fn directory_is_empty(target: &str) -> bool {
    let path = Path::new(&target);
    let entries = match fs::read_dir(&path) {
        Ok(values) => values,
        Err(_) => return false,
    };
    entries.count() == 0
}

pub fn rm_rf(target: &str) -> bool {
    match delete_directory(target) {
        Ok(_) => true,
        Err(_) => false,
    }
}
pub fn append_to_file(filename: &str, value: String) -> Result<(), Error> {
    let mut file = open_append(filename)?;
    match file.write_all(value.as_bytes()) {
        Ok(_) => Ok(()),
        Err(error) => Err(Error::with_message(format!(
            "cannot append to file {}: {}",
            filename, error
        ))),
    }
}
pub fn log_to_file(filename: &str, value: String) -> Result<(), Error> {
    let value = format!("[{}] {}\n", Utc::now(), value);
    append_to_file(filename, value)
}
pub fn write_map_to_yaml(map: &BTreeMap<String, String>, filename: &str) -> Result<(), Error> {
    let mut file = open_write(filename).unwrap();
    let yaml = match serde_yaml::to_string(map) {
        Ok(s) => s,
        Err(error) => {
            return Err(Error::with_message(format!(
                "{}{}{}",
                style("failed to serialze data to yaml ").color256(colors::ERR_MSG),
                style(filename).color256(colors::ERR_VAR),
                style(format!("\n\t{}", error)).color256(colors::ERR_HLT),
            )));
        }
    };
    match file.write_all(yaml.as_bytes()) {
        Ok(_) => {
            return Ok(());
        }
        Err(error) => {
            return Err(Error::with_message(format!(
                "{}{}{}",
                style("failed to write yaml data to: ").color256(colors::ERR_MSG),
                style(filename).color256(colors::ERR_VAR),
                style(format!("\n\t{}", error)).color256(colors::ERR_HLT),
            )));
        }
    }
}

pub fn get_cwd() -> Result<String, Error> {
    match env::current_dir() {
        Ok(pbuf) => match pbuf.as_path().canonicalize() {
            Ok(current_dir) => match current_dir.as_os_str().to_str() {
                Some(path) => Ok(String::from(path)),
                None => {
                    return Err(Error::with_message(format!(
                        "{}",
                        style("failed convert cwd path to string").color256(colors::ERR_HLT),
                    )));
                }
            },
            Err(error) => {
                return Err(Error::with_message(format!(
                    "{}{}",
                    style("failed to calculate absolute path of current working directory")
                        .color256(colors::ERR_MSG),
                    style(format!("\n\t{}", error)).color256(colors::ERR_HLT),
                )));
            }
        },
        Err(error) => {
            return Err(Error::with_message(format!(
                "{}{}",
                style("failed to retrieve current working directory").color256(colors::ERR_HLT),
                style(format!("\n\t{}", error)).color256(colors::ERR_HLT),
            )));
        }
    }
}
