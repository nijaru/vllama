use crate::engine::{EngineType, InferenceEngine};
use crate::{llama_cpp::LlamaCppEngine, max::MaxEngine, vllm::VllmEngine};
use hyperllama_core::{Hardware, Result};
use std::sync::Arc;
use tracing::{info, warn};

pub struct EngineOrchestrator {
    engines: Vec<Arc<dyn InferenceEngine>>,
    hardware: Hardware,
}

impl EngineOrchestrator {
    pub fn new(hardware: Hardware) -> Self {
        Self {
            engines: Vec::new(),
            hardware,
        }
    }

    pub async fn initialize(&mut self) -> Result<()> {
        info!("Initializing engine orchestrator");
        info!("Detected hardware: {:?}", self.hardware);

        if let Ok(vllm_engine) = VllmEngine::new() {
            if vllm_engine.supports_hardware(&self.hardware) {
                info!("vLLM Engine available and compatible");
                self.engines.push(Arc::new(vllm_engine));
            }
        }

        if let Ok(max_engine) = MaxEngine::new() {
            if max_engine.supports_hardware(&self.hardware) {
                info!("MAX Engine available and compatible (fallback)");
                self.engines.push(Arc::new(max_engine));
            }
        }

        if let Ok(llama_cpp_engine) = LlamaCppEngine::new() {
            info!("llama.cpp Engine available (fallback)");
            self.engines.push(Arc::new(llama_cpp_engine));
        }

        if self.engines.is_empty() {
            warn!("No inference engines available!");
        } else {
            info!("Initialized {} engine(s)", self.engines.len());
        }

        Ok(())
    }

    pub fn select_engine(&self) -> Option<Arc<dyn InferenceEngine>> {
        for engine in &self.engines {
            if engine.supports_hardware(&self.hardware) {
                let caps = engine.capabilities();
                info!(
                    "Selected {:?} engine (continuous_batching: {}, flash_attention: {})",
                    engine.engine_type(),
                    caps.supports_continuous_batching,
                    caps.supports_flash_attention
                );
                return Some(engine.clone());
            }
        }

        self.engines.first().cloned()
    }

    pub fn available_engines(&self) -> Vec<EngineType> {
        self.engines.iter().map(|e| e.engine_type()).collect()
    }

    pub fn hardware(&self) -> &Hardware {
        &self.hardware
    }
}
