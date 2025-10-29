use anyhow::Result;
use serde::Serialize;
use tracing::info;
use vllama_core::ModelDownloader;

use crate::output::{self, OutputMode};

#[derive(Serialize)]
struct RmResult {
    model: String,
    deleted: bool,
}

pub async fn execute(model: String, output_mode: OutputMode) -> Result<()> {
    info!("Removing model: {}", model);

    let downloader = ModelDownloader::new()?;

    // Check if model exists
    if !downloader.model_exists(&model).await {
        match output_mode {
            OutputMode::Json => {
                output::json(&RmResult {
                    model: model.clone(),
                    deleted: false,
                });
            }
            OutputMode::Quiet => {
                // Silent error - just return error code
            }
            OutputMode::Normal => {
                println!("{}", output::error(&format!("Model {} not found in cache", model)));
            }
        }
        anyhow::bail!("Model not found");
    }

    // Get size before deleting
    let size_mb = if let Ok(path) = downloader.get_model_path(&model).await {
        calculate_dir_size(&path).unwrap_or(0) / 1024 / 1024
    } else {
        0
    };

    // Delete the model
    downloader.delete_model(&model)?;

    // Output result
    match output_mode {
        OutputMode::Json => {
            output::json(&RmResult {
                model: model.clone(),
                deleted: true,
            });
        }
        OutputMode::Quiet => {
            // Silent success
        }
        OutputMode::Normal => {
            println!("{}", output::success(&format!("Deleted {}", model)));
            if size_mb > 0 {
                output::kv("Freed space", &format!("{} MB", size_mb));
            }
        }
    }

    Ok(())
}

/// Calculate total size of a directory recursively
fn calculate_dir_size(path: &std::path::Path) -> Result<u64> {
    let mut total = 0;

    if path.is_file() {
        return Ok(std::fs::metadata(path)?.len());
    }

    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let entry_path = entry.path();

        if entry_path.is_file() {
            total += std::fs::metadata(&entry_path)?.len();
        } else if entry_path.is_dir() {
            total += calculate_dir_size(&entry_path)?;
        }
    }

    Ok(total)
}
