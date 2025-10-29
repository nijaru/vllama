pub mod types;
pub mod request;
pub mod response;
pub mod model;
pub mod hardware;
pub mod error;
pub mod downloader;
pub mod openai;

pub use downloader::{CachedModel, DownloadProgress, ModelDownloader};
pub use error::{Error, Result};
pub use hardware::{Hardware, HardwareType, GpuInfo};
pub use model::{ModelHandle, ModelInfo, ModelFormat};
pub use openai::{OpenAIClient, CompletionRequest, CompletionResponse, ChatCompletionRequest, ChatCompletionResponse};
pub use request::{ChatMessage, ChatRequest, ChatRole, GenerateRequest, GenerateOptions, SamplingParams};
pub use response::{GenerateResponse, TokenInfo, GenerationStats};
pub use types::{RequestId, Token, TokenId};
