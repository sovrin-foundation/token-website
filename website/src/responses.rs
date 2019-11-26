use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct PaymentAddressChallengeResponse {
    pub address: String,
    pub challenge: String,
    pub signature: String
}
