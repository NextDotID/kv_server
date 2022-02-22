use serde::Deserialize;

use crate::error::Error;

#[derive(Clone, Deserialize, Default)]
pub struct Config {
    db: ConfigDB,
    web: ConfigWeb,
}

#[derive(Clone, Deserialize, Default)]
struct ConfigDB {
    host: String,
    port: u16,
    username: String,
    password: String,
    db: String,
}

#[derive(Clone, Deserialize, Default)]
struct ConfigWeb {
    listen: String,
    port: u16,
}

fn from_env() -> Result<Config, Error> {
    todo!()
}

fn from_aws_secret() -> Result<Config, Error> {
    todo!()
}
