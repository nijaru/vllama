/// vLLM OpenAI-compatible engine
///
/// This engine communicates with vLLM's official OpenAI-compatible server
/// for production-ready request batching and performance.
use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use std::path::Path;
use tracing::info;
use vllama_core::{
    CompletionRequest, GenerateRequest, GenerateResponse, GenerationStats,
    Hardware, ModelHandle, OpenAIClient, Result,
};

use crate::engine::{EngineCapabilities, EngineType, InferenceEngine};

pub struct VllmOpenAIEngine {
    client: OpenAIClient,
    #[allow(dead_code)]
    base_url: String,
}

impl VllmOpenAIEngine {
    pub fn new(base_url: impl Into<String>) -> Self {
        let base_url = base_url.into();
        let client = OpenAIClient::new(base_url.clone());

        Self { client, base_url }
    }

    /// Generate chat completion using OpenAI chat API
    pub async fn generate_chat_completion(
        &self,
        model: String,
        messages: Vec<vllama_core::ChatMessage>,
        options: vllama_core::GenerateOptions,
    ) -> Result<vllama_core::ChatCompletionResponse> {
        use vllama_core::openai::ChatMessage as OpenAIChatMessage;
        use vllama_core::openai::ChatCompletionRequest;

        let openai_messages: Vec<OpenAIChatMessage> = messages
            .iter()
            .map(|msg| {
                use vllama_core::ChatRole;
                let role = match msg.role {
                    ChatRole::System => "system",
                    ChatRole::User => "user",
                    ChatRole::Assistant => "assistant",
                    ChatRole::Tool => "tool",
                };
                OpenAIChatMessage {
                    role: role.to_string(),
                    content: msg.content.clone(),
                }
            })
            .collect();

        let request = ChatCompletionRequest {
            model: model.clone(),
            messages: openai_messages,
            max_tokens: options.sampling.max_tokens,
            temperature: Some(options.sampling.temperature),
            top_p: Some(options.sampling.top_p),
            stream: Some(false),
        };

        self.client.create_chat_completion(request).await
    }
}

#[async_trait]
impl InferenceEngine for VllmOpenAIEngine {
    fn engine_type(&self) -> EngineType {
        EngineType::Vllm
    }

    fn capabilities(&self) -> EngineCapabilities {
        EngineCapabilities {
            supports_continuous_batching: true,
            supports_flash_attention: true,
            supports_paged_attention: true,
            supports_speculative_decoding: false,
            supports_quantization: vec![
                "awq".to_string(),
                "gptq".to_string(),
                "squeezellm".to_string(),
                "fp8".to_string(),
            ],
            max_batch_size: 256,
            max_sequence_length: 32768,
        }
    }

    fn supports_hardware(&self, hardware: &Hardware) -> bool {
        hardware.has_gpu()
    }

    async fn load_model(&mut self, _path: &Path) -> Result<ModelHandle> {
        // vLLM OpenAI server loads models on startup
        // We don't need to load them explicitly
        // Just return a dummy handle
        Ok(ModelHandle(0))
    }

    async fn unload_model(&mut self, _handle: ModelHandle) -> Result<()> {
        // vLLM manages model lifecycle
        Ok(())
    }

    async fn generate(&self, request: GenerateRequest) -> Result<GenerateResponse> {
        info!("Generating via vLLM OpenAI API: {}", request.model);

        // Convert to OpenAI completion request
        let completion_request = CompletionRequest {
            model: request.model.clone(),
            prompt: request.prompt.clone(),
            max_tokens: request.options.sampling.max_tokens,
            temperature: Some(request.options.sampling.temperature),
            top_p: Some(request.options.sampling.top_p),
            stream: Some(false),
            stop: None,
        };

        let response = self.client.create_completion(completion_request).await?;

        // Convert OpenAI response to our format
        let text = response
            .choices
            .first()
            .map(|c| c.text.clone())
            .unwrap_or_default();

        let stats = GenerationStats::new(
            response.usage.prompt_tokens,
            response.usage.completion_tokens,
        );

        Ok(GenerateResponse {
            id: request.id,
            model: request.model,
            text,
            tokens: Vec::new(),
            stats,
            finished: true,
            finish_reason: response
                .choices
                .first()
                .and_then(|c| c.finish_reason.clone()),
        })
    }

    async fn generate_stream(
        &self,
        request: GenerateRequest,
    ) -> Result<BoxStream<'static, Result<GenerateResponse>>> {
        info!("Streaming via vLLM OpenAI API: {}", request.model);

        let request_id = request.id;
        let model = request.model.clone();

        // Convert to OpenAI completion request with streaming
        let completion_request = CompletionRequest {
            model: request.model.clone(),
            prompt: request.prompt.clone(),
            max_tokens: request.options.sampling.max_tokens,
            temperature: Some(request.options.sampling.temperature),
            top_p: Some(request.options.sampling.top_p),
            stream: Some(true),
            stop: None,
        };

        let stream = self
            .client
            .create_completion_stream(completion_request)
            .await?;

        // Convert chunks to GenerateResponse
        let response_stream = stream.map(move |result| {
            result.map(|chunk| {
                let text = chunk
                    .choices
                    .first()
                    .map(|c| c.text.clone())
                    .unwrap_or_default();

                let finish_reason = chunk
                    .choices
                    .first()
                    .and_then(|c| c.finish_reason.clone());

                GenerateResponse {
                    id: request_id,
                    model: model.clone(),
                    text,
                    tokens: Vec::new(),
                    stats: GenerationStats::new(0, 0),
                    finished: finish_reason.is_some(),
                    finish_reason,
                }
            })
        });

        Ok(Box::pin(response_stream))
    }

    async fn health_check(&self) -> Result<bool> {
        self.client.health().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let engine = VllmOpenAIEngine::new("http://localhost:8100");
        assert_eq!(engine.base_url, "http://localhost:8100");
    }

    #[test]
    fn test_capabilities() {
        let engine = VllmOpenAIEngine::new("http://localhost:8100");
        let caps = engine.capabilities();

        assert!(caps.supports_continuous_batching);
        assert!(caps.supports_paged_attention);
        assert_eq!(caps.max_batch_size, 256);
    }
}
