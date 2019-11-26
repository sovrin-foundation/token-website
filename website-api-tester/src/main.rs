use serde::Serialize;
use sha2::Digest;
use sodiumoxide::crypto::sign::{
    sign_detached, gen_keypair,
    ed25519::SecretKey
};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "basic",
    version = "0.1",
    about = "Sovrin Foundation Token Website"
)]
struct Opt {
    #[structopt(subcommand)]
    pub cmd: Command
}

#[derive(Debug, StructOpt)]
enum Command {
    #[structopt(name = "sign")]
    Sign {
        #[structopt(short, long)]
        key: Option<String>,
        #[structopt(name = "TOKEN")]
        token: String
    }
}

#[derive(Serialize)]
struct PaymentAddressChallengeReponse {
    address: String,
    challenge: String,
    signature: String
}


fn main() {
    sodiumoxide::init().unwrap();

    let opt = Opt::from_args();
    match opt.cmd {
        Command::Sign { key, token } => {
            let (pk, sk) = match key {
                Some(k) => {
                    let k1 = bs58::decode(k).into_vec().unwrap();
                    let sk1 = SecretKey::from_slice(k1.as_slice()).unwrap();
                    let pk1 = sk1.public_key();
                    (pk1, sk1)
                },
                None => gen_keypair()
            };
            let mut sha = sha2::Sha256::new();

            let challenge = base64_url::decode(&token).unwrap();

            sha.input(format!("\x6DSovrin Signed Message:\nLength: {}\n", challenge.len()).as_bytes());
            sha.input(challenge.as_slice());
            let data = sha.result();
            let signature = sign_detached(data.as_slice(), &sk);

            let response = PaymentAddressChallengeReponse {
                address: format!("pay:sov:{}", bs58::encode(&pk[..]).into_string()),
                challenge: token,
                signature: base64_url::encode(&signature[..])
            };

            println!("key = {}", bs58::encode(sk).into_string());
            println!("response = {}", serde_json::to_string(&response).unwrap());
        }
    }
}
