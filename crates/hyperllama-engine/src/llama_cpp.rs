use crate::engine::{EngineCapabilities, EngineType, InferenceEngine};
use async_trait::async_trait;
use hyperllama_core::{GenerateRequest, GenerateResponse, Hardware, ModelHandle, Result, Error};
use std::path::Path;

pub struct LlamaCppEngine {
    capabilities: EngineCapabilities,
}

impl LlamaCppEngine {
    pub fn new() -> Result<Self> {
        Ok(Self {
            capabilities: EngineCapabilities {
                supports_continuous_batching: false,
                supports_flash_attention: false,
                supports_paged_attention: false,
                supports_speculative_decoding: false,
                supports_quantization: vec![
                    "q4_0".to_string(),
                    "q4_1".to_string(),
                    "q5_0".to_string(),
                    "q5_1".to_string(),
                    "q8_0".to_string(),
                ],
                max_batch_size: 512,
                max_sequence_length: 32768,
            },
        })
    }
}

impl Default for LlamaCppEngine {
    fn default() -> Self {
        Self::new().expect("Failed to create LlamaCppEngine")
    }
}

#[async_trait]
impl InferenceEngine for LlamaCppEngine {
    fn engine_type(&self) -> EngineType {
        EngineType::LlamaCpp
    }

    fn capabilities(&self) -> EngineCapabilities {
        self.capabilities.clone()
    }

    fn supports_hardware(&self, _hardware: &Hardware) -> bool {
        true
    }

    async fn load_model(&mut self, _path: &Path) -> Result<ModelHandle> {
        Err(Error::EngineNotAvailable(
            "llama.cpp integration not yet implemented".to_string(),
        ))
    }

    async fn unload_model(&mut self, _handle: ModelHandle) -> Result<()> {
        Err(Error::EngineNotAvailable(
            "llama.cpp integration not yet implemented".to_string(),
        ))
    }

    async fn generate(&self, _request: GenerateRequest) -> Result<GenerateResponse> {
        Err(Error::EngineNotAvailable(
            "llama.cpp integration not yet implemented".to_string(),
        ))
    }

    async fn generate_stream(
        &self,
        _request: GenerateRequest,
    ) -> Result<futures::stream::BoxStream<'static, Result<GenerateResponse>>> {
        Err(Error::EngineNotAvailable(
            "llama.cpp integration not yet implemented".to_string(),
        ))
    }

    async fn health_check(&self) -> Result<bool> {
        Ok(false)
    }
}
