/// OpenAI API client for vLLM server
///
/// This module provides an OpenAI-compatible API client for communicating
/// with vLLM's OpenAI-compatible server.
use serde::{Deserialize, Serialize};
use crate::{Error, Result};

/// OpenAI API client
pub struct OpenAIClient {
    client: reqwest::Client,
    base_url: String,
}

impl OpenAIClient {
    /// Create a new OpenAI API client
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.into(),
        }
    }

    /// Create completion
    pub async fn create_completion(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        let url = format!("{}/v1/completions", self.base_url);

        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| Error::ModelLoadFailed(format!("OpenAI API request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::ModelLoadFailed(format!(
                "OpenAI API error ({}): {}",
                status, text
            )));
        }

        response
            .json()
            .await
            .map_err(|e| Error::ModelLoadFailed(format!("Failed to parse response: {}", e)))
    }

    /// Create chat completion
    pub async fn create_chat_completion(&self, request: ChatCompletionRequest) -> Result<ChatCompletionResponse> {
        let url = format!("{}/v1/chat/completions", self.base_url);

        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| Error::ModelLoadFailed(format!("OpenAI API request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::ModelLoadFailed(format!(
                "OpenAI API error ({}): {}",
                status, text
            )));
        }

        response
            .json()
            .await
            .map_err(|e| Error::ModelLoadFailed(format!("Failed to parse response: {}", e)))
    }

    /// Create streaming completion
    pub async fn create_completion_stream(
        &self,
        request: CompletionRequest,
    ) -> Result<impl futures::Stream<Item = Result<CompletionChunk>>> {
        let url = format!("{}/v1/completions", self.base_url);

        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| Error::ModelLoadFailed(format!("OpenAI API request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::ModelLoadFailed(format!(
                "OpenAI API error ({}): {}",
                status, text
            )));
        }

        Ok(Self::parse_sse_stream(response))
    }

    /// Parse SSE stream into completion chunks
    fn parse_sse_stream(
        response: reqwest::Response,
    ) -> impl futures::Stream<Item = Result<CompletionChunk>> {
        use futures::stream::StreamExt;

        response
            .bytes_stream()
            .map(|result| {
                result.map_err(|e| Error::ModelLoadFailed(format!("Stream error: {}", e)))
            })
            .filter_map(|result| async move {
                match result {
                    Ok(bytes) => {
                        let text = String::from_utf8_lossy(&bytes);
                        for line in text.lines() {
                            if let Some(data) = line.strip_prefix("data: ") {
                                if data == "[DONE]" {
                                    return None;
                                }
                                match serde_json::from_str::<CompletionChunk>(data) {
                                    Ok(chunk) => return Some(Ok(chunk)),
                                    Err(e) => {
                                        return Some(Err(Error::ModelLoadFailed(format!(
                                            "Failed to parse chunk: {}",
                                            e
                                        ))))
                                    }
                                }
                            }
                        }
                        None
                    }
                    Err(e) => Some(Err(e)),
                }
            })
    }

    /// Health check
    pub async fn health(&self) -> Result<bool> {
        let url = format!("{}/health", self.base_url);

        match self.client.get(&url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}

// ============================================================================
// OpenAI API Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    pub model: String,
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<CompletionChoice>,
    pub usage: Usage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionChoice {
    pub text: String,
    pub index: usize,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionChunk {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<CompletionChoiceChunk>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionChoiceChunk {
    pub text: String,
    pub index: usize,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<ChatCompletionChoice>,
    pub usage: Usage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionChoice {
    pub index: usize,
    pub message: ChatMessage,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub total_tokens: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = OpenAIClient::new("http://localhost:8100");
        assert_eq!(client.base_url, "http://localhost:8100");
    }

    #[test]
    fn test_completion_request_serialization() {
        let request = CompletionRequest {
            model: "test-model".to_string(),
            prompt: "Hello".to_string(),
            max_tokens: Some(50),
            temperature: Some(0.7),
            top_p: Some(0.9),
            stream: Some(false),
            stop: None,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("test-model"));
        assert!(json.contains("Hello"));
    }
}
