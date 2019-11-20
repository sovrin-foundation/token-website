use self::SecretBackend::*;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum SecretBackend {
    AwsKms,
    AzureKeyVault,
    OsKeyRing,
}

impl std::fmt::Display for SecretBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            AwsKms => write!(f, "awskms"),
            AzureKeyVault => write!(f, "azurekeyvault"),
            OsKeyRing => write!(f, "oskeyring")
        }
    }
}

impl FromStr for SecretBackend {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        SecretBackend::try_from(s)
    }
}

impl TryFrom<&str> for SecretBackend {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "awskms" => Ok(AwsKms),
            "azurekeyvault" => Ok(AzureKeyVault),
            "oskeyring" => Ok(OsKeyRing),
            _ => Err(format!("Unknown value: {}", s))
        }
    }
}

impl TryFrom<String> for SecretBackend {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        SecretBackend::try_from(s.as_str())
    }
}
