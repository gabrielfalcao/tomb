use super::config::TombConfig;
use crate::ioutils::log_to_file;

pub fn log_error(message: String) {
    let filename = TombConfig::load().log_filename;
    log_to_file(&filename, message).unwrap_or(())
}
