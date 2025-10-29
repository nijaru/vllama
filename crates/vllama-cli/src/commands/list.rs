use anyhow::Result;
use serde::Serialize;
use vllama_core::ModelDownloader;

use crate::output::{self, OutputMode};

#[derive(Serialize)]
struct ListResult {
    models: Vec<ModelEntry>,
    total_size_mb: u64,
}

#[derive(Serialize)]
struct ModelEntry {
    name: String,
    size_mb: u64,
    path: String,
}

pub async fn execute(output_mode: OutputMode) -> Result<()> {
    let downloader = ModelDownloader::new()?;
    let models = downloader.list_cached_models()?;

    if models.is_empty() {
        match output_mode {
            OutputMode::Json => {
                output::json(&ListResult {
                    models: Vec::new(),
                    total_size_mb: 0,
                });
            }
            OutputMode::Quiet => {
                // Silent
            }
            OutputMode::Normal => {
                println!("{}", output::info("No models cached"));
                println!("{}", output::bullet("Pull a model with: vllama pull <model-name>"));
            }
        }
        return Ok(());
    }

    let total_size_mb: u64 = models.iter().map(|m| m.size_mb).sum();

    match output_mode {
        OutputMode::Json => {
            output::json(&ListResult {
                models: models.iter().map(|m| ModelEntry {
                    name: m.name.clone(),
                    size_mb: m.size_mb,
                    path: m.path.display().to_string(),
                }).collect(),
                total_size_mb,
            });
        }
        OutputMode::Quiet => {
            // Just list names
            for model in &models {
                println!("{}", model.name);
            }
        }
        OutputMode::Normal => {
            println!("{}", output::section("Cached Models"));
            println!();

            for model in &models {
                println!("  {}", model.name);
                output::kv("Size", &format!("{} MB", model.size_mb));
                output::kv("Path", &model.path.display().to_string());
                println!();
            }

            println!("{}", output::info(&format!("Total: {} models, {} MB", models.len(), total_size_mb)));
        }
    }

    Ok(())
}
