use figment::{
    providers::{Env, Format, Toml},
    Figment,
};
use once_cell::sync::OnceCell;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Database {
    pub url: String,
    pub port: Option<u8>,
    pub user: Option<String>,
    pub password: Option<String>,
    pub database: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub db: Database,
}

static INSTANCE: OnceCell<Config> = OnceCell::new();

impl Config {
    pub fn global() -> &'static Config {
        INSTANCE.get().expect("configuration is not initialized")
    }

    pub fn load() -> () {
        let config: Config = Figment::new()
            .merge(Toml::file("toedi.toml"))
            .merge(Env::prefixed("TOEDI_"))
            .extract()
            .expect("cannot load config");
        INSTANCE.set(config).expect("config only set once");
    }
}
