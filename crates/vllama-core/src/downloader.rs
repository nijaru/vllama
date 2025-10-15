use std::path::PathBuf;
use std::fs;
use std::io::Write;
use crate::{Error, Result};

pub struct DownloadProgress {
    pub downloaded: u64,
    pub total: u64,
    pub status: String,
}

pub struct ModelDownloader {
    cache_dir: PathBuf,
    client: reqwest::Client,
}

impl ModelDownloader {
    pub fn new() -> Result<Self> {
        let cache_dir = Self::get_cache_dir()?;
        fs::create_dir_all(&cache_dir)?;

        Ok(Self {
            cache_dir,
            client: reqwest::Client::new(),
        })
    }

    fn get_cache_dir() -> Result<PathBuf> {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .map_err(|_| Error::ConfigError("HOME directory not found".to_string()))?;

        Ok(PathBuf::from(home).join(".cache").join("vllama").join("models"))
    }

    pub fn get_model_dir(&self, repo_id: &str) -> PathBuf {
        let safe_name = repo_id.replace("/", "_");
        self.cache_dir.join(&safe_name)
    }

    pub fn get_model_path(&self, repo_id: &str, filename: &str) -> PathBuf {
        self.get_model_dir(repo_id).join(filename)
    }

    pub fn model_exists(&self, repo_id: &str, filename: &str) -> bool {
        self.get_model_path(repo_id, filename).exists()
    }

    pub async fn download_model(
        &self,
        repo_id: &str,
        filename: Option<&str>,
        progress_callback: impl Fn(DownloadProgress),
    ) -> Result<PathBuf> {
        let file_name = match filename {
            Some(f) => f.to_string(),
            None => {
                if repo_id.ends_with(".gguf") || repo_id.ends_with(".GGUF") {
                    repo_id.split('/').last().unwrap_or("model.gguf").to_string()
                } else {
                    let model_name = repo_id.split('/').last().unwrap_or("model");
                    if model_name.ends_with("-GGUF") {
                        let base_name = &model_name[..model_name.len() - 5];
                        format!("{}-Q4_K_M.gguf", base_name)
                    } else {
                        "model.gguf".to_string()
                    }
                }
            }
        };

        let model_dir = self.get_model_dir(repo_id);
        let model_path = model_dir.join(&file_name);

        if model_path.exists() {
            progress_callback(DownloadProgress {
                downloaded: 0,
                total: 0,
                status: "already_exists".to_string(),
            });
            return Ok(model_path);
        }

        fs::create_dir_all(&model_dir)
            .map_err(|e| Error::ModelLoadFailed(format!("Failed to create model directory: {}", e)))?;

        let url = format!("https://huggingface.co/{}/resolve/main/{}", repo_id, file_name);

        progress_callback(DownloadProgress {
            downloaded: 0,
            total: 0,
            status: format!("downloading from {}", url),
        });

        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| Error::ModelLoadFailed(format!("Failed to download: {}", e)))?;

        if !response.status().is_success() {
            return Err(Error::ModelNotFound(format!(
                "Failed to download model from HuggingFace: {} (status: {})",
                url,
                response.status()
            )));
        }

        let total_size = response.content_length().unwrap_or(0);

        let temp_path = model_path.with_extension("tmp");
        let mut file = fs::File::create(&temp_path)
            .map_err(|e| Error::ModelLoadFailed(format!("Failed to create file: {}", e)))?;

        let mut downloaded = 0u64;
        let mut stream = response.bytes_stream();

        use futures::StreamExt;
        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result
                .map_err(|e| Error::ModelLoadFailed(format!("Download error: {}", e)))?;

            file.write_all(&chunk)
                .map_err(|e| Error::ModelLoadFailed(format!("Write error: {}", e)))?;

            downloaded += chunk.len() as u64;

            progress_callback(DownloadProgress {
                downloaded,
                total: total_size,
                status: "downloading".to_string(),
            });
        }

        file.flush()
            .map_err(|e| Error::ModelLoadFailed(format!("Failed to flush file: {}", e)))?;

        drop(file);

        fs::rename(&temp_path, &model_path)
            .map_err(|e| Error::ModelLoadFailed(format!("Failed to rename file: {}", e)))?;

        if temp_path.exists() {
            let _ = fs::remove_file(&temp_path);
        }

        progress_callback(DownloadProgress {
            downloaded: total_size,
            total: total_size,
            status: "completed".to_string(),
        });

        Ok(model_path)
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
    fn test_cache_dir() {
        let downloader = ModelDownloader::new().unwrap();
        assert!(downloader.cache_dir.to_string_lossy().contains("vllama"));
    }

    #[test]
    fn test_model_path() {
        let downloader = ModelDownloader::new().unwrap();
        let path = downloader.get_model_path("modularai/Llama-3.1-8B-Instruct-GGUF", "model.gguf");
        assert!(path.to_string_lossy().contains("modularai_Llama-3.1-8B-Instruct-GGUF"));
        assert!(path.to_string_lossy().ends_with("model.gguf"));
    }
}
