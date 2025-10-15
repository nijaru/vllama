use crate::types::RequestId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ChatRole {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<String>>,
}

impl ChatMessage {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: ChatRole::System,
            content: content.into(),
            images: None,
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: ChatRole::User,
            content: content.into(),
            images: None,
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: ChatRole::Assistant,
            content: content.into(),
            images: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingParams {
    pub temperature: f32,
    pub top_p: f32,
    pub top_k: Option<u32>,
    pub repetition_penalty: f32,
    pub frequency_penalty: f32,
    pub presence_penalty: f32,
    pub max_tokens: Option<usize>,
    pub stop_sequences: Vec<String>,
}

impl Default for SamplingParams {
    fn default() -> Self {
        Self {
            temperature: 0.7,
            top_p: 0.9,
            top_k: None,
            repetition_penalty: 1.0,
            frequency_penalty: 0.0,
            presence_penalty: 0.0,
            max_tokens: None,
            stop_sequences: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct GenerateOptions {
    pub stream: bool,
    pub sampling: SamplingParams,
    pub return_logprobs: bool,
    pub echo_prompt: bool,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateRequest {
    pub id: RequestId,
    pub model: String,
    pub prompt: String,
    pub options: GenerateOptions,
}

impl GenerateRequest {
    pub fn new(id: u64, model: String, prompt: String) -> Self {
        Self {
            id: RequestId(id),
            model,
            prompt,
            options: GenerateOptions::default(),
        }
    }

    pub fn with_options(mut self, options: GenerateOptions) -> Self {
        self.options = options;
        self
    }

    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.options.sampling.temperature = temperature;
        self
    }

    pub fn with_max_tokens(mut self, max_tokens: usize) -> Self {
        self.options.sampling.max_tokens = Some(max_tokens);
        self
    }

    pub fn with_stream(mut self, stream: bool) -> Self {
        self.options.stream = stream;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub id: RequestId,
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub options: GenerateOptions,
}

impl ChatRequest {
    pub fn new(id: u64, model: String, messages: Vec<ChatMessage>) -> Self {
        Self {
            id: RequestId(id),
            model,
            messages,
            options: GenerateOptions::default(),
        }
    }

    pub fn with_options(mut self, options: GenerateOptions) -> Self {
        self.options = options;
        self
    }

    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.options.sampling.temperature = temperature;
        self
    }

    pub fn with_max_tokens(mut self, max_tokens: usize) -> Self {
        self.options.sampling.max_tokens = Some(max_tokens);
        self
    }

    pub fn with_stream(mut self, stream: bool) -> Self {
        self.options.stream = stream;
        self
    }

    pub fn to_prompt(&self) -> String {
        self.messages
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
}
