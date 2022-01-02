use crate::ioutils::log_to_file;
use shellexpand;

pub const TOMB_LOG: &'static str = "~/.tomb.log";

pub fn default_log_filename() -> String {
    match std::env::var("TOMB_LOG") {
        Ok(filename) => String::from(shellexpand::tilde(&filename)),
        Err(_error) => String::from(TOMB_LOG),
    }
}

pub fn log_error(message: String) {
    let filename = default_log_filename();
    log_to_file(&filename, message).unwrap_or(())
}
