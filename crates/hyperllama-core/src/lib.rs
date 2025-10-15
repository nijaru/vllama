pub mod types;
pub mod request;
pub mod response;
pub mod model;
pub mod hardware;
pub mod error;
pub mod templates;

pub use error::{Error, Result};
pub use hardware::{Hardware, HardwareType, GpuInfo};
pub use model::{ModelHandle, ModelInfo, ModelFormat};
pub use request::{ChatMessage, ChatRequest, ChatRole, GenerateRequest, GenerateOptions, SamplingParams};
pub use response::{GenerateResponse, TokenInfo, GenerationStats};
pub use templates::{ChatTemplate, Llama3Template, SimpleChatTemplate, get_template_for_model};
pub use types::{RequestId, Token, TokenId};
