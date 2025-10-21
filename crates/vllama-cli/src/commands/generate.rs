use anyhow::Result;
use vllama_core::GenerateRequest;
use vllama_engine::{InferenceEngine, VllmOpenAIEngine};
use tracing::info;

pub async fn execute(model: String, prompt: String, stream: bool) -> Result<()> {
    info!("Generating with model: {}", model);
    info!("Stream: {}", stream);

    if stream {
        println!("(Streaming not yet implemented)");
        return Ok(());
    }

    let vllm_engine = VllmOpenAIEngine::new("http://127.0.0.1:8100");

    if !vllm_engine.health_check().await? {
        anyhow::bail!("vLLM OpenAI server not available (run: vllama serve --model <model-name>)");
    }

    let request = GenerateRequest::new(1, model.clone(), prompt.clone()).with_max_tokens(100);

    println!("Generating response...\n");

    let response = vllm_engine.generate(request).await?;

    println!("Response: {}", response.text);
    println!();

    Ok(())
}
