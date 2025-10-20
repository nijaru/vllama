use std::path::PathBuf;
use crate::{Error, Result};
use hf_hub::api::tokio::Api;
use tracing::{info, warn};

pub struct DownloadProgress {
    pub downloaded: u64,
    pub total: u64,
    pub status: String,
}

pub struct ModelDownloader {
    api: Api,
}

impl ModelDownloader {
    pub fn new() -> Result<Self> {
        // Use default API (no custom builder needed)
        // This automatically uses https://huggingface.co
        let api = Api::new()
            .map_err(|e| Error::ConfigError(format!("Failed to create HF API: {}", e)))?;

        Ok(Self { api })
    }

    /// Get the cached path for a model
    pub async fn get_model_path(&self, repo_id: &str) -> Result<PathBuf> {
        let repo = self.api.model(repo_id.to_string());

        // Get config file which gives us the cache directory
        let config_path = repo.get("config.json").await
            .map_err(|e| Error::ModelNotFound(format!("Model {} not found: {}", repo_id, e)))?;

        // Return parent directory (the model directory)
        Ok(config_path.parent()
            .ok_or_else(|| Error::ModelNotFound("Invalid model path".to_string()))?
            .to_path_buf())
    }

    /// Check if model exists in cache
    pub async fn model_exists(&self, repo_id: &str) -> bool {
        // Check if config.json exists in cache
        let repo = self.api.model(repo_id.to_string());
        repo.get("config.json").await.is_ok()
    }

    /// Download model from HuggingFace Hub
    ///
    /// This uses the official hf-hub crate which provides:
    /// - Automatic resume on network failures
    /// - Progress tracking
    /// - Authentication via HF_TOKEN env var
    /// - Caching in ~/.cache/huggingface/
    /// - Mirror support and CDN optimization
    pub async fn download_model(
        &self,
        repo_id: &str,
        progress_callback: impl Fn(DownloadProgress),
    ) -> Result<PathBuf> {
        info!("Downloading model: {}", repo_id);

        progress_callback(DownloadProgress {
            downloaded: 0,
            total: 0,
            status: format!("Fetching {} from HuggingFace Hub", repo_id),
        });

        let repo = self.api.model(repo_id.to_string());

        // Download essential model files
        // hf-hub handles all the complexity: resume, retries, progress, etc.

        // Download config first (small file)
        let config_path = repo.get("config.json").await
            .map_err(|e| Error::ModelLoadFailed(format!("Failed to download config.json: {}", e)))?;

        progress_callback(DownloadProgress {
            downloaded: 1,
            total: 3,
            status: "Downloaded config.json".to_string(),
        });

        // Download tokenizer config
        let _tokenizer_config = repo.get("tokenizer_config.json").await
            .map_err(|e| {
                warn!("tokenizer_config.json not found: {}", e);
                e
            })
            .ok();

        progress_callback(DownloadProgress {
            downloaded: 2,
            total: 3,
            status: "Downloaded tokenizer config".to_string(),
        });

        // Try to download model weights (safetensors preferred)
        let _model_file = match repo.get("model.safetensors").await {
            Ok(path) => path,
            Err(_) => {
                // Fallback to pytorch_model.bin
                repo.get("pytorch_model.bin").await
                    .map_err(|e| Error::ModelLoadFailed(format!("Failed to download model weights: {}", e)))?
            }
        };

        progress_callback(DownloadProgress {
            downloaded: 3,
            total: 3,
            status: "completed".to_string(),
        });

        info!("Model {} downloaded successfully", repo_id);

        // Return the model directory (parent of config file)
        Ok(config_path.parent()
            .ok_or_else(|| Error::ModelLoadFailed("Invalid model path".to_string()))?
            .to_path_buf())
    }
}

impl Default for ModelDownloader {
    fn default() -> Self {
        Self::new().expect("Failed to create ModelDownloader")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_downloader_creation() {
        let downloader = ModelDownloader::new();
        assert!(downloader.is_ok());
    }

    #[tokio::test]
    async fn test_model_path() {
        // This test requires network access
        // Skip in CI unless HF_TOKEN is set
        if std::env::var("HF_TOKEN").is_err() {
            return;
        }

        let downloader = ModelDownloader::new().unwrap();

        // Try a small test model
        let result = downloader.download_model(
            "hf-internal-testing/tiny-random-gpt2",
            |progress| {
                println!("Progress: {}/{} - {}",
                    progress.downloaded,
                    progress.total,
                    progress.status
                );
            }
        ).await;

        assert!(result.is_ok());
    }
}
