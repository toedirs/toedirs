cfg_if::cfg_if! {
    if #[cfg(feature = "ssr")]{
        use figment::{
            providers::{Env, Format, Toml},
            Figment,
        };
        use once_cell::sync::OnceCell;
        use serde::Deserialize;
    }
}

#[cfg(feature = "ssr")]
#[derive(Deserialize, Debug)]
pub struct Database {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub database: String,
}

#[cfg(feature = "ssr")]
#[derive(Deserialize, Debug)]
pub struct Config {
    pub db: Database,
}

#[cfg(feature = "ssr")]
static INSTANCE: OnceCell<Config> = OnceCell::new();

#[cfg(feature = "ssr")]
impl Config {
    pub fn global() -> &'static Config {
        INSTANCE.get().expect("configuration is not initialized")
    }

    pub fn load() {
        let config: Config = Figment::new()
            .merge(Toml::file("toedi.toml"))
            .merge(Env::prefixed("TOEDI_"))
            .extract()
            .expect("cannot load config");
        INSTANCE.set(config).expect("config only set once");
    }
}