use crate::types::RequestId;
use serde::{Deserialize, Serialize};

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
pub struct GenerateOptions {
    pub stream: bool,
    pub sampling: SamplingParams,
    pub return_logprobs: bool,
    pub echo_prompt: bool,
}

impl Default for GenerateOptions {
    fn default() -> Self {
        Self {
            stream: false,
            sampling: SamplingParams::default(),
            return_logprobs: false,
            echo_prompt: false,
        }
    }
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
