//! Configuration file support
//!
//! Loads settings from config files to avoid repetitive CLI flags.
//! Config precedence (highest to lowest):
//! 1. CLI flags
//! 2. ./vllama.toml (project-local)
//! 3. ~/.config/vllama/config.toml (user global)
//! 4. Built-in defaults

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::debug;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub server: ServerConfig,

    #[serde(default)]
    pub model: ModelConfig,

    #[serde(default)]
    pub logging: LoggingConfig,

    #[serde(default)]
    pub output: OutputConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,

    #[serde(default = "default_port")]
    pub port: u16,

    #[serde(default = "default_vllm_port")]
    pub vllm_port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub default_model: Option<String>,

    #[serde(default = "default_gpu_memory_utilization")]
    pub gpu_memory_utilization: f32,

    #[serde(default = "default_max_num_seqs")]
    pub max_num_seqs: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    #[serde(default = "default_log_level")]
    pub level: String,

    #[serde(default)]
    pub json: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OutputConfig {
    #[serde(default)]
    pub quiet: bool,

    #[serde(default)]
    pub json: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
            vllm_port: default_vllm_port(),
        }
    }
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            default_model: None,
            gpu_memory_utilization: default_gpu_memory_utilization(),
            max_num_seqs: default_max_num_seqs(),
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            json: false,
        }
    }
}

// Default value functions (for serde)
fn default_host() -> String {
    "127.0.0.1".to_string()
}

fn default_port() -> u16 {
    11435  // Ollama-compatible API on next port (works alongside Ollama on 11434)
}

fn default_vllm_port() -> u16 {
    8100
}

fn default_gpu_memory_utilization() -> f32 {
    0.9
}

fn default_max_num_seqs() -> usize {
    256
}

fn default_log_level() -> String {
    "info".to_string()
}

impl Config {
    /// Load configuration from files
    ///
    /// Priority (highest to lowest):
    /// 1. ./vllama.toml (current directory)
    /// 2. ~/.config/vllama/config.toml (user config)
    /// 3. Built-in defaults
    pub fn load() -> Result<Self> {
        let mut config = Config::default();

        // Load user config
        if let Some(user_config_path) = Self::user_config_path() {
            if user_config_path.exists() {
                debug!("Loading user config from {:?}", user_config_path);
                let user_config = Self::load_from_file(&user_config_path)?;
                config = config.merge(user_config);
            }
        }

        // Load project-local config (overrides user config)
        let local_config_path = PathBuf::from("vllama.toml");
        if local_config_path.exists() {
            debug!("Loading project config from {:?}", local_config_path);
            let local_config = Self::load_from_file(&local_config_path)?;
            config = config.merge(local_config);
        }

        Ok(config)
    }

    /// Load config from a specific file
    fn load_from_file(path: &PathBuf) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {:?}", path))?;

        toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {:?}", path))
    }

    /// Get user config file path (~/.config/vllama/config.toml)
    fn user_config_path() -> Option<PathBuf> {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .ok()?;

        let config_dir = std::env::var("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(home).join(".config"));

        Some(config_dir.join("vllama").join("config.toml"))
    }

    /// Merge another config into this one (other takes priority)
    fn merge(mut self, other: Self) -> Self {
        // Server settings
        if other.server.host != default_host() {
            self.server.host = other.server.host;
        }
        if other.server.port != default_port() {
            self.server.port = other.server.port;
        }
        if other.server.vllm_port != default_vllm_port() {
            self.server.vllm_port = other.server.vllm_port;
        }

        // Model settings
        if other.model.default_model.is_some() {
            self.model.default_model = other.model.default_model;
        }
        if (other.model.gpu_memory_utilization - default_gpu_memory_utilization()).abs() > 0.001 {
            self.model.gpu_memory_utilization = other.model.gpu_memory_utilization;
        }
        if other.model.max_num_seqs != default_max_num_seqs() {
            self.model.max_num_seqs = other.model.max_num_seqs;
        }

        // Logging settings
        if other.logging.level != default_log_level() {
            self.logging.level = other.logging.level;
        }
        if other.logging.json {
            self.logging.json = true;
        }

        // Output settings
        if other.output.quiet {
            self.output.quiet = true;
        }
        if other.output.json {
            self.output.json = true;
        }

        self
    }

    /// Generate example config file
    pub fn example() -> String {
        let config = Config::default();
        toml::to_string_pretty(&config).unwrap_or_else(|_| String::from("# Failed to generate example"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 11435);
        assert_eq!(config.model.gpu_memory_utilization, 0.9);
    }

    #[test]
    fn test_config_merge() {
        let base = Config::default();
        let mut override_config = Config::default();
        override_config.server.port = 9999;

        let merged = base.merge(override_config);
        assert_eq!(merged.server.port, 9999);
        assert_eq!(merged.server.host, "127.0.0.1"); // unchanged
    }

    #[test]
    fn test_example_config() {
        let example = Config::example();
        assert!(example.contains("[server]"));
        assert!(example.contains("[model]"));
    }
}
