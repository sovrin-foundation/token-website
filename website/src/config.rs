use crate::cmd_opt::Opt;
use crate::secret_backend::SecretBackend;
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    pub port: u16,
    pub secret_backend: Option<SecretBackend>,
    pub trulioo: Option<Trulioo>
}

impl Default for Config {
    fn default() -> Self {
        Config {
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
        Config {
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
