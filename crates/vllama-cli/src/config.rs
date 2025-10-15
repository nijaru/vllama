use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub models_dir: PathBuf,
    pub server_host: String,
    pub server_port: u16,
}

impl Default for Config {
    fn default() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let models_dir = PathBuf::from(home).join(".vllama").join("models");

        Self {
            models_dir,
            server_host: "127.0.0.1".to_string(),
            server_port: 11434,
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        Ok(Self::default())
    }
}
