use anyhow::Result;
use serde::Serialize;
use tracing::info;
use vllama_core::ModelDownloader;

use crate::output::{self, OutputMode};

#[derive(Serialize)]
struct PullResult {
    model: String,
    cached: bool,
    path: String,
}

pub async fn execute(model: String, output_mode: OutputMode) -> Result<()> {
    info!("Pulling model: {}", model);

    let downloader = ModelDownloader::new()?;

    // Check if already cached
    if downloader.model_exists(&model).await {
        let path = downloader.get_model_path(&model).await?;

        match output_mode {
            OutputMode::Json => {
                output::json(&PullResult {
                    model: model.clone(),
                    cached: true,
                    path: path.display().to_string(),
                });
            }
            OutputMode::Quiet => {
                // Silent - model already exists
            }
            OutputMode::Normal => {
                println!("{}", output::success(&format!("Model {} already cached", model)));
                output::kv("Location", &path.display().to_string());
            }
        }

        return Ok(());
    }

    // Show download header
    if output_mode == OutputMode::Normal {
        println!("{}", output::section(&format!("Downloading {}", model)));
    }

    // Create progress bar for download
    let pb = if output_mode == OutputMode::Normal {
        Some(output::progress_bar(3, "Fetching from HuggingFace Hub"))
    } else {
        None
    };

    // Download with progress updates
    let path = downloader.download_model(&model, |progress| {
        if let Some(ref pb) = pb {
            pb.set_position(progress.downloaded);
            if !progress.status.is_empty() && progress.status != "completed" {
                pb.set_message(progress.status.clone());
            }
        }
    }).await?;

    // Finish progress bar
    if let Some(pb) = pb {
        pb.finish_and_clear();
    }

    // Output result
    match output_mode {
        OutputMode::Json => {
            output::json(&PullResult {
                model: model.clone(),
                cached: false,
                path: path.display().to_string(),
            });
        }
        OutputMode::Quiet => {
            // Silent success
        }
        OutputMode::Normal => {
            println!("{}", output::success(&format!("Downloaded {}", model)));
            output::kv("Location", &path.display().to_string());
        }
    }

    Ok(())
}
