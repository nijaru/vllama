use anyhow::Result;
use hyperllama_core::GenerateRequest;
use hyperllama_engine::{InferenceEngine, MaxEngine};
use std::path::PathBuf;
use tracing::info;

pub async fn execute(model: String, prompt: String, stream: bool) -> Result<()> {
    info!("Generating with model: {}", model);
    info!("Stream: {}", stream);

    if stream {
        println!("(Streaming not yet implemented)");
        return Ok(());
    }

    let mut max_engine = MaxEngine::new()?;

    if !max_engine.health_check().await? {
        anyhow::bail!("MAX Engine service not available (is the Python service running?)");
    }

    let model_path = PathBuf::from(&model);
    let handle = max_engine.load_model(&model_path).await?;

    let model_id = max_engine
        .get_model_id(handle)
        .ok_or_else(|| anyhow::anyhow!("Model handle not found"))?;

    let request = GenerateRequest::new(1, model_id, prompt.clone()).with_max_tokens(100);

    println!("Generating response...\n");

    let response = max_engine.generate(request).await?;

    println!("Response: {}", response.text);
    println!();

    Ok(())
}
