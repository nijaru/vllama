use vllama_core::{
    Error, GenerateRequest, GenerateResponse, ModelHandle, RequestId, Result,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use tracing::{debug, error, info};

#[derive(Debug, Serialize)]
pub struct LoadModelRequest {
    pub model_path: String,
    pub max_length: usize,
}

#[derive(Debug, Deserialize)]
pub struct LoadModelResponse {
    pub model_id: String,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct GenerateRequestPayload {
    pub model_id: String,
    pub prompt: String,
    pub max_tokens: Option<usize>,
    pub temperature: f32,
    pub top_p: f32,
    pub stream: bool,
}

#[derive(Debug, Deserialize)]
pub struct GenerateResponsePayload {
    pub text: String,
    pub tokens_generated: usize,
    pub prompt_tokens: usize,
}

pub struct HttpEngineClient {
    client: reqwest::Client,
    service_url: String,
    engine_name: String,
    loaded_models: HashMap<u64, String>,
    next_handle: AtomicU64,
}

impl HttpEngineClient {
    pub fn new(service_url: impl Into<String>, engine_name: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            service_url: service_url.into(),
            engine_name: engine_name.into(),
            loaded_models: HashMap::new(),
            next_handle: AtomicU64::new(1),
        }
    }

    pub fn next_model_handle(&self) -> ModelHandle {
        ModelHandle(self.next_handle.fetch_add(1, Ordering::SeqCst))
    }

    pub fn get_model_id(&self, handle: ModelHandle) -> Option<String> {
        self.loaded_models.get(&handle.0).cloned()
    }

    pub async fn load_model(&mut self, path: &Path) -> Result<ModelHandle> {
        let model_path = path.to_string_lossy().to_string();
        info!("Loading model via {} service: {}", self.engine_name, model_path);

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

    pub async fn unload_model(&mut self, handle: ModelHandle) -> Result<()> {
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

    pub async fn generate(&self, request: GenerateRequest) -> Result<GenerateResponse> {
        let model_id = &request.model;
        debug!("Generating with {} model: {}", self.engine_name, model_id);

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

    pub async fn generate_stream(
        &self,
        request: GenerateRequest,
    ) -> Result<futures::stream::BoxStream<'static, Result<GenerateResponse>>> {
        use futures::stream::StreamExt;

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

    pub async fn health_check(&self, availability_field: &str) -> Result<bool> {
        let url = format!("{}/health", self.service_url);
        match self.client.get(&url).send().await {
            Ok(response) => {
                if let Ok(json) = response.json::<serde_json::Value>().await {
                    Ok(json.get(availability_field)
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false))
                } else {
                    Ok(false)
                }
            }
            Err(_) => Ok(false),
        }
    }
}
