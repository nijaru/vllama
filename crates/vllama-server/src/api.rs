use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response, sse::{Event, Sse}},
    Json,
};
use futures::stream::{self};
use vllama_core::{ChatMessage, ChatRole, GenerateRequest, GenerateOptions};
use vllama_engine::InferenceEngine;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::convert::Infallible;
use std::hash::{Hash, Hasher};
use std::time::Instant;
use tracing::{error, info};

use crate::state::ServerState;

fn messages_to_prompt(messages: &[ChatMessage]) -> String {
    messages
        .iter()
        .map(|msg| match msg.role {
            ChatRole::System => format!("System: {}", msg.content),
            ChatRole::User => format!("User: {}", msg.content),
            ChatRole::Assistant => format!("Assistant: {}", msg.content),
            ChatRole::Tool => format!("Tool: {}", msg.content),
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

#[derive(Debug, Deserialize)]
pub struct GenerateApiRequest {
    pub model: String,
    pub prompt: String,
    #[serde(default = "default_stream")]
    pub stream: bool,
    pub options: Option<GenerateOptionsApi>,
}

fn default_stream() -> bool {
    true
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GenerateOptionsApi {
    #[serde(default)]
    pub temperature: Option<f32>,
    #[serde(default)]
    pub top_p: Option<f32>,
    #[serde(default)]
    pub max_tokens: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct GenerateApiResponse {
    pub model: String,
    pub response: String,
    pub done: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_duration: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eval_count: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct TagsResponse {
    pub models: Vec<ModelInfo>,
}

#[derive(Debug, Serialize)]
pub struct ModelInfo {
    pub name: String,
    pub size: u64,
    pub digest: String,
}

#[derive(Debug, Deserialize)]
pub struct ChatApiRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    #[serde(default = "default_stream")]
    pub stream: bool,
    pub options: Option<GenerateOptionsApi>,
}

#[derive(Debug, Serialize)]
pub struct ChatApiResponse {
    pub model: String,
    pub message: ChatMessage,
    pub done: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_duration: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eval_count: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct ShowApiRequest {
    pub model: String,
    #[serde(default)]
    #[allow(dead_code)]
    pub verbose: bool,
}

#[derive(Debug, Serialize)]
pub struct ShowApiResponse {
    pub modelfile: String,
    pub parameters: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,
    pub details: ModelDetails,
}

#[derive(Debug, Serialize)]
pub struct ModelDetails {
    pub parent_model: String,
    pub format: String,
    pub family: String,
    pub parameter_size: String,
    pub quantization_level: String,
}

#[derive(Debug, Deserialize)]
pub struct PullApiRequest {
    #[serde(alias = "name")]
    pub model: String,
    #[serde(default = "default_stream")]
    pub stream: bool,
}

#[derive(Debug, Serialize)]
pub struct PullApiResponse {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub digest: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct OpenAIChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    #[serde(default)]
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
}

#[derive(Debug, Serialize)]
pub struct OpenAIChatResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<OpenAIChoice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<OpenAIUsage>,
}

#[derive(Debug, Serialize)]
pub struct OpenAIChoice {
    pub index: usize,
    pub message: ChatMessage,
    pub finish_reason: String,
}

#[derive(Debug, Serialize)]
pub struct OpenAIUsage {
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub total_tokens: usize,
}

#[derive(Debug, Serialize)]
pub struct OpenAIChatChunk {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<OpenAIChunkChoice>,
}

#[derive(Debug, Serialize)]
pub struct OpenAIChunkChoice {
    pub index: usize,
    pub delta: OpenAIDelta,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct OpenAIDelta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

pub async fn generate(
    State(state): State<ServerState>,
    Json(req): Json<GenerateApiRequest>,
) -> Response {
    info!("Generate request for model: {}", req.model);

    let mut gen_req = GenerateRequest::new(
        0,  // Request ID
        req.model.clone(),
        req.prompt.clone(),
    );

    if let Some(opts) = req.options {
        let mut gen_opts = GenerateOptions::default();
        if let Some(temp) = opts.temperature {
            gen_opts.sampling.temperature = temp;
        }
        if let Some(top_p) = opts.top_p {
            gen_opts.sampling.top_p = top_p;
        }
        if let Some(max_tokens) = opts.max_tokens {
            gen_opts.sampling.max_tokens = Some(max_tokens);
        }
        gen_req.options = gen_opts;
    }

    if req.stream {
        let engine = state.engine.lock().await;
        match engine.generate_stream(gen_req).await {
            Ok(stream) => {
                use futures::StreamExt;

                let event_stream = stream::unfold(
                    (stream, req.model.clone(), 0usize, false),
                    |(mut s, model, count, done)| async move {
                        if done {
                            return None;
                        }
                        match s.next().await {
                            Some(Ok(resp)) => {
                                let event = GenerateApiResponse {
                                    model: model.clone(),
                                    response: resp.text,
                                    done: false,
                                    total_duration: None,
                                    eval_count: None,
                                };
                                let json = serde_json::to_string(&event).unwrap();
                                Some((
                                    Ok::<_, Infallible>(Event::default().data(json)),
                                    (s, model, count + 1, false)
                                ))
                            }
                            Some(Err(e)) => {
                                error!("Stream error: {}", e);
                                None
                            }
                            None => {
                                let final_event = GenerateApiResponse {
                                    model,
                                    response: String::new(),
                                    done: true,
                                    total_duration: None,
                                    eval_count: Some(count),
                                };
                                let json = serde_json::to_string(&final_event).unwrap();
                                Some((Ok(Event::default().data(json)), (s, String::new(), count, true)))
                            }
                        }
                    }
                );

                Sse::new(event_stream).into_response()
            }
            Err(e) => {
                error!("Streaming generation failed: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                    "error": format!("Generation failed: {}", e)
                }))).into_response()
            }
        }
    } else {
        let start = Instant::now();
        let engine = state.engine.lock().await;
        match engine.generate(gen_req).await {
            Ok(resp) => {
                let duration = start.elapsed();
                Json(GenerateApiResponse {
                    model: req.model,
                    response: resp.text,
                    done: true,
                    total_duration: Some(duration.as_nanos() as u64),
                    eval_count: None,
                }).into_response()
            }
            Err(e) => {
                error!("Generation failed: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                    "error": format!("Generation failed: {}", e)
                }))).into_response()
            }
        }
    }
}

pub async fn tags(State(state): State<ServerState>) -> Json<TagsResponse> {
    let mut models = Vec::new();

    for entry in state.loaded_models.iter() {
        let model_name = entry.key().clone();

        let size = match std::fs::metadata(&model_name) {
            Ok(metadata) => metadata.len(),
            Err(_) => 0,
        };

        let mut hasher = DefaultHasher::new();
        model_name.hash(&mut hasher);
        let digest = format!("{:x}", hasher.finish());

        models.push(ModelInfo {
            name: model_name,
            size,
            digest,
        });
    }

    Json(TagsResponse { models })
}

pub async fn health() -> &'static str {
    "OK"
}

pub async fn pull(
    State(state): State<ServerState>,
    Json(req): Json<PullApiRequest>,
) -> Response {
    info!("Pull request for model: {}", req.model);

    if state.loaded_models.contains_key(&req.model) {
        return Json(PullApiResponse {
            status: "success".to_string(),
            digest: None,
            total: None,
            completed: None,
        }).into_response();
    }

    use vllama_core::{ModelDownloader, DownloadProgress};
    use tokio::sync::mpsc;

    let downloader = match ModelDownloader::new() {
        Ok(d) => d,
        Err(e) => {
            error!("Failed to create downloader: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": format!("Failed to initialize downloader: {}", e)
            }))).into_response();
        }
    };

    if req.stream {
        let model_name = req.model.clone();
        let engine_clone = state.engine.clone();
        let loaded_models_clone = state.loaded_models.clone();

        let (tx, rx) = mpsc::channel::<DownloadProgress>(100);

        tokio::spawn(async move {
            let result = downloader.download_model(
                &model_name,
                |progress| {
                    let _ = tx.try_send(progress);
                }
            ).await;

            if let Ok(model_path) = result {
                let mut engine = engine_clone.lock().await;
                if let Ok(handle) = engine.load_model(&model_path).await {
                    loaded_models_clone.insert(model_name, handle);
                }
            }
        });

        let event_stream = stream::unfold(
            rx,
            |mut receiver| async move {
                match receiver.recv().await {
                    Some(progress) => {
                        let event = PullApiResponse {
                            status: progress.status.clone(),
                            digest: None,
                            total: if progress.total > 0 { Some(progress.total) } else { None },
                            completed: if progress.downloaded > 0 { Some(progress.downloaded) } else { None },
                        };
                        Some((
                            Ok::<_, Infallible>(Event::default().data(serde_json::to_string(&event).unwrap())),
                            receiver
                        ))
                    }
                    None => {
                        let final_event = PullApiResponse {
                            status: "success".to_string(),
                            digest: None,
                            total: None,
                            completed: None,
                        };
                        Some((
                            Ok(Event::default().data(serde_json::to_string(&final_event).unwrap())),
                            receiver
                        ))
                    }
                }
            }
        );

        Sse::new(event_stream).into_response()
    } else {
        match downloader.download_model(&req.model, |_| {}).await {
            Ok(model_path) => {
                let mut engine = state.engine.lock().await;
                match engine.load_model(&model_path).await {
                    Ok(handle) => {
                        state.loaded_models.insert(req.model.clone(), handle);
                        Json(PullApiResponse {
                            status: "success".to_string(),
                            digest: None,
                            total: None,
                            completed: None,
                        }).into_response()
                    }
                    Err(e) => {
                        error!("Failed to load model: {}", e);
                        (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                            "error": format!("Downloaded model successfully but failed to load it: {}. This may be due to MAX Engine limitations (only supports whitelisted models).", e)
                        }))).into_response()
                    }
                }
            }
            Err(e) => {
                error!("Failed to download model: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                    "error": format!("Failed to download model from HuggingFace: {}. Check that the model repo and file exist. Example: 'bartowski/Llama-3.2-1B-Instruct-GGUF'", e)
                }))).into_response()
            }
        }
    }
}

pub async fn show(
    State(_state): State<ServerState>,
    Json(req): Json<ShowApiRequest>,
) -> Response {
    info!("Show request for model: {}", req.model);

    #[derive(Debug, Deserialize)]
    struct VllmModelsResponse {
        data: Vec<VllmModelInfo>,
    }

    #[derive(Debug, Deserialize)]
    struct VllmModelInfo {
        id: String,
        #[serde(default)]
        #[allow(dead_code)]
        max_model_len: Option<u64>,
    }

    let client = reqwest::Client::new();
    let models_response = match client.get("http://127.0.0.1:8100/v1/models").send().await {
        Ok(response) => match response.json::<VllmModelsResponse>().await {
            Ok(data) => data,
            Err(e) => {
                error!("Failed to parse vLLM models response: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                    "error": "Failed to query vLLM server"
                }))).into_response();
            }
        },
        Err(e) => {
            error!("Failed to query vLLM models: {}", e);
            return (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({
                "error": "vLLM server not available"
            }))).into_response();
        }
    };

    let model_info = models_response.data.iter().find(|m| m.id == req.model);
    if model_info.is_none() {
        return (StatusCode::NOT_FOUND, Json(serde_json::json!({
            "error": format!("Model '{}' not found in vLLM server", req.model)
        }))).into_response();
    }

    let model_name = &req.model;

    let (family, parameter_size) = if model_name.contains("llama") || model_name.contains("Llama") {
        let size = if model_name.contains("70B") || model_name.contains("70b") {
            "70B"
        } else if model_name.contains("8B") || model_name.contains("8b") {
            "8B"
        } else if model_name.contains("3B") || model_name.contains("3b") {
            "3B"
        } else if model_name.contains("1B") || model_name.contains("1b") {
            "1B"
        } else {
            "unknown"
        };
        ("llama", size)
    } else if model_name.contains("opt") {
        let size = if model_name.contains("125m") {
            "125M"
        } else if model_name.contains("350m") {
            "350M"
        } else {
            "unknown"
        };
        ("opt", size)
    } else if model_name.contains("qwen") || model_name.contains("Qwen") {
        let size = if model_name.contains("1.5B") || model_name.contains("1.5b") {
            "1.5B"
        } else if model_name.contains("7B") || model_name.contains("7b") {
            "7B"
        } else {
            "unknown"
        };
        ("qwen", size)
    } else {
        ("unknown", "unknown")
    };

    let response = ShowApiResponse {
        modelfile: format!("# Modelfile for {}\n# Loaded via vLLama + vLLM", model_name),
        parameters: "temperature 0.7\ntop_p 0.9\nrepetition_penalty 1.0".to_string(),
        template: Some("{{ .System }}\n{{ .Prompt }}".to_string()),
        details: ModelDetails {
            parent_model: model_name.clone(),
            format: "safetensors".to_string(),
            family: family.to_string(),
            parameter_size: parameter_size.to_string(),
            quantization_level: "none".to_string(),
        },
    };

    Json(response).into_response()
}

pub async fn openai_chat_completions(
    State(state): State<ServerState>,
    Json(req): Json<OpenAIChatRequest>,
) -> Response {
    info!("OpenAI chat completions request for model: {}", req.model);

    let prompt = messages_to_prompt(&req.messages);

    let mut gen_req = GenerateRequest::new(
        0,  // Request ID
        req.model.clone(),
        prompt,
    );

    let mut gen_opts = GenerateOptions::default();
    if let Some(temp) = req.temperature {
        gen_opts.sampling.temperature = temp;
    }
    if let Some(max_tokens) = req.max_tokens {
        gen_opts.sampling.max_tokens = Some(max_tokens);
    }
    gen_req.options = gen_opts;

    let request_id = format!("chatcmpl-{:x}", std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos());
    let created = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    if req.stream {
        let engine = state.engine.lock().await;
        match engine.generate_stream(gen_req).await {
            Ok(stream) => {
                use futures::StreamExt;

                let event_stream = stream::unfold(
                    (stream, req.model.clone(), request_id.clone(), created, 0usize, false),
                    |(mut s, model, id, timestamp, count, done)| async move {
                        if done {
                            return None;
                        }
                        match s.next().await {
                            Some(Ok(resp)) => {
                                let chunk = OpenAIChatChunk {
                                    id: id.clone(),
                                    object: "chat.completion.chunk".to_string(),
                                    created: timestamp,
                                    model: model.clone(),
                                    choices: vec![OpenAIChunkChoice {
                                        index: 0,
                                        delta: OpenAIDelta {
                                            role: None,
                                            content: Some(resp.text),
                                        },
                                        finish_reason: None,
                                    }],
                                };
                                let json = serde_json::to_string(&chunk).unwrap();
                                Some((
                                    Ok::<_, Infallible>(Event::default().data(json)),
                                    (s, model, id, timestamp, count + 1, false)
                                ))
                            }
                            Some(Err(e)) => {
                                error!("Stream error: {}", e);
                                None
                            }
                            None => {
                                let final_chunk = OpenAIChatChunk {
                                    id,
                                    object: "chat.completion.chunk".to_string(),
                                    created: timestamp,
                                    model,
                                    choices: vec![OpenAIChunkChoice {
                                        index: 0,
                                        delta: OpenAIDelta {
                                            role: None,
                                            content: None,
                                        },
                                        finish_reason: Some("stop".to_string()),
                                    }],
                                };
                                let json = serde_json::to_string(&final_chunk).unwrap();
                                Some((Ok(Event::default().data(json)), (s, String::new(), String::new(), timestamp, count, true)))
                            }
                        }
                    }
                );

                Sse::new(event_stream).into_response()
            }
            Err(e) => {
                error!("Streaming chat failed: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                    "error": {
                        "message": format!("Generation failed: {}", e),
                        "type": "server_error"
                    }
                }))).into_response()
            }
        }
    } else {
        let engine = state.engine.lock().await;
        match engine.generate(gen_req).await {
            Ok(resp) => {
                let response = OpenAIChatResponse {
                    id: request_id,
                    object: "chat.completion".to_string(),
                    created,
                    model: req.model,
                    choices: vec![OpenAIChoice {
                        index: 0,
                        message: ChatMessage::assistant(resp.text),
                        finish_reason: "stop".to_string(),
                    }],
                    usage: Some(OpenAIUsage {
                        prompt_tokens: 0,
                        completion_tokens: 0,
                        total_tokens: 0,
                    }),
                };
                Json(response).into_response()
            }
            Err(e) => {
                error!("Chat failed: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                    "error": {
                        "message": format!("Generation failed: {}", e),
                        "type": "server_error"
                    }
                }))).into_response()
            }
        }
    }
}

pub async fn chat(
    State(state): State<ServerState>,
    Json(req): Json<ChatApiRequest>,
) -> Response {
    info!("Chat request for model: {}", req.model);

    let mut gen_opts = GenerateOptions::default();
    if let Some(opts) = req.options {
        if let Some(temp) = opts.temperature {
            gen_opts.sampling.temperature = temp;
        }
        if let Some(top_p) = opts.top_p {
            gen_opts.sampling.top_p = top_p;
        }
        if let Some(max_tokens) = opts.max_tokens {
            gen_opts.sampling.max_tokens = Some(max_tokens);
        }
    }

    if req.stream {
        // Streaming still uses prompt-based approach
        let prompt = messages_to_prompt(&req.messages);
        let mut gen_req = GenerateRequest::new(0, req.model.clone(), prompt);
        gen_req.options = gen_opts;
        let engine = state.engine.lock().await;
        match engine.generate_stream(gen_req).await {
            Ok(stream) => {
                use futures::StreamExt;

                let event_stream = stream::unfold(
                    (stream, req.model.clone(), String::new(), 0usize, false),
                    |(mut s, model, mut accumulated, count, done)| async move {
                        if done {
                            return None;
                        }
                        match s.next().await {
                            Some(Ok(resp)) => {
                                accumulated.push_str(&resp.text);
                                let msg = ChatMessage::assistant(resp.text);
                                let event = ChatApiResponse {
                                    model: model.clone(),
                                    message: msg,
                                    done: false,
                                    total_duration: None,
                                    eval_count: None,
                                };
                                let json = serde_json::to_string(&event).unwrap();
                                Some((
                                    Ok::<_, Infallible>(Event::default().data(json)),
                                    (s, model, accumulated, count + 1, false)
                                ))
                            }
                            Some(Err(e)) => {
                                error!("Stream error: {}", e);
                                None
                            }
                            None => {
                                let msg = ChatMessage::assistant("");
                                let final_event = ChatApiResponse {
                                    model,
                                    message: msg,
                                    done: true,
                                    total_duration: None,
                                    eval_count: Some(count),
                                };
                                let json = serde_json::to_string(&final_event).unwrap();
                                Some((Ok(Event::default().data(json)), (s, String::new(), accumulated, count, true)))
                            }
                        }
                    }
                );

                Sse::new(event_stream).into_response()
            }
            Err(e) => {
                error!("Streaming chat failed: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                    "error": format!("Chat failed: {}", e)
                }))).into_response()
            }
        }
    } else {
        // Non-streaming: use proper chat completion endpoint
        let start = Instant::now();
        let engine = state.engine.lock().await;
        match engine.generate_chat_completion(req.model.clone(), req.messages.clone(), gen_opts).await {
            Ok(chat_response) => {
                let duration = start.elapsed();
                let message = chat_response.choices
                    .first()
                    .map(|choice| choice.message.clone())
                    .unwrap_or_else(|| vllama_core::openai::ChatMessage {
                        role: "assistant".to_string(),
                        content: String::new(),
                    });

                let msg = ChatMessage {
                    role: vllama_core::ChatRole::Assistant,
                    content: message.content,
                    images: None,
                };

                Json(ChatApiResponse {
                    model: req.model,
                    message: msg,
                    done: true,
                    total_duration: Some(duration.as_nanos() as u64),
                    eval_count: Some(chat_response.usage.completion_tokens),
                }).into_response()
            }
            Err(e) => {
                error!("Chat failed: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                    "error": format!("Chat failed: {}", e)
                }))).into_response()
            }
        }
    }
}


#[derive(Debug, Serialize)]
pub struct VersionResponse {
    pub version: String,
}

pub async fn version() -> Json<VersionResponse> {
    info!("Version request");
    Json(VersionResponse {
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

#[derive(Debug, Serialize)]
pub struct ProcessInfo {
    pub name: String,
    pub model: String,
    pub size: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub digest: Option<String>,
    pub details: ModelDetails,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_vram: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct PsResponse {
    pub models: Vec<ProcessInfo>,
}

pub async fn ps(State(_state): State<ServerState>) -> Response {
    info!("Process status request");

    #[derive(Debug, Deserialize)]
    struct VllmModelsResponse {
        data: Vec<VllmModelInfo>,
    }

    #[derive(Debug, Deserialize)]
    struct VllmModelInfo {
        id: String,
        #[allow(dead_code)]
        created: u64,
        #[serde(default)]
        max_model_len: Option<u64>,
    }

    let client = reqwest::Client::new();
    match client.get("http://127.0.0.1:8100/v1/models").send().await {
        Ok(response) => {
            match response.json::<VllmModelsResponse>().await {
                Ok(vllm_models) => {
                    let models = vllm_models.data.into_iter().map(|m| {
                        let model_name = m.id.clone();

                        let (family, parameter_size) = if model_name.contains("llama") || model_name.contains("Llama") {
                            let size = if model_name.contains("70B") || model_name.contains("70b") {
                                "70B"
                            } else if model_name.contains("8B") || model_name.contains("8b") {
                                "8B"
                            } else if model_name.contains("3B") || model_name.contains("3b") {
                                "3B"
                            } else if model_name.contains("1B") || model_name.contains("1b") {
                                "1B"
                            } else {
                                "unknown"
                            };
                            ("llama", size)
                        } else if model_name.contains("opt") {
                            let size = if model_name.contains("125m") {
                                "125M"
                            } else {
                                "unknown"
                            };
                            ("opt", size)
                        } else {
                            ("unknown", "unknown")
                        };

                        ProcessInfo {
                            name: model_name.clone(),
                            model: model_name.clone(),
                            size: 0,
                            digest: None,
                            details: ModelDetails {
                                parent_model: model_name,
                                format: "safetensors".to_string(),
                                family: family.to_string(),
                                parameter_size: parameter_size.to_string(),
                                quantization_level: "none".to_string(),
                            },
                            expires_at: None,
                            size_vram: m.max_model_len,
                        }
                    }).collect();

                    Json(PsResponse { models }).into_response()
                }
                Err(e) => {
                    error!("Failed to parse vLLM models response: {}", e);
                    Json(PsResponse { models: vec![] }).into_response()
                }
            }
        }
        Err(e) => {
            error!("Failed to query vLLM models: {}", e);
            Json(PsResponse { models: vec![] }).into_response()
        }
    }
}
