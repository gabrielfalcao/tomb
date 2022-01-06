use crate::colors;
use crate::logger;
use chrono::prelude::*;
use console::style;
use shellexpand;
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::Formatter;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, Read, Write};
use std::ops::Deref;
use std::path::Path;
use std::{env, fmt, fs};

pub fn absolute_path(src: &str) -> String {
    String::from(shellexpand::tilde(src))
}

pub fn homedir() -> String {
    absolute_path("~")
}

#[derive(Debug)]
pub struct IoError {
    source: Option<Box<dyn Error + 'static>>,
    kind: IoErrorKind,
}

impl IoError {
    fn failed_to_open(filename: impl AsRef<str>) -> Self {
        Self {
            source: None,
            kind: IoErrorKind::FailedToOpenFile {
                filename: filename.as_ref().into(),
            },
        }
    }

    fn with_error(mut self, error: impl Error + 'static) -> Self {
        self.source = Some(Box::new(error));
        self
    }
}

impl fmt::Display for IoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.source() {
            None => write!(f, "{}", self.kind),
            Some(error) => {
                write!(
                    f,
                    "{}\n\t{}",
                    self.kind,
                    style(error).color256(colors::ERR_HLT)
                )
            }
        }
    }
}

impl Error for IoError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source.as_ref().map(|error| error.deref())
    }
}

#[derive(Debug, Clone)]
pub enum IoErrorKind {
    FailedToOpenFile { filename: String },
}

impl fmt::Display for IoErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            IoErrorKind::FailedToOpenFile { filename } => {
                write!(
                    f,
                    "{}{}",
                    style("cannot open file ").color256(colors::ERR_MSG),
                    style(filename).color256(colors::ERR_VAR)
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_error() {
        let filename = "blob.txt";
        let error = IoError {
            source: None,
            kind: IoErrorKind::FailedToOpenFile {
                filename: filename.into(),
            },
        };

        assert_eq!(
            format!("{}", error),
            format!(
                "{}{}",
                style("cannot open file ").color256(colors::ERR_MSG),
                style(filename).color256(colors::ERR_VAR)
            )
        )
    }
}

#[derive(Debug, Clone)]
pub struct TombError {
    pub message: String,
}

impl fmt::Display for TombError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl TombError {
    pub fn with_message(message: String) -> TombError {
        TombError {
            message: logger::paint::error(format!("{}", message)),
        }
    }
}

/// Allow conversion between new error type to old type
impl From<IoError> for TombError {
    fn from(err: IoError) -> Self {
        TombError::with_message(format!("{}", err))
    }
}

pub fn open_read(filename: &str) -> Result<File, IoError> {
    let filename = absolute_path(filename);
    File::open(filename.as_str()).map_err(|err| IoError::failed_to_open(filename).with_error(err))
}

pub fn open_write(target: &str) -> Result<std::fs::File, TombError> {
    let target = absolute_path(target);
    OpenOptions::new()
        .create(true)
        .write(true)
        .open(target.as_str())
        .map_err(|error| {
            TombError::with_message(format!(
                "{}{}{}{}",
                style("failed to open file ").color256(colors::ERR_MSG),
                style(target).color256(colors::ERR_VAR),
                style("in write mode").color256(colors::ERR_MSG),
                style(format!("\n\t{}", error)).color256(colors::ERR_HLT),
            ))
        })
}

pub fn open_append(target: &str) -> Result<std::fs::File, TombError> {
    let target = absolute_path(target);
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(target.as_str())
        .map_err(|error| {
            TombError::with_message(format!(
                "{}{}{}{}",
                style("failed to open file ").color256(colors::ERR_MSG),
                style(target).color256(colors::ERR_VAR),
                style("in append mode").color256(colors::ERR_MSG),
                style(format!("\n\t{}", error)).color256(colors::ERR_HLT),
            ))
        })
}

pub fn create_file(filename: &str) -> Result<std::fs::File, TombError> {
    let filename = absolute_path(filename);
    File::create(filename.as_str()).map_err(|error| {
        TombError::with_message(format!(
            "{}{}{}{}",
            style("failed to create file ").color256(colors::ERR_MSG),
            style(filename).color256(colors::ERR_VAR),
            style("in write mode").color256(colors::ERR_MSG),
            style(format!("\n\t{}", error)).color256(colors::ERR_HLT),
        ))
    })
}

pub fn read_file(filename: &str) -> Result<String, TombError> {
    let mut file = open_read(filename)?;
    let mut text = String::new();
    file.read_to_string(&mut text).map_err(|error| {
        TombError::with_message(format!(
            "{}{}{}",
            style("reading file ").color256(colors::ERR_MSG),
            style(filename).color256(colors::ERR_VAR),
            style(format!("\n\t{}", error)).color256(colors::ERR_HLT),
        ))
    })?;
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
pub fn b64decode(bytes: &[u8]) -> Result<Vec<u8>, TombError> {
    base64::decode(&bytes).map_err(|error| {
        TombError::with_message(format!(
            "{}{}",
            style("base64 decode failed").color256(colors::ERR_MSG),
            style(format!("\n\t{}", error)).color256(colors::ERR_HLT),
        ))
    })
}

/// Reads the given filename as Vec<u8>
pub fn read_bytes(filename: &str) -> Result<Vec<u8>, TombError> {
    let mut reader = BufReader::new(open_read(filename)?);
    let mut buffer = Vec::new();
    reader
        .read_to_end(&mut buffer)
        .and_then(|byte_count| {
            logger::err::ok(format!(
                "{}{}",
                style(format!("read {:?} bytes from file ", byte_count)).color256(colors::ERR_MSG),
                style(filename).color256(colors::ERR_VAR),
            ));
            Ok(())
        })
        .map_err(|error| {
            TombError::with_message(format!(
                "{}{}{}",
                style("reading file ").color256(colors::ERR_MSG),
                style(filename).color256(colors::ERR_VAR),
                style(format!("\n\t{}", error)).color256(colors::ERR_HLT),
            ))
        })?;
    Ok(buffer)
}

pub fn delete_file(target: &str) -> Result<(), TombError> {
    fs::remove_file(target)
        .and_then(|_| {
            logger::err::warning(format!(
                "{}{}",
                style("deleted index ").color256(241),
                style(target).color256(246),
            ));
            Ok(())
        })
        .map_err(|error| {
            TombError::with_message(format!(
                "{}{}{}",
                style("deleting ").color256(colors::ERR_MSG),
                style(target).color256(colors::ERR_VAR),
                style(format!("\n\t{}", error)).color256(colors::ERR_HLT),
            ))
        })
}

pub fn delete_directory(target: &str) -> Result<(), TombError> {
    fs::remove_dir_all(target).map_err(|error| {
        TombError::with_message(format!(
            "{}{}{}",
            style("deleting ").color256(colors::ERR_MSG),
            style(target).color256(colors::ERR_VAR),
            style(format!("\n\t{}", error)).color256(colors::ERR_HLT),
        ))
    })
}

pub fn directory_is_empty(target: &str) -> bool {
    let path = Path::new(&target);
    fs::read_dir(&path)
        .map(|entries| entries.count() == 0)
        .unwrap_or(false)
}

pub fn rm_rf(target: &str) -> bool {
    delete_directory(target).is_ok()
}

pub fn append_to_file(filename: &str, value: String) -> Result<(), TombError> {
    let mut file = open_append(filename)?;
    file.write_all(value.as_bytes()).map_err(|error| {
        TombError::with_message(format!("cannot append to file {}: {}", filename, error))
    })
}

pub fn log_to_file(filename: &str, value: String) -> Result<(), TombError> {
    let value = format!("[{}] {}\n", Utc::now(), value);
    append_to_file(filename, value)
}

pub fn write_map_to_yaml(map: &BTreeMap<String, String>, filename: &str) -> Result<(), TombError> {
    let mut file = open_write(filename).unwrap();
    let yaml = serde_yaml::to_string(map).map_err(|error| {
        TombError::with_message(format!(
            "{}{}{}",
            style("failed to serialize data to yaml ").color256(colors::ERR_MSG),
            style(filename).color256(colors::ERR_VAR),
            style(format!("\n\t{}", error)).color256(colors::ERR_HLT),
        ))
    })?;
    file.write_all(yaml.as_bytes()).map_err(|error| {
        TombError::with_message(format!(
            "{}{}{}",
            style("failed to write yaml data to: ").color256(colors::ERR_MSG),
            style(filename).color256(colors::ERR_VAR),
            style(format!("\n\t{}", error)).color256(colors::ERR_HLT),
        ))
    })
}

pub fn get_cwd() -> Result<String, TombError> {
    env::current_dir()
        .map_err(|error| {
            TombError::with_message(format!(
                "{}{}",
                style("failed to retrieve current working directory").color256(colors::ERR_HLT),
                style(format!("\n\t{}", error)).color256(colors::ERR_HLT),
            ))
        })?
        .as_path()
        .canonicalize()
        .map_err(|error| {
            TombError::with_message(format!(
                "{}{}",
                style("failed to calculate absolute path of current working directory")
                    .color256(colors::ERR_MSG),
                style(format!("\n\t{}", error)).color256(colors::ERR_HLT),
            ))
        })?
        .as_os_str()
        .to_str()
        .map(|s| s.to_string())
        .ok_or_else(|| {
            TombError::with_message(format!(
                "{}",
                style("failed convert cwd path to string").color256(colors::ERR_HLT),
            ))
        })
}
