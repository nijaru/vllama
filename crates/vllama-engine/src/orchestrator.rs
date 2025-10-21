use crate::engine::{EngineType, InferenceEngine};
use crate::vllm_openai::VllmOpenAIEngine;
use vllama_core::{Hardware, Result};
use std::sync::Arc;
use tracing::info;

pub struct EngineOrchestrator {
    engine: Arc<dyn InferenceEngine>,
    hardware: Hardware,
}

impl EngineOrchestrator {
    pub fn new(hardware: Hardware) -> Self {
        let engine = VllmOpenAIEngine::new("http://127.0.0.1:8100");

        Self {
            engine: Arc::new(engine),
            hardware,
        }
    }

    pub async fn initialize(&mut self) -> Result<()> {
        info!("Initializing vLLM OpenAI engine");
        info!("Detected hardware: {:?}", self.hardware);

        let caps = self.engine.capabilities();
        info!(
            "vLLM engine ready (continuous_batching: {}, flash_attention: {})",
            caps.supports_continuous_batching,
            caps.supports_flash_attention
        );

        Ok(())
    }

    pub fn select_engine(&self) -> Option<Arc<dyn InferenceEngine>> {
        Some(self.engine.clone())
    }

    pub fn available_engines(&self) -> Vec<EngineType> {
        vec![self.engine.engine_type()]
    }

    pub fn hardware(&self) -> &Hardware {
        &self.hardware
    }
}
