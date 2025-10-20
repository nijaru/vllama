use anyhow::Result;
use tracing::info;
use vllama_core::ModelDownloader;

pub async fn execute(model: String) -> Result<()> {
    info!("Pulling model: {}", model);
    println!("Downloading {} from HuggingFace Hub...", model);

    let downloader = ModelDownloader::new()?;

    // Check if already cached
    if downloader.model_exists(&model).await {
        println!("✓ Model {} already exists in cache", model);
        let path = downloader.get_model_path(&model).await?;
        println!("  Location: {}", path.display());
        return Ok(());
    }

    // Download with progress callback
    let path = downloader.download_model(&model, |progress| {
        if progress.total > 0 {
            let percent = (progress.downloaded * 100) / progress.total;
            println!("  Progress: {}% - {}", percent, progress.status);
        } else {
            println!("  {}", progress.status);
        }
    }).await?;

    println!("✓ Model {} downloaded successfully", model);
    println!("  Location: {}", path.display());

    Ok(())
}
