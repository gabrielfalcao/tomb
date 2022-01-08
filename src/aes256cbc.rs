/*!
aes-256-cbc module

This library provides user-friendly utilities for performing AES-256-CBC operations.

Currently, supports:

- key derivation with password
- encryption
- decryption

# Example

This example shows how to create a "standard" printer and execute a search.

```
use tomb::aes256cbc::{Key, Config};

let config = Config::from_vec(&[100, 200, 300]);

let password = String::from("I <3 Nickelback");
let key = Key::from_password(&password, &config);

let plaintext = b"Some secret information";
let cyphertext = key.encrypt(plaintext).ok().expect("encryption failed");

let decrypted = key.decrypt(&cyphertext).ok().expect("decryption failed");

assert_eq!((*plaintext).to_vec(), decrypted);
```
*/

extern crate rand;

use crate::{
    colors,
    config::{YamlFile, YamlFileError},
    ioutils::{b64decode, b64encode},
    logger,
};

use console::style;
use crypto::buffer::{BufferResult, ReadBuffer, WriteBuffer};
use crypto::hmac::Hmac;
use crypto::mac::Mac;
use crypto::sha2::Sha256;
use crypto::{aes, blockmodes, buffer, pbkdf2};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use shellexpand;

use std::borrow::Borrow;
use std::io::Read;
use std::{fmt, fs::File};

pub const ALGO: &'static str = "aes-256-cbc";
pub const DIGEST_SIZE: usize = 32;

///The path used by `Key::default()` and `Config::default()`
pub const TOMB_KEY: &'static str = "~/.tomb.key";

pub fn default_key_filename() -> String {
    match std::env::var("TOMB_KEY") {
        Ok(filename) => String::from(shellexpand::tilde(&filename)),
        Err(_err) => String::from(TOMB_KEY),
    }
}

///The builtin number of cycles for a key derivation
pub const KEY_CYCLES: u32 = 16000;
///The builtin number of cycles for a salt derivation
pub const SALT_CYCLES: u32 = 16000;
///The builtin number of cycles for an iv derivation
pub const IV_CYCLES: u32 = 16000;

pub const KEY_SIZE: usize = 256;
pub const IV_SIZE: usize = 16;
pub const BLOCK_SIZE: usize = 4096;

pub type Digest = [u8; DIGEST_SIZE];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Error {
    pub message: String,
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}
impl YamlFileError for Error {
    fn with_message(message: String) -> Error {
        Error {
            message: logger::paint::error(format!("{}", message)),
        }
    }
}

pub fn bytes_match(a: &[u8], b: &[u8]) -> bool {
    let diff = a
        .iter()
        .zip(b.iter())
        .map(|(a, b)| a ^ b)
        .fold(0, |acc, x| acc | x);
    diff == 0 && a.len() == b.len()
}
/// Dummy example of hmac_256_digest
pub fn hmac_256_digest(mac_key: &[u8], iv: &[u8]) -> Result<Digest, Error> {
    let mut mac = Hmac::new(Sha256::new(), &mac_key);
    mac.input(&iv);
    let result = mac.result();
    let mac_digest = result.code();
    Ok(match mac_digest[..DIGEST_SIZE].try_into() {
        Ok(digest) => digest,
        Err(err) => {
            return Err(Error::with_message(format!(
                "failed to convert digest into [u8] {}",
                err
            )))
        }
    })
}

/// Generates a random KEY;
pub fn generate_key() -> [u8; KEY_SIZE] {
    let mut rng = rand::thread_rng();
    let mut key: [u8; KEY_SIZE] = [0; KEY_SIZE];
    rng.fill_bytes(&mut key);
    key
}
/// Generates a random IV;
pub fn generate_iv() -> [u8; IV_SIZE] {
    let mut rng = rand::thread_rng();
    let mut iv: [u8; IV_SIZE] = [0; IV_SIZE];
    rng.fill_bytes(&mut iv);
    iv
}

#[derive(PartialEq, Clone, Copy, Serialize, Deserialize)]
pub struct CyclesConfig {
    pub key: u32,
    pub salt: u32,
    pub iv: u32,
}
impl CyclesConfig {
    pub fn to_vec(&self) -> Vec<u32> {
        let mut cycles: Vec<u32> = Vec::new();
        cycles.push(self.key);
        cycles.push(self.salt);
        cycles.push(self.iv);
        cycles
    }
    pub fn from_vec(vec: &[u32; 3]) -> CyclesConfig {
        CyclesConfig {
            key: vec[0],
            salt: vec[1],
            iv: vec[2],
        }
    }
}

/// The configuration for the Key.
///
/// It contains the cycles for key, salt and iv used in key derivation.
#[derive(PartialEq, Clone, Serialize, Deserialize)]
pub struct Config {
    pub cycles: CyclesConfig,
    pub default_key_path: Option<String>,
}

impl YamlFile<Error> for Config {
    fn default() -> Result<Config, Error> {
        Ok(Config::builtin(Some(default_key_filename().to_string())))
    }
}
impl Config {
    /// Creates a new config based on a &Vec<u32>
    pub fn from_vec(vec: &[u32; 3]) -> Config {
        Config {
            cycles: CyclesConfig::from_vec(vec),
            default_key_path: None,
        }
    }
    /// Creates a new builtin config
    pub fn builtin(default_key_path: Option<String>) -> Config {
        Config {
            default_key_path,
            cycles: CyclesConfig {
                key: KEY_CYCLES,
                salt: SALT_CYCLES,
                iv: IV_CYCLES,
            },
        }
    }

    pub fn iv_cycles(&self) -> u32 {
        self.cycles.iv
    }

    pub fn key_cycles(&self) -> u32 {
        self.cycles.key
    }

    pub fn salt_cycles(&self) -> u32 {
        self.cycles.salt
    }

    pub fn derive_key(&self, password: &str, salt: &[u8]) -> [u8; KEY_SIZE] {
        let mut dk = [0u8; KEY_SIZE]; // derived key
        let mut mac = Hmac::new(Sha256::new(), password.as_bytes());
        pbkdf2::pbkdf2(&mut mac, salt, self.key_cycles(), &mut dk);
        dk
    }

    pub fn derive_salt(&self, password: &str) -> [u8; KEY_SIZE] {
        let mut dk = [0u8; KEY_SIZE]; // derived key
        let mut mac = Hmac::new(Sha256::new(), password.as_bytes());
        pbkdf2::pbkdf2(&mut mac, password.as_bytes(), self.salt_cycles(), &mut dk);
        dk
    }

    pub fn derive_iv(&self, password: &str) -> [u8; IV_SIZE] {
        let mut dk = [0u8; IV_SIZE]; // derived iv
        let mut mac = Hmac::new(Sha256::new(), password.as_bytes());
        pbkdf2::pbkdf2(&mut mac, password.as_bytes(), self.iv_cycles(), &mut dk);
        dk
    }
}

/// AES-256 Key data
#[derive(PartialEq, Serialize, Deserialize, Clone)]
pub struct Key {
    pub algo: String,
    pub key: String,
    pub mac: String,
    pub iv: String,
    pub magic: Option<Vec<u32>>,
}

impl YamlFile<Error> for Key {
    fn default() -> Result<Key, Error> {
        let filename = default_key_filename();
        Key::import(filename.borrow())
    }
}

impl Key {
    /// Derive a key from a password using the cycles from the given config
    pub fn from_password(password: &str, config: &Config) -> Key {
        let iv = config.derive_iv(password);
        let salt = config.derive_salt(password);
        //let salt = generate_iv();
        let key_material = config.derive_key(password, &salt);

        let enc_key = &key_material[0..127];
        let mac_key = &key_material[128..255];

        Key {
            key: b64encode(&enc_key),
            mac: b64encode(&mac_key),
            iv: b64encode(&iv),
            algo: String::from(ALGO),
            magic: Some(config.cycles.to_vec()),
        }
    }
    /// Generate a new key
    pub fn generate() -> Key {
        let iv = generate_iv();
        let key_material = generate_key();
        let enc_key = &key_material[0..127];
        let mac_key = &key_material[128..255];

        Key {
            key: b64encode(&enc_key),
            mac: b64encode(&mac_key),
            iv: b64encode(&iv),
            algo: String::from(ALGO),
            magic: None,
        }
    }
    /// Checks if a file is encrypted with this key
    pub fn owns_file(&self, filename: &str) -> Result<bool, Error> {
        let mut fd =
            File::open(filename).expect(format!("failed to open file {}", filename).as_str());
        let mut buffer = [0; DIGEST_SIZE];
        match fd.read(&mut buffer) {
            Ok(_) => {}
            Err(error) => {
                return Err(Error::with_message(format!(
                    "{}{}{}",
                    style("reading the first {:?} bytes from file ").color256(colors::ERR_MSG),
                    style(filename).color256(colors::ERR_VAR),
                    style(format!("\n\t{}", error)).color256(colors::ERR_HLT),
                )))
            }
        };

        Ok(self.check_digest(&buffer))
    }
    /// Checks the digest of the given bytes
    pub fn check_digest(&self, buffer: &Digest) -> bool {
        let digest = self.digest();
        bytes_match(buffer, &digest)
    }
    pub fn digest(&self) -> Digest {
        let mac = self.mac_bytes().unwrap();
        let iv = self.iv_bytes().unwrap();
        hmac_256_digest(&mac, &iv).unwrap()
    }
    pub fn iv_bytes(&self) -> Result<Vec<u8>, Error> {
        match b64decode(self.iv.as_bytes()) {
            Ok(v) => Ok(v),
            Err(e) => Err(Error::with_message(format!("parse base64 iv: {}", e))),
        }
    }
    pub fn key_bytes(&self) -> Result<Vec<u8>, Error> {
        match b64decode(self.key.as_bytes()) {
            Ok(v) => Ok(v),
            Err(e) => Err(Error::with_message(format!("parse base64 key: {}", e))),
        }
    }
    pub fn mac_bytes(&self) -> Result<Vec<u8>, Error> {
        match b64decode(self.mac.as_bytes()) {
            Ok(v) => Ok(v),
            Err(e) => Err(Error::with_message(format!("parse base64 mac: {}", e))),
        }
    }

    /// Encrypt a buffer with the key
    /// AES-256/CBC/Pkcs encryption.
    pub fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let enc_key = self.key_bytes().unwrap();
        let iv = self.iv_bytes().unwrap();
        let mut encryptor = aes::cbc_encryptor(
            aes::KeySize::KeySize256,
            &enc_key,
            &iv,
            blockmodes::PkcsPadding,
        );

        let mut cyphertext = Vec::<u8>::new();
        let mut read_buffer = buffer::RefReadBuffer::new(data);
        let mut buffer = [0; BLOCK_SIZE];
        let mut write_buffer = buffer::RefWriteBuffer::new(&mut buffer);

        // The first 32 bytes of the cyphertext are always the digest,
        // this allows for checking if a file was encrypted with the
        // correct key and take the appropriate action.
        let digest = self.digest();
        cyphertext.extend_from_slice(&digest);

        loop {
            let result = match encryptor.encrypt(&mut read_buffer, &mut write_buffer, true) {
                Ok(result) => result,
                Err(error) => {
                    return Err(Error::with_message(format!(
                        "failed to encrypt data: {:?}",
                        error
                    )))
                }
            };
            cyphertext.extend(
                write_buffer
                    .take_read_buffer()
                    .take_remaining()
                    .iter()
                    .map(|&i| i),
            );

            match result {
                BufferResult::BufferUnderflow => break,
                BufferResult::BufferOverflow => {}
            }
        }
        Ok(cyphertext)
    }

    /// Decrypts a buffer with the key
    /// AES-256/CBC/Pkcs decryption.
    pub fn decrypt(&self, cyphertext: &[u8]) -> Result<Vec<u8>, Error> {
        let mut decryptor = aes::cbc_decryptor(
            aes::KeySize::KeySize256,
            &match self.key_bytes() {
                Ok(bytes) => bytes,
                Err(e) => return Err(e),
            },
            &match self.iv_bytes() {
                Ok(bytes) => bytes,
                Err(e) => return Err(e),
            },
            blockmodes::PkcsPadding,
        );

        let mut plaintext = Vec::<u8>::new();
        let hmac_bytes: Digest = match cyphertext[..DIGEST_SIZE].try_into() {
            Ok(digest) => digest,
            Err(error) => {
                return Err(Error::with_message(format!(
                    "failed to convert digest to u8: {}",
                    error
                )))
            }
        };
        if !self.check_digest(&hmac_bytes) {
            return Err(Error::with_message(format!(
                "Cannot decrypt: data was not encrypted with the provided key. Leaving file as is."
            )));
        }

        let ciphertext = &cyphertext[DIGEST_SIZE..];
        let mut read_buffer = buffer::RefReadBuffer::new(&ciphertext);
        let mut buffer = [0; BLOCK_SIZE];
        let mut write_buffer = buffer::RefWriteBuffer::new(&mut buffer);

        loop {
            let result = match decryptor.decrypt(&mut read_buffer, &mut write_buffer, true) {
                Ok(result) => result,
                Err(error) => {
                    return Err(Error::with_message(format!(
                        "cannot decrypt data: {:?}",
                        error
                    )))
                }
            };

            plaintext.extend(
                write_buffer
                    .take_read_buffer()
                    .take_remaining()
                    .iter()
                    .map(|&i| i),
            );
            match result {
                BufferResult::BufferUnderflow => break,
                BufferResult::BufferOverflow => {}
            }
        }

        Ok(plaintext)
    }
}
#[cfg(test)]
mod tests {
    use crate::aes256cbc::Config;
    use crate::aes256cbc::Key;
    use k9::assert_equal;

    #[test]
    fn test_encrypt_and_decrypt() {
        let config = Config::builtin(None);
        let password = "123456";
        let key = Key::from_password(password, &config);

        let plaintext = b"This is a secret";
        let ciphertext = key.encrypt(plaintext).unwrap();

        let decrypted = key.decrypt(&ciphertext).unwrap();
        assert_equal!(decrypted, b"This is a secret");
    }
}
