use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response, sse::{Event, Sse}},
    Json,
};
use futures::stream::{self, Stream};
use hyperllama_core::{ChatMessage, ChatRequest, ChatTemplate, GenerateRequest, GenerateOptions};
use hyperllama_engine::InferenceEngine;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::convert::Infallible;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::time::Instant;
use tracing::{error, info};

use crate::state::ServerState;

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

    let model_path = PathBuf::from(&req.model);

    let handle = match state.loaded_models.get(&req.model) {
        Some(h) => *h,
        None => {
            info!("Loading model: {}", req.model);
            let mut engine = state.engine.lock().await;

            match engine.load_model(&model_path).await {
                Ok(h) => {
                    state.loaded_models.insert(req.model.clone(), h);
                    h
                }
                Err(e) => {
                    error!("Failed to load model: {}", e);
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                        "error": format!("Failed to load model: {}", e)
                    }))).into_response();
                }
            }
        }
    };

    let model_id = {
        let engine = state.engine.lock().await;
        match engine.get_model_id(handle) {
            Some(id) => id,
            None => {
                return (StatusCode::NOT_FOUND, Json(serde_json::json!({
                    "error": "Model not found"
                }))).into_response();
            }
        }
    };

    let mut gen_req = GenerateRequest::new(
        handle.0,
        model_id,
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
            Ok(mut stream) => {
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

    let model_path = PathBuf::from(&req.model);

    if req.stream {
        let model_name = req.model.clone();
        let engine_clone = state.engine.clone();
        let loaded_models_clone = state.loaded_models.clone();

        let event_stream = stream::unfold(
            (0, false),
            move |(step, done)| {
                let model = model_name.clone();
                let path = model_path.clone();
                let eng = engine_clone.clone();
                let loaded = loaded_models_clone.clone();

                async move {
                    if done {
                        return None;
                    }

                    match step {
                        0 => {
                            let event = PullApiResponse {
                                status: format!("pulling {}", model),
                                digest: None,
                                total: None,
                                completed: None,
                            };
                            Some((
                                Ok::<_, Infallible>(Event::default().data(serde_json::to_string(&event).unwrap())),
                                (1, false)
                            ))
                        }
                        1 => {
                            let mut engine = eng.lock().await;
                            match engine.load_model(&path).await {
                                Ok(handle) => {
                                    loaded.insert(model, handle);
                                    let event = PullApiResponse {
                                        status: "success".to_string(),
                                        digest: None,
                                        total: None,
                                        completed: None,
                                    };
                                    Some((
                                        Ok(Event::default().data(serde_json::to_string(&event).unwrap())),
                                        (2, true)
                                    ))
                                }
                                Err(e) => {
                                    error!("Failed to pull model: {}", e);
                                    let event = serde_json::json!({
                                        "error": format!("Failed to pull model: {}", e)
                                    });
                                    Some((
                                        Ok(Event::default().data(serde_json::to_string(&event).unwrap())),
                                        (2, true)
                                    ))
                                }
                            }
                        }
                        _ => None,
                    }
                }
            }
        );

        Sse::new(event_stream).into_response()
    } else {
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
                error!("Failed to pull model: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                    "error": format!("Failed to pull model: {}", e)
                }))).into_response()
            }
        }
    }
}

pub async fn show(
    State(state): State<ServerState>,
    Json(req): Json<ShowApiRequest>,
) -> Response {
    info!("Show request for model: {}", req.model);

    if !state.loaded_models.contains_key(&req.model) {
        return (StatusCode::NOT_FOUND, Json(serde_json::json!({
            "error": format!("Model '{}' not found. Load it first by making a generate or chat request.", req.model)
        }))).into_response();
    }

    let model_name = &req.model;

    let (family, parameter_size, format) = if model_name.contains("Llama") || model_name.contains("llama") {
        let size = if model_name.contains("70B") {
            "70B"
        } else if model_name.contains("8B") {
            "8B"
        } else if model_name.contains("3B") {
            "3B"
        } else {
            "unknown"
        };
        ("llama", size, "gguf")
    } else {
        ("unknown", "unknown", "gguf")
    };

    let response = ShowApiResponse {
        modelfile: format!("# Modelfile for {}\n# Loaded via HyperLlama", model_name),
        parameters: "temperature 0.7\ntop_p 0.9\nrepetition_penalty 1.0".to_string(),
        template: Some("{{ .System }}\n{{ .Prompt }}".to_string()),
        details: ModelDetails {
            parent_model: model_name.clone(),
            format: format.to_string(),
            family: family.to_string(),
            parameter_size: parameter_size.to_string(),
            quantization_level: "Q4_K_M".to_string(),
        },
    };

    Json(response).into_response()
}

pub async fn openai_chat_completions(
    State(state): State<ServerState>,
    Json(req): Json<OpenAIChatRequest>,
) -> Response {
    info!("OpenAI chat completions request for model: {}", req.model);

    let model_path = PathBuf::from(&req.model);

    let handle = match state.loaded_models.get(&req.model) {
        Some(h) => *h,
        None => {
            info!("Loading model: {}", req.model);
            let mut engine = state.engine.lock().await;

            match engine.load_model(&model_path).await {
                Ok(h) => {
                    state.loaded_models.insert(req.model.clone(), h);
                    h
                }
                Err(e) => {
                    error!("Failed to load model: {}", e);
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                        "error": {
                            "message": format!("Failed to load model: {}", e),
                            "type": "server_error"
                        }
                    }))).into_response();
                }
            }
        }
    };

    let model_id = {
        let engine = state.engine.lock().await;
        match engine.get_model_id(handle) {
            Some(id) => id,
            None => {
                return (StatusCode::NOT_FOUND, Json(serde_json::json!({
                    "error": {
                        "message": "Model not found",
                        "type": "invalid_request_error"
                    }
                }))).into_response();
            }
        }
    };

    let prompt = if req.model.to_lowercase().contains("llama") {
        hyperllama_core::Llama3Template.apply(&req.messages)
    } else {
        hyperllama_core::SimpleChatTemplate.apply(&req.messages)
    };

    let mut gen_req = GenerateRequest::new(
        handle.0,
        model_id,
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
            Ok(mut stream) => {
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

    let model_path = PathBuf::from(&req.model);

    let handle = match state.loaded_models.get(&req.model) {
        Some(h) => *h,
        None => {
            info!("Loading model: {}", req.model);
            let mut engine = state.engine.lock().await;

            match engine.load_model(&model_path).await {
                Ok(h) => {
                    state.loaded_models.insert(req.model.clone(), h);
                    h
                }
                Err(e) => {
                    error!("Failed to load model: {}", e);
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                        "error": format!("Failed to load model: {}", e)
                    }))).into_response();
                }
            }
        }
    };

    let model_id = {
        let engine = state.engine.lock().await;
        match engine.get_model_id(handle) {
            Some(id) => id,
            None => {
                return (StatusCode::NOT_FOUND, Json(serde_json::json!({
                    "error": "Model not found"
                }))).into_response();
            }
        }
    };

    let prompt = if req.model.to_lowercase().contains("llama") {
        hyperllama_core::Llama3Template.apply(&req.messages)
    } else {
        hyperllama_core::SimpleChatTemplate.apply(&req.messages)
    };

    let mut gen_req = GenerateRequest::new(
        handle.0,
        model_id,
        prompt,
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
            Ok(mut stream) => {
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
        let start = Instant::now();
        let engine = state.engine.lock().await;
        match engine.generate(gen_req).await {
            Ok(resp) => {
                let duration = start.elapsed();
                let msg = ChatMessage::assistant(resp.text);
                Json(ChatApiResponse {
                    model: req.model,
                    message: msg,
                    done: true,
                    total_duration: Some(duration.as_nanos() as u64),
                    eval_count: None,
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

#[derive(Debug, Deserialize)]
struct MaxServiceModel {
    model_id: String,
    model_path: String,
    loaded: bool,
    size_vram: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct MaxServiceGpuStats {
    total_vram: u64,
    used_vram: u64,
    free_vram: u64,
}

#[derive(Debug, Deserialize)]
struct MaxServiceModelsResponse {
    models: Vec<MaxServiceModel>,
    gpu_stats: Option<MaxServiceGpuStats>,
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

pub async fn ps(State(state): State<ServerState>) -> Response {
    info!("Process status request");

    let client = reqwest::Client::new();
    let max_service_url = "http://127.0.0.1:8100";

    match client.get(format!("{}/models", max_service_url)).send().await {
        Ok(response) => {
            if !response.status().is_success() {
                error!("MAX service returned error: {}", response.status());
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                    "error": "Failed to get model information from MAX service"
                }))).into_response();
            }

            match response.json::<MaxServiceModelsResponse>().await {
                Ok(max_models) => {
                    let mut processes = Vec::new();

                    for model in max_models.models {
                        let model_name = model.model_path.clone();

                        let size = match std::fs::metadata(&model_name) {
                            Ok(metadata) => metadata.len(),
                            Err(_) => 0,
                        };

                        let (family, parameter_size, format) = if model_name.contains("Llama") || model_name.contains("llama") {
                            let size = if model_name.contains("70B") {
                                "70B"
                            } else if model_name.contains("8B") {
                                "8B"
                            } else if model_name.contains("3B") {
                                "3B"
                            } else {
                                "unknown"
                            };
                            ("llama", size, "gguf")
                        } else {
                            ("unknown", "unknown", "gguf")
                        };

                        let mut hasher = DefaultHasher::new();
                        model_name.hash(&mut hasher);
                        let digest = format!("sha256:{:x}", hasher.finish());

                        processes.push(ProcessInfo {
                            name: model_name.clone(),
                            model: model_name,
                            size,
                            digest: Some(digest),
                            details: ModelDetails {
                                parent_model: "".to_string(),
                                format: format.to_string(),
                                family: family.to_string(),
                                parameter_size: parameter_size.to_string(),
                                quantization_level: "Q4_K_M".to_string(),
                            },
                            expires_at: None,
                            size_vram: model.size_vram,
                        });
                    }

                    Json(PsResponse { models: processes }).into_response()
                }
                Err(e) => {
                    error!("Failed to parse MAX service response: {}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                        "error": format!("Failed to parse model information: {}", e)
                    }))).into_response()
                }
            }
        }
        Err(e) => {
            error!("Failed to connect to MAX service: {}", e);
            (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({
                "error": "MAX service unavailable"
            }))).into_response()
        }
    }
}
