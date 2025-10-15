use crate::types::{RequestId, Token};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    pub token: Token,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationStats {
    pub prompt_tokens: usize,
    pub generated_tokens: usize,
    pub total_tokens: usize,
    pub prompt_time_ms: u64,
    pub generation_time_ms: u64,
    pub tokens_per_second: f64,
}

impl GenerationStats {
    pub fn new(prompt_tokens: usize, generated_tokens: usize) -> Self {
        Self {
            prompt_tokens,
            generated_tokens,
            total_tokens: prompt_tokens + generated_tokens,
            prompt_time_ms: 0,
            generation_time_ms: 0,
            tokens_per_second: 0.0,
        }
    }

    pub fn with_timings(mut self, prompt_time: Duration, generation_time: Duration) -> Self {
        self.prompt_time_ms = prompt_time.as_millis() as u64;
        self.generation_time_ms = generation_time.as_millis() as u64;

        if self.generation_time_ms > 0 {
            self.tokens_per_second =
                self.generated_tokens as f64 / generation_time.as_secs_f64();
        }

        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateResponse {
    pub id: RequestId,
    pub model: String,
    pub text: String,
    pub tokens: Vec<TokenInfo>,
    pub stats: GenerationStats,
    pub finished: bool,
    pub finish_reason: Option<String>,
}

impl GenerateResponse {
    pub fn new(id: RequestId, model: String) -> Self {
        Self {
            id,
            model,
            text: String::new(),
            tokens: Vec::new(),
            stats: GenerationStats::new(0, 0),
            finished: false,
            finish_reason: None,
        }
    }

    pub fn with_text(mut self, text: String) -> Self {
        self.text = text;
        self
    }

    pub fn with_stats(mut self, stats: GenerationStats) -> Self {
        self.stats = stats;
        self
    }

    pub fn finish(mut self, reason: String) -> Self {
        self.finished = true;
        self.finish_reason = Some(reason);
        self
    }
}
