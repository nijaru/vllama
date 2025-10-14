use async_trait::async_trait;
use hyperllama_core::{GenerateRequest, GenerateResponse, Hardware, ModelHandle, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EngineType {
    Max,
    Vllm,
    LlamaCpp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineCapabilities {
    pub supports_continuous_batching: bool,
    pub supports_flash_attention: bool,
    pub supports_paged_attention: bool,
    pub supports_speculative_decoding: bool,
    pub supports_quantization: Vec<String>,
    pub max_batch_size: usize,
    pub max_sequence_length: usize,
}

impl Default for EngineCapabilities {
    fn default() -> Self {
        Self {
            supports_continuous_batching: false,
            supports_flash_attention: false,
            supports_paged_attention: false,
            supports_speculative_decoding: false,
            supports_quantization: Vec::new(),
            max_batch_size: 1,
            max_sequence_length: 4096,
        }
    }
}

#[async_trait]
pub trait InferenceEngine: Send + Sync {
    fn engine_type(&self) -> EngineType;

    fn capabilities(&self) -> EngineCapabilities;

    fn supports_hardware(&self, hardware: &Hardware) -> bool;

    async fn load_model(&mut self, path: &Path) -> Result<ModelHandle>;

    async fn unload_model(&mut self, handle: ModelHandle) -> Result<()>;

    async fn generate(&self, request: GenerateRequest) -> Result<GenerateResponse>;

    async fn generate_stream(
        &self,
        request: GenerateRequest,
    ) -> Result<futures::stream::BoxStream<'static, Result<GenerateResponse>>>;

    async fn health_check(&self) -> Result<bool>;
}
