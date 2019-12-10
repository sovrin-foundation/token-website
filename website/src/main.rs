#![feature(proc_macro_hygiene,
           decl_macro)]
//#![deny(warnings,
//        unused_import_braces,
//        unused_qualifications,
//        trivial_casts,
//        trivial_numeric_casts)]
#[macro_use]
extern crate arrayref;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate rocket;

mod cmd_opt;
mod config;
mod secret_backend;
mod consents;
mod responses;

use celes::Country;
use cmd_opt::Opt;
use config::Config;
use ed25519_dalek::{Signature, PublicKey};
use hmac::{Hmac, Mac};
use lox::prelude::*;
use rand::RngCore;
use rocket::{
    State
};
use rocket_contrib::{
    helmet::SpaceHelmet,
    json::Json,
    serve::StaticFiles,
};
use secret_backend::SecretBackend;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::{
    collections::BTreeMap,
    error::Error,
    fs,
    io::Write,
    path::PathBuf,
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH}
};
use structopt::StructOpt;
use subtle::ConstantTimeEq;
use trulioo::TruliooRequest;

const TOKEN_WEBSITE_SERVICE: &str = "token_website";
const TRULIOO_SERVICE: &str = "trulioo";

type HmacSha256 = Hmac<Sha256>;

#[get("/countries")]
pub(crate) fn get_allowed_countries(countries: State<BTreeMap<String, Country>>) -> String {
    #[derive(Serialize)]
    struct SimpleCountry {
        alpha2: String,
        long_name: String
    };
    let list = countries.inner().iter().map(|(_, c)| SimpleCountry { alpha2: c.alpha2.to_string(), long_name: c.long_name.to_string() }).collect::<Vec<SimpleCountry>>();
    format!(r#"{{ "status": "success", "result": {} }}"#, serde_json::to_string(&list).unwrap())
}

#[get("/consents/<country>")]
pub(crate) fn get_consents(country: String, request: State<TruliooRequest>, countries: State<BTreeMap<String, Country>>) -> String {
    if !countries.inner().contains_key(&country) {
        return format!(r#"{{ "status": "error", "message": {} }}"#, "Invalid country code");
    }

    let mut consents = Ok(Vec::new());
    async_std::task::block_on(async {
       consents = request.inner().get_detailed_consents(country).await
    });
    match consents {
        Ok(c) => format!(r#"{{ "status": "success", "result": {} }}"#, serde_json::to_string(&c).unwrap()),
        Err(e) => format!(r#"{{ "status": "error", "message": {} }}"#, e)
    }
}

#[get("/payment_address_challenge")]
pub(crate) fn get_payment_address_challenge(challenge_signing_key: State<Vec<u8>>) -> String {
    let mut rng = rand::rngs::OsRng{};
    let mut result = generate_timestamp().unwrap().to_be_bytes().to_vec();
    let mut challenge = vec![0u8; 32];
    rng.fill_bytes(challenge.as_mut_slice());

    let mut hmac = HmacSha256::new_varkey(&challenge_signing_key.inner().as_slice()).unwrap();
    hmac.input(result.as_slice());
    hmac.input(challenge.as_slice());
    let hash = hmac.result().code();

    result.extend_from_slice(challenge.as_slice());
    result.extend_from_slice(hash.as_slice());

    format!(r#"{{ "status": "success", "result": "{}" }}"#, base64_url::encode(result.as_slice()))
}

#[post("/payment_address_challenge", format = "application/json", data = "<challenge>")]
pub(crate) fn receive_payment_address_challenge(challenge: Json<responses::PaymentAddressChallengeResponse>, challenge_signing_key: State<Vec<u8>>) -> String {
    const TIMESTAMP: usize = 8;
    const NONCE: usize = 32;
    const EXPIRE: u64 = 3600;
    let response = challenge.into_inner();

    let challenge = match base64_url::decode(&response.challenge) {
        Err(why) => return format!(r#"{{ "status": "error", "message": {} }}"#, why.description()),
        Ok(c) => c,
    };

    let signature = match base64_url::decode(&response.signature) {
        Err(why) => return format!(r#"{{ "status": "error", "message": {} }}"#, why.description()),
        Ok(s) => s,
    };

    let timestamp = u64::from_be_bytes(*array_ref!(challenge, 0, TIMESTAMP));

    if timestamp + EXPIRE < generate_timestamp().unwrap() {
        return format!(r#"{{ "status": "error", "message": "Challenge has expired" }}"#);
    }

    let mut hmac = HmacSha256::new_varkey(&challenge_signing_key.inner().as_slice()).unwrap();
    hmac.input(&challenge[..(TIMESTAMP + NONCE)]);
    let expected_tag = hmac.result().code();

    //Check if this is a challenge from here
    if expected_tag.ct_eq(&challenge[(TIMESTAMP + NONCE)..]).unwrap_u8() != 1 {
        return format!(r#"{{ "status": "error", "message": "Invalid challenge" }}"#);
    }

    let decodedkey = match bs58::decode(&response.address[8..]).with_check(None).into_vec() {
        Err(_) => return format!(r#"{{ "status": "error", "message": "Invalid address" }}"#),
        Ok(d) => d,
    };

     let pubkey = match PublicKey::from_bytes(decodedkey.as_slice()) {
        Err(_) => return format!(r#"{{ "status": "error", "message": "Address cannot be converted to a public key" }}"#),
        Ok(p) => p,
    };

    let sig = match Signature::from_bytes(signature.as_slice()) {
        Err(_) => return format!(r#"{{ "status": "error", "message": "Invalid signature" }}"#),
        Ok(s) => s,
    };

    let mut sha = Sha256::new();
    sha.input(format!("\x6DSovrin Signed Message:\nLength: {}\n", challenge.len()).as_bytes());
    sha.input(challenge.as_slice());
    let digest = sha.result();

    match pubkey.verify(digest.as_slice(), &sig) {
        Err(_) => format!(r#"{{ "status": "success", "result": false }}"#),
        Ok(_) => format!(r#"{{ "status": "success", "result": true }} "#)
    }
}

fn main() {
    let opt = Opt::from_args();
    let config = get_config(&opt);

    let request = get_trulioo_request(&config);
    let mut countries = BTreeMap::new();

    async_std::task::block_on(async {
        let codes = request.get_country_codes().await.unwrap();
        for code in codes {
            if let Ok(c) = Country::from_str(&code) {
                countries.insert(c.alpha2.to_string(), c);
            }
        }
    });

    let mut home = PathBuf::new();
    home.push(env!("HOME"));
    home.push(".token-website");
    if !home.exists() {
        fs::create_dir_all(home.clone()).unwrap();
    }
    home.push("config");

    if !home.exists() {
        let mut file = match fs::File::create(&home) {
            Err(why) => panic!("Couldn't create {:?}: {}", home, why.description()),
            Ok(file) => file
        };
        println!("config = {:?}", config);
        let recipe_toml = toml::Value::try_from(&config).unwrap();
        let contents = toml::to_string(&recipe_toml).unwrap();
        println!("contents = {}", contents);
        if let Err(why) = file.write_all(contents.as_bytes()) {
            panic!("Unable to write to {:?}: {}", home, why.description());
        }
    }

    rocket::ignite()
        .attach(SpaceHelmet::default())
        .manage(countries)
        .manage(base64_url::decode(&config.keys.challenge_signing_key).unwrap())
        .manage(request)
        .mount("/", StaticFiles::from("/public"))
        .mount("/api/v1", routes![get_allowed_countries,
                                      get_consents,
                                      get_payment_address_challenge,
                                      receive_payment_address_challenge
]).launch();
}

fn get_trulioo_request(config: &Config) -> TruliooRequest {
    let (url, key);
    if let Some(ref t) = config.trulioo {
        url = t.url.clone();
        if let Some(key_name) = &t.key_name {
            key = std::str::from_utf8(&get_trulioo_secret(&key_name, config.secret_backend)).unwrap().to_string();
        } else if let Some(key_value) = &t.key_value {
            key = key_value.clone();
        } else {
            key = prompt_for_value(trulioo::API_KEY_HEADER);
        }
    } else {
        url = prompt_for_value("trulioo api url");
        key = prompt_for_value(trulioo::API_KEY_HEADER);
    }

    TruliooRequest { key, url }
}

fn get_trulioo_secret(key_name: &str, secret_backend: Option<SecretBackend>) -> Vec<u8> {
    let apikey;
    if let Some(backend) = secret_backend {
        match backend {
            SecretBackend::OsKeyRing => {
                let mut keyring = get_os_keyring(TRULIOO_SERVICE).unwrap();
                apikey = keyring.get_secret(key_name).unwrap().as_slice().to_vec();
            },
            _ => {
                panic!("{} not handled", backend);
            }
        }
    } else {
        panic!("trulioo name cannot be used without a secret backend");
    }
    apikey
}

fn get_config(opt: &Opt) -> Config {
    let mut config: Config;
    match &opt.config {
        Some(c) => {
            if !c.exists() || !c.is_file() {
                panic!("The config file does not exist: '{:?}'", c);
            }

            match fs::read_to_string(c) {
                Err(why) => panic!("Unable to read {:?}: {}", c, why.description()),
                Ok(contents) => {
                    config = match toml::from_str(contents.as_str()) {
                        Ok(f) => f,
                        Err(e) => panic!("An error occurred while parsing '{:?}': {}", c, e.description())
                    };
                }
            };
            config.copy_from_opt(opt);
        },
        None => {
            config = get_home_config(opt);
        }
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
                panic ! ("An error occurred while reading {}: {}", value_name, e);
            }
        };
    }
}

fn get_home_config(opt: &Opt) -> Config {
    let mut home = PathBuf::new();
    home.push(env!("HOME"));
    home.push(".token-website");
    if !home.exists() {
        fs::create_dir_all(home.clone()).unwrap();
    }
    home.push("config");
    let mut config: Config;
    if home.exists() {
        let config_temp = match fs::read_to_string(&home) {
            Err(why) => panic!("Unable to read {:?}: {}", home, why.description()),
            Ok(c) => c
        };
        config = match toml::from_str(&config_temp) {
            Err(why) => panic!("Unable to parse {:?}: {}", home, why.description()),
            Ok(t) => t
        };
        config.copy_from_opt(opt);
    } else {
        config = opt.into();
    }
    config
}

fn generate_timestamp() -> Result<u64, String> {
    Ok(SystemTime::now().duration_since(UNIX_EPOCH).map_err(|e| e.to_string())?.as_secs())
}
