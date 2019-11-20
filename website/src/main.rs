mod cmd_opt;
mod config;
mod secret_backend;

use cmd_opt::Opt;
use config::{Config, Truiloo};
use lox::prelude::*;
use secret_backend::SecretBackend;
use std::fs::File;
use std::io::Read;
use std::process::exit;
use structopt::StructOpt;

const TRUILOO_SERVICE: &str = "truiloo";

fn main() {
    let opt = Opt::from_args();
    let config = get_config(&opt);
    let truilooapikey = get_truiloo_secret(&config);


}

fn get_truiloo_secret(config: &Config) -> Vec<u8> {
    let truiloo = config.truiloo.as_ref().unwrap();
    let mut apikey = Vec::new();
    if let Some(name) = &truiloo.name {
        if let Some(backend) = config.secret_backend {
            match backend {
                SecretBackend::OsKeyRing => {
                    let mut keyring = get_os_keyring(TRUILOO_SERVICE).unwrap();
                    apikey = keyring.get_secret(name.as_str()).unwrap().as_slice().to_vec();
                },
                _ => {
                    eprintln!("{} not handled", backend);
                    exit(1);
                }
            }
        } else {
            eprintln!("truiloo name cannot be used without a secret backend");
            exit(1);
        }
    } else if let Some(value) = &truiloo.value {
        apikey = value.as_bytes().to_vec();
    }
    apikey
}

fn get_config(opt: &Opt) -> Config {
    let mut config: Config;
    match &opt.config {
        Some(c) => {
            if !c.exists() || !c.is_file() {
                eprintln!("The config file does not exist: '{:?}'", c);
                exit(1);
            }
            match File::open(c) {
                Ok(mut f) => {
                    let mut contents = String::new();
                    match f.read_to_string(&mut contents) {
                        Ok(_) => {
                            match toml::from_str(contents.as_str()) {
                                Ok(g) => config = g,
                                Err(e) => {
                                    eprintln!("An error occurred while parsing '{:?}': {}", c, e);
                                    exit(1);
                                }
                            }
                        },
                        Err(e) => {
                            eprintln!("An error occurred while reading '{:?}': {}", c, e);
                            exit(1);
                        }
                    }
                },
                Err(e) => {
                    eprintln!("An error occurred while opening '{:?}': {}", c, e);
                    exit(1);
                }
            }
        },
        None => config = opt.into()
    };
    loop {
        if config.truiloo.is_none() {
            match rpassword::read_password_from_tty(Some("Enter x-truiloo-api-key:  ")) {
                Ok(v) => {
                    if v.len() > 0 {
                        config.truiloo = Some(Truiloo {
                            name: None,
                            value: Some(v)
                        });
                        break;
                    } else {
                        eprintln!("x-truiloo-api-key cannot be empty.");
                    }
                },
                Err(e) => {
                    eprintln!("An error occurred while reading api key: {}", e);
                    exit(1);
                }
            };
        } else {
            break;
        }
    }
    config
}
