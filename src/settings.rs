use std::net::SocketAddr;

use anyhow::Result;
use bip39::MnemonicType;
use config::{Config, File, FileFormat};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    pub password: String,
    pub server_addr: SocketAddr,
}

impl Settings {
    pub fn new() -> Result<Self> {
        let mut config = Config::new();
        config.merge(File::new("settings.json", FileFormat::Json))?;

        let settings = config.try_into()?;
        Ok(settings)
    }
}
