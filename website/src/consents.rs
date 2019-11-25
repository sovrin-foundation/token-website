use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Consent {
    name: String,
    description: String,
    country_code: celes::Country
}
