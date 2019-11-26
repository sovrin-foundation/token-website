use crate::cmd_opt::Opt;
use crate::secret_backend::SecretBackend;
use rand::RngCore;
use serde::{Serialize, Deserialize};
use zeroize::Zeroize;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    pub keys: Keys,
    pub port: u16,
    pub secret_backend: Option<SecretBackend>,
    pub trulioo: Option<Trulioo>
}

impl Config {
    pub fn copy_from_opt(&mut self, opt: &Opt) {
        if let Some(ref c) = opt.challenge_signing_key {
            if let Err(why) = base64_url::decode(c) {
                panic!("Incompatible format for challenge signing key: {}", why);
            }
           self.keys.challenge_signing_key = c.to_string();
        }

        if let Some(ref mut t) = self.trulioo {
            if let Some(url) = &opt.truliooapiurl {
                t.url = url.to_string();
            }
            if let Some(name) = &opt.truliooapikeyname {
                t.key_name = Some(name.to_string());
            }
            if let Some(value) = &opt.truliooapikeyvalue {
                t.key_value = Some(value.to_string());
            }
        } else {
            let url = opt.truliooapiurl.clone().unwrap_or(String::new());
            if let Some(name) = &opt.truliooapikeyname {
                self.trulioo = Some(Trulioo {
                    key_name: Some(name.to_string()),
                    key_value: None,
                    url
                });
            } else if let Some(value) = &opt.truliooapikeyvalue {
                self.trulioo = Some(Trulioo {
                    key_name: None,
                    key_value: Some(value.to_string()),
                    url
                });
            }
        }

        if opt.secretbackend.is_some() {
            self.secret_backend = opt.secretbackend.clone();
        }

        self.port = opt.port;
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            keys: Keys::default(),
            port: 8000,
            secret_backend: None,
            trulioo: None
        }
    }
}

impl From<&Opt> for Config {
    fn from(opt: &Opt) -> Self {
        let mut trulioo = None;
        let url = opt.truliooapiurl.clone().unwrap_or(String::new());
        if let Some(name) = &opt.truliooapikeyname {
            trulioo = Some(Trulioo {
                key_name: Some(name.to_string()),
                key_value: None,
                url
            });
        } else if let Some(value) = &opt.truliooapikeyvalue {
            trulioo = Some(Trulioo {
                key_name: None,
                key_value: Some(value.to_string()),
                url
            });
        }
        let keys =
            if let Some(ref c) = opt.challenge_signing_key {
                if let Err(why) = base64_url::decode(c) {
                    panic!("Incompatible format for challenge signing key: {}", why);
                }
                Keys { challenge_signing_key: c.to_string() }
            } else {
                Keys::default()
            };
        Config {
            keys,
            port: opt.port,
            secret_backend: opt.secretbackend,
            trulioo
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Trulioo {
    pub key_name: Option<String>,
    pub key_value: Option<String>,
    pub url: String
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Keys {
    pub challenge_signing_key: String
}

impl Default for Keys {
    fn default() -> Self {
        let mut rng = rand::rngs::OsRng{};
        let mut key = vec![0u8; 32];
        rng.fill_bytes(key.as_mut_slice());
        let challenge_signing_key = base64_url::encode(&key);
        key.zeroize();
        Self { challenge_signing_key }
    }
}
