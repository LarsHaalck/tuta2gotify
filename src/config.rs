use serde::Deserialize;
use tracing::debug;
use tuta_poll::config::Account;
use anyhow::{Result, Error, Context};

#[derive(Deserialize, Debug)]
pub struct Config {
    pub account: Account,
    pub gotify: Gotify
}


#[derive(Deserialize, Debug)]
pub struct Gotify {
    pub url: url::Url,
    pub token: String,
    #[serde(default = "default_format")]
    pub format: String,
}

fn default_format() -> String {
    "New Mail from {{name}} <{{address}}>: {{subject}}\n{{body}}".to_string()
}



impl Config {
    pub fn read(config_file: std::path::PathBuf) -> Result<Config> {
        let config: Option<Config>;
        // try to read file first
        if config_file.is_file() {
            debug!("Trying to read config from file: {}", config_file.display());
            let config_str =
                std::fs::read_to_string(config_file).context("Could not read config file")?;
            config =
                Some(toml::from_str(config_str.as_str()).context("Could not parse config file")?);
        } else {
            debug!("Trying to read config from env");
            let account = envy::prefixed("T2G_ACCOUNT_").from_env::<Account>()?;
            let gotify = envy::prefixed("T2G_GOTIFY_").from_env::<Gotify>()?;

            config = Some(Config { account, gotify });
        }

        let config = config.ok_or(Error::msg(
            "Could not read config file from either file nor environment",
        ))?;
        Ok(config)
    }
}
