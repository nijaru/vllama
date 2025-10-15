use crate::engine::{EngineCapabilities, EngineType, InferenceEngine};
use async_trait::async_trait;
use hyperllama_core::{
    Error, GenerateRequest, GenerateResponse, Hardware, ModelHandle, RequestId, Result,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use tracing::{debug, error, info};

#[derive(Debug, Serialize)]
struct LoadModelRequest {
    model_path: String,
    max_length: usize,
}

#[derive(Debug, Deserialize)]
struct LoadModelResponse {
    model_id: String,
    status: String,
}

#[derive(Debug, Serialize)]
struct GenerateRequestPayload {
    model_id: String,
    prompt: String,
    max_tokens: Option<usize>,
    temperature: f32,
    top_p: f32,
    stream: bool,
}

#[derive(Debug, Deserialize)]
struct GenerateResponsePayload {
    text: String,
    tokens_generated: usize,
    prompt_tokens: usize,
}

#[derive(Debug, Deserialize)]
struct HealthResponse {
    status: String,
    max_available: bool,
}

pub struct MaxEngine {
    capabilities: EngineCapabilities,
    client: reqwest::Client,
    service_url: String,
    loaded_models: HashMap<u64, String>,
    next_handle: AtomicU64,
}

impl MaxEngine {
    pub fn new() -> Result<Self> {
        Self::with_service_url("http://127.0.0.1:8100")
    }

    pub fn with_service_url(url: impl Into<String>) -> Result<Self> {
        Ok(Self {
            capabilities: EngineCapabilities {
                supports_continuous_batching: true,
                supports_flash_attention: true,
                supports_paged_attention: true,
                supports_speculative_decoding: false,
                supports_quantization: vec!["int8".to_string(), "int4".to_string()],
                max_batch_size: 128,
                max_sequence_length: 32768,
            },
            client: reqwest::Client::new(),
            service_url: url.into(),
            loaded_models: HashMap::new(),
            next_handle: AtomicU64::new(1),
        })
    }

    fn next_model_handle(&self) -> ModelHandle {
        ModelHandle(self.next_handle.fetch_add(1, Ordering::SeqCst))
    }

    pub fn get_model_id(&self, handle: ModelHandle) -> Option<String> {
        self.loaded_models.get(&handle.0).cloned()
    }
}

impl Default for MaxEngine {
    fn default() -> Self {
        Self::new().expect("Failed to create MaxEngine")
    }
}

#[async_trait]
impl InferenceEngine for MaxEngine {
    fn engine_type(&self) -> EngineType {
        EngineType::Max
    }

    fn capabilities(&self) -> EngineCapabilities {
        self.capabilities.clone()
    }

    fn supports_hardware(&self, hardware: &Hardware) -> bool {
        hardware.has_gpu() || hardware.cpu_cores >= 4
    }

    async fn load_model(&mut self, path: &Path) -> Result<ModelHandle> {
        let model_path = path.to_string_lossy().to_string();
        info!("Loading model via MAX Engine service: {}", model_path);

        let request = LoadModelRequest {
            model_path: model_path.clone(),
            max_length: 32768,
        };

        let url = format!("{}/models/load", self.service_url);
        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                error!("Failed to load model: {}", e);
                Error::ModelLoadFailed(format!("HTTP request failed: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!("Model load failed with status {}: {}", status, error_text);
            return Err(Error::ModelLoadFailed(format!(
                "Status {}: {}",
                status, error_text
            )));
        }

        let load_response: LoadModelResponse = response.json().await.map_err(|e| {
            error!("Failed to parse response: {}", e);
            Error::ModelLoadFailed(format!("Failed to parse response: {}", e))
        })?;

        let handle = self.next_model_handle();
        self.loaded_models
            .insert(handle.0, load_response.model_id.clone());

        info!(
            "Model loaded successfully: {} (handle: {})",
            load_response.model_id, handle.0
        );
        Ok(handle)
    }

    async fn unload_model(&mut self, handle: ModelHandle) -> Result<()> {
        let model_id = self
            .loaded_models
            .get(&handle.0)
            .ok_or_else(|| Error::ModelNotFound(format!("Handle {} not found", handle.0)))?
            .clone();

        info!("Unloading model: {} (handle: {})", model_id, handle.0);

        let url = format!("{}/models/unload?model_id={}", self.service_url, model_id);
        let response = self.client.post(&url).send().await.map_err(|e| {
            error!("Failed to unload model: {}", e);
            Error::InferenceFailed(format!("HTTP request failed: {}", e))
        })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!("Model unload failed with status {}: {}", status, error_text);
            return Err(Error::InferenceFailed(format!(
                "Status {}: {}",
                status, error_text
            )));
        }

        self.loaded_models.remove(&handle.0);
        info!("Model unloaded: {}", model_id);
        Ok(())
    }

    async fn generate(&self, request: GenerateRequest) -> Result<GenerateResponse> {
        let model_id = &request.model;
        debug!("Generating with MAX Engine model: {}", model_id);

        let payload = GenerateRequestPayload {
            model_id: model_id.clone(),
            prompt: request.prompt.clone(),
            max_tokens: request.options.sampling.max_tokens,
            temperature: request.options.sampling.temperature,
            top_p: request.options.sampling.top_p,
            stream: false,
        };

        let url = format!("{}/generate", self.service_url);
        let response = self
            .client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| {
                error!("Generation request failed: {}", e);
                Error::InferenceFailed(format!("HTTP request failed: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!("Generation failed with status {}: {}", status, error_text);
            return Err(Error::InferenceFailed(format!(
                "Status {}: {}",
                status, error_text
            )));
        }

        let gen_response: GenerateResponsePayload = response.json().await.map_err(|e| {
            error!("Failed to parse generation response: {}", e);
            Error::InferenceFailed(format!("Failed to parse response: {}", e))
        })?;

        debug!("Generated {} tokens", gen_response.tokens_generated);

        Ok(GenerateResponse::new(
            RequestId(request.id.0),
            model_id.clone(),
        )
        .with_text(gen_response.text))
    }

    async fn generate_stream(
        &self,
        request: GenerateRequest,
    ) -> Result<futures::stream::BoxStream<'static, Result<GenerateResponse>>> {
        use futures::stream::{self, StreamExt};

        let model_id = request.model.clone();
        let payload = GenerateRequestPayload {
            model_id: model_id.clone(),
            prompt: request.prompt.clone(),
            max_tokens: request.options.sampling.max_tokens,
            temperature: request.options.sampling.temperature,
            top_p: request.options.sampling.top_p,
            stream: true,
        };

        let url = format!("{}/generate", self.service_url);
        let response = self
            .client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| {
                error!("Streaming request failed: {}", e);
                Error::InferenceFailed(format!("HTTP request failed: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!("Streaming failed with status {}: {}", status, error_text);
            return Err(Error::InferenceFailed(format!(
                "Status {}: {}",
                status, error_text
            )));
        }

        let bytes_stream = response.bytes_stream();
        let request_id = request.id;

        let result_stream = bytes_stream
            .map(move |chunk_result| {
                let chunk = chunk_result.map_err(|e| Error::InferenceFailed(e.to_string()))?;
                let text = String::from_utf8_lossy(&chunk).to_string();

                for line in text.lines() {
                    if let Some(data) = line.strip_prefix("data: ") {
                        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(data) {
                            if let Some(text_chunk) = parsed.get("text").and_then(|v| v.as_str()) {
                                return Ok(
                                    GenerateResponse::new(RequestId(request_id.0), model_id.clone())
                                        .with_text(text_chunk.to_string()),
                                );
                            }
                        }
                    }
                }

                Ok(GenerateResponse::new(RequestId(request_id.0), model_id.clone())
                    .with_text(String::new()))
            })
            .boxed();

        Ok(result_stream)
    }

    async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/health", self.service_url);
        match self.client.get(&url).send().await {
            Ok(response) => {
                if let Ok(health) = response.json::<HealthResponse>().await {
                    Ok(health.max_available)
                } else {
                    Ok(false)
                }
            }
            Err(_) => Ok(false),
        }
    }
}
