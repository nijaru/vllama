use dashmap::DashMap;
use hyperllama_engine::MaxEngine;
use hyperllama_core::ModelHandle;
use tokio::sync::Mutex;
use std::sync::Arc;

#[derive(Clone)]
pub struct ServerState {
    pub engine: Arc<Mutex<MaxEngine>>,
    pub loaded_models: Arc<DashMap<String, ModelHandle>>,
}

impl ServerState {
    pub fn new() -> crate::Result<Self> {
        let engine = MaxEngine::new()?;

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
