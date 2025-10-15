use crate::engine::{EngineCapabilities, EngineType, InferenceEngine};
use crate::http_engine::HttpEngineClient;
use async_trait::async_trait;
use vllama_core::{GenerateRequest, GenerateResponse, Hardware, ModelHandle, Result};
use std::path::Path;

pub struct VllmEngine {
    capabilities: EngineCapabilities,
    client: HttpEngineClient,
}

impl VllmEngine {
    pub fn new() -> Result<Self> {
        Self::with_service_url("http://127.0.0.1:8100")
    }

    pub fn with_service_url(url: impl Into<String>) -> Result<Self> {
        Ok(Self {
            capabilities: EngineCapabilities {
                supports_continuous_batching: true,
                supports_flash_attention: true,
                supports_paged_attention: true,
                supports_speculative_decoding: true,
                supports_quantization: vec![
                    "awq".to_string(),
                    "gptq".to_string(),
                    "squeezellm".to_string(),
                    "gguf".to_string(),
                ],
                max_batch_size: 256,
                max_sequence_length: 32768,
            },
            client: HttpEngineClient::new(url, "vLLM"),
        })
    }

    pub fn get_model_id(&self, handle: ModelHandle) -> Option<String> {
        self.client.get_model_id(handle)
    }
}

impl Default for VllmEngine {
    fn default() -> Self {
        Self::new().expect("Failed to create VllmEngine")
    }
}

#[async_trait]
impl InferenceEngine for VllmEngine {
    fn engine_type(&self) -> EngineType {
        EngineType::Vllm
    }

    fn capabilities(&self) -> EngineCapabilities {
        self.capabilities.clone()
    }

    fn supports_hardware(&self, hardware: &Hardware) -> bool {
        hardware.has_gpu() || hardware.cpu_cores >= 4
    }

    async fn load_model(&mut self, path: &Path) -> Result<ModelHandle> {
        self.client.load_model(path).await
    }

    async fn unload_model(&mut self, handle: ModelHandle) -> Result<()> {
        self.client.unload_model(handle).await
    }

    async fn generate(&self, request: GenerateRequest) -> Result<GenerateResponse> {
        self.client.generate(request).await
    }

    async fn generate_stream(
        &self,
        request: GenerateRequest,
    ) -> Result<futures::stream::BoxStream<'static, Result<GenerateResponse>>> {
        self.client.generate_stream(request).await
    }

    async fn health_check(&self) -> Result<bool> {
        self.client.health_check("vllm_available").await
    }
}
