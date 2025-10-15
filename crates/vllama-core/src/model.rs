use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ModelHandle(pub u64);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelFormat {
    Gguf,
    SafeTensors,
    Pytorch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub path: PathBuf,
    pub format: ModelFormat,
    pub size_bytes: u64,
    pub architecture: String,
    pub context_length: usize,
    pub quantization: Option<String>,
}

impl ModelInfo {
    pub fn new(name: String, path: PathBuf) -> Self {
        Self {
            name,
            path,
            format: ModelFormat::Gguf,
            size_bytes: 0,
            architecture: "unknown".to_string(),
            context_length: 4096,
            quantization: None,
        }
    }

    pub fn with_format(mut self, format: ModelFormat) -> Self {
        self.format = format;
        self
    }

    pub fn with_size(mut self, size_bytes: u64) -> Self {
        self.size_bytes = size_bytes;
        self
    }

    pub fn with_architecture(mut self, architecture: String) -> Self {
        self.architecture = architecture;
        self
    }
}
