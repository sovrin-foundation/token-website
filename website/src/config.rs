use crate::cmd_opt::Opt;
use crate::secret_backend::SecretBackend;
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    pub port: u16,
    pub secret_backend: Option<SecretBackend>,
    pub truiloo: Option<Truiloo>
}

impl Default for Config {
    fn default() -> Self {
        Config {
            port: 5000,
            secret_backend: None,
            truiloo: None
        }
    }
}

impl From<&Opt> for Config {
    fn from(opt: &Opt) -> Self {
        let mut truiloo = None;
        if let Some(name) = &opt.truilooapikeyname {
            truiloo = Some(Truiloo {
                name: Some(name.to_string()),
                value: None
            });
        } else if let Some(value) = &opt.truilooapikeyvalue {
            truiloo = Some(Truiloo {
                name: None,
                value: Some(value.to_string())
            });
        }
        Config {
            port: opt.port,
            secret_backend: opt.secretbackend,
            truiloo
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Truiloo {
    pub name: Option<String>,
    pub value: Option<String>
}
