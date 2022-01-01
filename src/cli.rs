extern crate clap;
use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use clipboard::ClipboardContext;
use clipboard::ClipboardProvider;
use console::style;
#[cfg(feature = "osx")]
use mac_notification_sys::*;
use std::path::Path;
//use console::style;
use std::panic;
use tomb::{
    aes256cbc::{default_key_filename, Config as AesConfig, Key},
    app::{self, TombConfig},
    config::YamlFile,
    logger,
    tomb::{default_tomb_filename, AES256Tomb},
};

pub fn confirm_password() -> Option<String> {
    let password = rpassword::prompt_password_stderr("Password: ").unwrap();
    let confirmation = rpassword::prompt_password_stderr("Confirm password: ").unwrap();

    if password != confirmation {
        logger::err::error(format!(
            "{}",
            style("Password/Confirmation mismatch").color256(202)
        ));
        None
    } else {
        Some(password)
    }
}
fn get_password_from_matches(matches: &ArgMatches) -> String {
    let ask_password = matches.is_present("ask_password");
    let password = if ask_password {
        match confirm_password() {
            Some(password) => {
                logger::out::ok(format!("confirmed password: {}", password));
                password
            }
            None => String::from(matches.value_of("password").unwrap_or("")),
        }
    } else {
        String::from(matches.value_of("password").unwrap_or(""))
    };
    password
}

fn load_key(matches: &ArgMatches) -> Key {
    let config = AesConfig::default().unwrap();
    let password = get_password_from_matches(matches);
    let key_filename = matches.value_of("key_filename").unwrap_or("");

    if key_filename.len() > 0 {
        Key::import(key_filename).unwrap()
    } else if password.len() > 0 {
        Key::from_password(&password.as_bytes(), &config)
    } else {
        logger::err::error(format!(
            "{}{}{}{}{}",
            style("either").color256(195),
            style("--password, --key-filename").color256(190),
            style(" or ").color256(195),
            style("--ask-password").color256(190),
            style(" is required").color256(195),
        ));
        std::process::exit(1);
    }
}

fn load_tomb(matches: &ArgMatches) -> AES256Tomb {
    let tomb_filepath = matches.value_of("tomb_filename").unwrap();
    match AES256Tomb::import(tomb_filepath) {
        Ok(tomb) => tomb.with_filepath(tomb_filepath),
        Err(err) => {
            eprintln!("{}", err);
            std::process::exit(1);
        }
    }
}

fn init_command(matches: &ArgMatches) {
    let mut tomb_config = TombConfig::load();
    tomb_config.save().unwrap();
    let ask_password = matches.is_present("ask_password");
    let key_cycles = matches
        .value_of("key_cycles")
        .unwrap_or("")
        .parse::<u32>()
        .unwrap_or(1000);
    let salt_cycles = matches
        .value_of("salt_cycles")
        .unwrap_or("")
        .parse::<u32>()
        .unwrap_or(1000);
    let iv_cycles = matches
        .value_of("iv_cycles")
        .unwrap_or("")
        .parse::<u32>()
        .unwrap_or(1000);

    let tomb_filepath = matches.value_of("tomb_filename").unwrap();
    let key_filename = matches.value_of("key_filename").unwrap();

    let key = if !Path::new(key_filename).exists() {
        let vec: [u32; 3] = [key_cycles, salt_cycles, iv_cycles];
        let custom_config = AesConfig::from_vec(&vec);
        let password = if ask_password {
            match confirm_password() {
                Some(password) => password,
                None => String::from(matches.value_of("password").unwrap_or("")),
            }
        } else {
            String::from(matches.value_of("password").unwrap_or(""))
        };
        logger::err::info(format!("deriving key from password, please be patient..."));
        let key = Key::from_password(&password.as_bytes(), &custom_config);

        let key_path = match key.export(key_filename) {
            Ok(path) => path,
            Err(error) => {
                eprintln!("{}", error);
                std::process::exit(1);
            }
        };
        logger::err::ok(format!("generated key: {}", style(key_path).color256(214)));
        key
    } else {
        Key::import(key_filename).unwrap()
    };

    let config = AesConfig::default().unwrap();

    if AES256Tomb::import(tomb_filepath).is_ok() {
        logger::err::warning(format!("file already exists: {}", tomb_filepath));
        std::process::exit(0);
    }
    let mut tomb = AES256Tomb::new(tomb_filepath, key.clone(), config.clone());
    match tomb.save() {
        Ok(target) => {
            logger::out::ok(format!("initialized tomb file: {}", target));
        }
        Err(err) => {
            logger::err::error(format!("failed to save tomb file - {}", err));
            std::process::exit(1);
        }
    };
}

fn save_command(matches: &ArgMatches) {
    let path = matches.value_of("path").expect("missing key path");
    let value = matches.value_of("value").expect("missing value");
    let key = load_key(matches);
    let mut tomb = load_tomb(matches);
    match tomb.add_secret(path, String::from(value), key) {
        Ok(_) => {
            match tomb.save() {
                Ok(_) => {
                    logger::out::ok(format!("added secret: {}", path));
                }
                Err(err) => {
                    eprintln!("{}", err);
                    std::process::exit(1);
                }
            };
        }
        Err(err) => {
            eprintln!("{}", err);
            std::process::exit(1);
        }
    }
}
fn get_command(matches: &ArgMatches) {
    let path = matches.value_of("path").expect("missing key path");
    let key = load_key(matches);
    let tomb = load_tomb(matches);
    match tomb.get_string(path, key) {
        Ok(plaintext) => {
            println!("{}", plaintext)
        }
        Err(err) => {
            eprintln!("{}", err);
            std::process::exit(1);
        }
    }
}
fn copy_command(matches: &ArgMatches) {
    let path = matches.value_of("path").expect("missing key path");
    let sound = matches.value_of("sound").unwrap_or("Glass");
    let key = load_key(matches);
    let tomb = load_tomb(matches);
    match tomb.get_string(path, key) {
        Ok(plaintext) => {
            let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
            ctx.set_contents(plaintext).unwrap();
            eprintln!("{} secret copied to clipboard 🎉", path);

            #[cfg(feature = "osx")]
            send_notification(
                format!("{} secret", path).as_str(),
                &Some("copied to clipboard 🎉"),
                "",
                &Some(sound),
            )
            .unwrap();
        }
        Err(err) => {
            eprintln!("{}", err);
            std::process::exit(1);
        }
    }
}
fn delete_command(matches: &ArgMatches) {
    let path = matches.value_of("path").expect("missing key path");
    //let key = load_key(matches);
    let mut tomb = load_tomb(matches);
    match tomb.delete_secret(path) {
        Ok(_) => {
            match tomb.save() {
                Ok(_) => {
                    logger::out::ok(format!("deleted secret: {}", path));
                }
                Err(err) => {
                    eprintln!("{}", err);
                    std::process::exit(1);
                }
            };
        }
        Err(err) => {
            eprintln!("{}", err);
            std::process::exit(1);
        }
    }
}
fn list_command(matches: &ArgMatches) {
    let pattern = matches.value_of("pattern").expect("missing key pattern");
    // let key = load_key(matches);
    let tomb = load_tomb(matches);
    match tomb.list(pattern) {
        Ok(secrets) => {
            for entry in secrets {
                println!("{}", entry.path)
            }
        }
        Err(err) => {
            eprintln!("{}", err);
            std::process::exit(1);
        }
    }
}
fn ui_command(matches: &ArgMatches) {
    let key = load_key(matches);
    let mut tomb = load_tomb(matches);
    match tomb.save() {
        Ok(target) => {
            logger::out::ok(format!("saved file: {}", target));
        }
        Err(err) => {
            logger::err::error(format!("failed to save tomb file - {}", err));
            std::process::exit(1);
        }
    };

    let tomb_config = TombConfig::load();
    let aes_config = AesConfig::default().unwrap_or(AesConfig::builtin(None));
    let tick_interval = matches.value_of("tick_interval").unwrap_or("314");
    let tick_interval = match tick_interval.parse::<u64>() {
        Ok(tick_interval) => tick_interval,
        Err(err) => {
            logger::err::error(format!(
                "tick interval is not a valid number {:?}: {}",
                tick_interval, err
            ));
            std::process::exit(1);
        }
    };
    match app::start(tomb, key, tomb_config, aes_config, tick_interval) {
        Ok(()) => {}
        Err(error) => {
            eprintln!("{}", error);
            std::process::exit(1);
        }
    }
}

fn main() {
    panic::set_hook(Box::new(|e| {
        eprintln!("{}", e);
    }));

    let tomb_filename = default_tomb_filename();
    let key_filename = default_key_filename();
    let app = App::new("⚰Tomb")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .about("Password Manager")
        .subcommand(
            SubCommand::with_name("save")
                .about("store a secret in the tomb")
                .arg(
                    Arg::with_name("key_filename")
                        .long("key-filename")
                        .help("the path to the aes256cbc key to encrypt the tomb secrets")
                        .short("k")
                        .default_value(&key_filename)
                        .required(true)
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("tomb_filename")
                        .long("tomb")
                        .short("t")
                        .value_name("FILENAME")
                        .default_value(&tomb_filename)
                        .help("the path to the tomb file containing the encrypted secrets")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("path")
                        .value_name("KEY PATH")
                        .help("the path to the secret")
                        .required(true)
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("value")
                        .value_name("VALUE")
                        .required(true)
                        .help("the secret value to be saved")
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("init")
                .about("initializes a tomb file and generates a key")
                .arg(
                    Arg::with_name("key_filename")
                        .long("key-filename")
                        .short("k")
                        .default_value(&key_filename)
                        .required(true)
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("password")
                        .long("password")
                        .short("P")
                        .required_unless_one(&["key_filename", "ask_password"])
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("ask_password")
                        .long("ask-password")
                        .short("p")
                        .required_unless_one(&["password", "ask_password"])
                        .takes_value(false),
                )
                .arg(
                    Arg::with_name("key_cycles")
                        .default_value("16000")
                        .long("key")
                        .short("K")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("salt_cycles")
                        .default_value("16000")
                        .long("salt")
                        .short("S")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("iv_cycles")
                        .default_value("16000")
                        .long("iv")
                        .short("I")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("tomb_filename")
                        .long("tomb")
                        .short("t")
                        .value_name("FILENAME")
                        .default_value(&tomb_filename)
                        .help("the path to the tomb file containing the encrypted secrets")
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("ui")
                .about("open the terminal ui")
                .arg(
                    Arg::with_name("key_filename")
                        .long("key-filename")
                        .help("the path to the aes256cbc key to encrypt the tomb secrets")
                        .short("k")
                        .default_value(&key_filename)
                        .required(true)
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("tick_interval")
                        .long("--tick-interval")
                        .help("the duration of each internal tick in milliseconds")
                        .short("T")
                        .default_value("314")
                        .required(false)
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("tomb_filename")
                        .long("tomb")
                        .short("t")
                        .value_name("FILENAME")
                        .default_value(&tomb_filename)
                        .help("the path to the tomb file containing the encrypted secrets")
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("get")
                .about("get a secret")
                .arg(
                    Arg::with_name("key_filename")
                        .long("key-filename")
                        .help("the path to the aes256cbc key to encrypt the tomb secrets")
                        .short("k")
                        .default_value(&key_filename)
                        .required(true)
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("tomb_filename")
                        .long("tomb")
                        .short("t")
                        .value_name("FILENAME")
                        .default_value(&tomb_filename)
                        .help("the path to the tomb file containing the encrypted secrets")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("path")
                        .value_name("KEY PATH")
                        .help("the path to the secret")
                        .required(true)
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("copy")
                .about("copy a secret to the clipboard")
                .arg(
                    Arg::with_name("key_filename")
                        .long("key-filename")
                        .help("the path to the aes256cbc key to encrypt the tomb secrets")
                        .short("k")
                        .default_value(&key_filename)
                        .required(true)
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("sound")
                        .long("sound")
                        .help("name of sound to play (MacOS-only)")
                        .short("S")
                        .default_value("Glass")
                        .required(false)
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("tomb_filename")
                        .long("tomb")
                        .short("t")
                        .value_name("FILENAME")
                        .default_value(&tomb_filename)
                        .help("the path to the tomb file containing the encrypted secrets")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("path")
                        .value_name("KEY PATH")
                        .help("the path to the secret")
                        .required(true)
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("delete")
                .about("delete a secret")
                .arg(
                    Arg::with_name("key_filename")
                        .long("key-filename")
                        .help("the path to the aes256cbc key to encrypt the tomb secrets")
                        .short("k")
                        .default_value(&key_filename)
                        .required(false)
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("tomb_filename")
                        .long("tomb")
                        .short("t")
                        .value_name("FILENAME")
                        .default_value(&tomb_filename)
                        .help("the path to the tomb file containing the encrypted secrets")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("path")
                        .value_name("KEY PATH")
                        .help("the path to the secret")
                        .required(true)
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("list")
                .about("list secrets")
                .arg(
                    Arg::with_name("key_filename")
                        .long("key-filename")
                        .help("the path to the aes256cbc key to encrypt the tomb secrets")
                        .short("k")
                        .default_value(&key_filename)
                        .required(false)
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("tomb_filename")
                        .long("tomb")
                        .short("t")
                        .value_name("FILENAME")
                        .default_value(&tomb_filename)
                        .help("the path to the tomb file containing the encrypted secrets")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("pattern")
                        .value_name("PATTERN")
                        .help("the path to the secret")
                        .default_value("*")
                        .takes_value(true),
                ),
        );

    let matches = app.get_matches();

    match matches.subcommand() {
        ("init", Some(matches)) => {
            init_command(&matches);
        }
        ("save", Some(matches)) => {
            save_command(&matches);
        }
        ("get", Some(matches)) => {
            get_command(&matches);
        }
        ("copy", Some(matches)) => {
            copy_command(&matches);
        }
        ("delete", Some(matches)) => {
            delete_command(&matches);
        }
        ("list", Some(matches)) => {
            list_command(&matches);
        }
        ("ui", Some(matches)) => {
            ui_command(&matches);
        }
        (cmd, Some(_matches)) => {
            eprintln!("command not implemented: {}", cmd);
        }
        (cmd, None) => {
            eprintln!("unhandled command: {}", cmd);
        }
    }
}
