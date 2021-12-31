pub mod colors {
    pub const TYPE_WARNING: u8 = 214;
    pub const MESG_WARNING: u8 = 246;

    pub const TYPE_OK: u8 = 220;
    pub const MESG_OK: u8 = 222;

    pub const TYPE_ERROR: u8 = 202;
    pub const MESG_ERROR: u8 = 209;

    pub const TYPE_INFO: u8 = 111;
    pub const MESG_INFO: u8 = 153;

    pub const TYPE_SUCCESS: u8 = 148;
    pub const MESG_SUCCESS: u8 = 190;
}

pub mod paint {
    use super::colors;
    use console::style;

    pub fn message(message: String, color: u8) -> String {
        format!("{}", style(format!("{}", message)).color256(color))
    }

    pub fn warning(msg: String) -> String {
        message(msg, colors::TYPE_WARNING)
    }
    pub fn ok(msg: String) -> String {
        message(msg, colors::TYPE_OK)
    }
    pub fn error(msg: String) -> String {
        message(msg, colors::TYPE_ERROR)
    }
    pub fn info(msg: String) -> String {
        message(msg, colors::TYPE_INFO)
    }
    pub fn success(msg: String) -> String {
        message(msg, colors::TYPE_SUCCESS)
    }
}

pub mod format {
    use super::colors;
    use console::style;

    pub fn prefix(prefix: &str, color: u8) -> String {
        format!("{}", style(format!("[{}] ", prefix)).color256(color))
    }
    pub fn message(message: String, color: u8) -> String {
        format!("{}", style(format!("{}", message)).color256(color))
    }
    pub fn prefix_and_message(prf: &str, msg: String, prefix_color: u8, msg_color: u8) -> String {
        format!("{}{}", prefix(prf, prefix_color), message(msg, msg_color),)
    }

    pub fn warning(msg: String) -> String {
        prefix_and_message("W", msg, colors::TYPE_WARNING, colors::MESG_WARNING)
    }
    pub fn ok(msg: String) -> String {
        prefix_and_message("K", msg, colors::TYPE_OK, colors::MESG_OK)
    }
    pub fn error(msg: String) -> String {
        prefix_and_message("E", msg, colors::TYPE_ERROR, colors::MESG_ERROR)
    }
    pub fn info(msg: String) -> String {
        prefix_and_message("I", msg, colors::TYPE_INFO, colors::MESG_INFO)
    }
    pub fn success(msg: String) -> String {
        prefix_and_message("S", msg, colors::TYPE_SUCCESS, colors::MESG_SUCCESS)
    }
}

pub mod out {
    use super::format;

    pub fn warning(msg: String) {
        println!("{}", format::warning(msg));
    }
    pub fn ok(msg: String) {
        println!("{}", format::ok(msg));
    }
    pub fn error(msg: String) {
        println!("{}", format::error(msg));
    }
    pub fn info(msg: String) {
        println!("{}", format::info(msg));
    }
    pub fn success(msg: String) {
        println!("{}", format::success(msg));
    }
}

pub mod err {
    use super::format;

    pub fn warning(msg: String) {
        eprintln!("{}", format::warning(msg));
    }
    pub fn ok(msg: String) {
        eprintln!("{}", format::ok(msg));
    }
    pub fn error(msg: String) {
        eprintln!("{}", format::error(msg));
    }
    pub fn info(msg: String) {
        eprintln!("{}", format::info(msg));
    }
    pub fn success(msg: String) {
        eprintln!("{}", format::success(msg));
    }
}
