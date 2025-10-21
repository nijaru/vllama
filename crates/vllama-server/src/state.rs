use dashmap::DashMap;
use vllama_engine::VllmOpenAIEngine;
use vllama_core::ModelHandle;
use tokio::sync::Mutex;
use std::sync::Arc;

#[derive(Clone)]
pub struct ServerState {
    pub engine: Arc<Mutex<VllmOpenAIEngine>>,
    pub loaded_models: Arc<DashMap<String, ModelHandle>>,
}

impl ServerState {
    pub fn new() -> crate::Result<Self> {
        let engine = VllmOpenAIEngine::new("http://127.0.0.1:8100");

        Ok(Self {
            engine: Arc::new(Mutex::new(engine)),
            loaded_models: Arc::new(DashMap::new()),
        })
    }
}

impl Default for ServerState {
    fn default() -> Self {
        Self::new().expect("Failed to create ServerState")
    }
}
