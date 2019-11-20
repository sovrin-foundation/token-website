use crate::secret_backend::SecretBackend;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "basic",
    version = "0.1",
    about = "Sovrin Foundation Token Website"
)]
pub struct Opt {
    #[structopt(short, long, parse(from_os_str))]
    pub config: Option<PathBuf>,
    #[structopt(short, long, default_value = "5000")]
    pub port: u16,
    #[structopt(short, long)]
    pub secretbackend: Option<SecretBackend>,
    #[structopt(short = "n", long)]
    pub truilooapikeyname: Option<String>,
    #[structopt(short = "k", long)]
    pub truilooapikeyvalue: Option<String>,
}
