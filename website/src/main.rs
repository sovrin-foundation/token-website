#![feature(proc_macro_hygiene,
           decl_macro)]
//#![deny(warnings,
//        unused_import_braces,
//        unused_qualifications,
//        trivial_casts,
//        trivial_numeric_casts)]
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate lazy_static;

mod cmd_opt;
mod config;
mod secret_backend;
mod consents;

use celes::Country;
use cmd_opt::Opt;
use config::{Config, Trulioo};
use lox::prelude::*;
use rocket::{
    response::status,
    State
};
use rocket_contrib::{
    helmet::SpaceHelmet,
    serve::StaticFiles,
};
use secret_backend::SecretBackend;
use serde::Serialize;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::Read;
use std::process::exit;
use std::str::FromStr;
use structopt::StructOpt;
use trulioo::TruliooRequest;

const TOKEN_WEBSITE_SERVICE: &str = "token_website";
const TRULIOO_SERVICE: &str = "trulioo";

#[get("/countries")]
pub(crate) fn get_allowed_countries(countries: State<BTreeMap<String, Country>>) -> String {
    #[derive(Serialize)]
    struct SimpleCountry {
        value: usize,
        long_name: String
    };
    let list = countries.inner().iter().map(|(_, c)| SimpleCountry { value: c.value, long_name: c.long_name.to_string() }).collect::<Vec<SimpleCountry>>();
    serde_json::to_string(&list).unwrap()
}

#[get("/consents/<country_value>")]
pub(crate) fn get_consents(country_value: usize, request: State<TruliooRequest>) -> String {
    let country;

    match Country::from_value(country_value) {
        Ok(c) => country = c,
        Err(e) => return format!("{{ error: {} }}", e)
    };

    let mut consents = Ok(Vec::new());
    async_std::task::block_on(async {
       consents = request.inner().get_detailed_consents(country.alpha2).await
    });
    match consents {
        Ok(c) => serde_json::to_string(&c).unwrap(),
        Err(e) => format!("{{ error: {} }}", e)
    }
}

fn main() {
    let opt = Opt::from_args();
    let config = get_config(&opt);

    let (url, key);
    if let Some(ref t) = config.trulioo {
        url = t.url.clone();
        if let Some(key_name) = &t.key_name {
            key = std::str::from_utf8(&get_trulioo_secret(&key_name, config.secret_backend)).unwrap().to_string();
        } else if let Some(key_value) = &t.key_value {
            key = t.key_value.clone().unwrap();
        } else {
            key = prompt_for_value(trulioo::API_KEY_HEADER);
        }
    } else {
        url = prompt_for_value("trulioo api url");
        key = prompt_for_value(trulioo::API_KEY_HEADER);
    }

    let request = TruliooRequest { key, url };
    let mut countries = BTreeMap::new();

    async_std::task::block_on(async {
        let codes = request.get_country_codes().await.unwrap();
        for code in codes {
            if let Ok(c) = Country::from_str(&code) {
                countries.insert(c.alpha2.to_string(), c);
            }
        }
    });
    rocket::ignite()
            .attach(SpaceHelmet::default())
            .manage(request)
            .manage(countries)
            .mount("/", StaticFiles::from("/public"))
            .mount("/api/v1", routes![get_allowed_countries, get_consents]).launch();
}

fn get_trulioo_secret(key_name: &str, secret_backend: Option<SecretBackend>) -> Vec<u8> {
    let mut apikey = Vec::new();
    if let Some(backend) = secret_backend {
        match backend {
            SecretBackend::OsKeyRing => {
                let mut keyring = get_os_keyring(TRULIOO_SERVICE).unwrap();
                apikey = keyring.get_secret(key_name).unwrap().as_slice().to_vec();
            },
            _ => {
                eprintln!("{} not handled", backend);
                exit(1);
            }
        }
    } else {
        eprintln!("trulioo name cannot be used without a secret backend");
        exit(1);
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

    config
}

fn prompt_for_value(value_name: &str) -> String {
    loop {
        match rpassword::read_password_from_tty(Some(format!("Enter {}:  ", value_name).as_str())) {
            Ok(v) => {
                if v.len() > 0 {
                    return v;
                } else {
                    eprintln!("{} cannot be empty.", value_name);
                }
            },
            Err(e) => {
                eprintln ! ("An error occurred while reading {}: {}", value_name, e);
                exit(1);
            }
        };
    }
}
