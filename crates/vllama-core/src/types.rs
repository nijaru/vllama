use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RequestId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TokenId(pub u32);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Token {
    pub id: TokenId,
    pub text: String,
    pub logprob: Option<f32>,
}

impl Token {
    pub fn new(id: u32, text: String) -> Self {
        Self {
            id: TokenId(id),
            text,
            logprob: None,
        }
    }

    pub fn with_logprob(mut self, logprob: f32) -> Self {
        self.logprob = Some(logprob);
        self
    }
}
