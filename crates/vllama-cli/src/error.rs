//! User-friendly error handling for vllama CLI
//!
//! Provides clear error messages with helpful suggestions instead of raw stack traces.

use crate::output;
use std::fmt;
use std::process::ExitCode;

/// Exit codes following Unix conventions
pub const EXIT_SUCCESS: u8 = 0;
pub const EXIT_ERROR: u8 = 1;
pub const EXIT_INVALID_INPUT: u8 = 2;

/// User-facing error with helpful context
pub struct UserError {
    /// What went wrong (user-friendly)
    pub message: String,
    /// Why it happened (optional context)
    pub context: Option<String>,
    /// What to try next (helpful suggestions)
    pub suggestions: Vec<String>,
    /// Exit code
    pub exit_code: u8,
}

impl UserError {
    /// Create a new user error
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            context: None,
            suggestions: Vec::new(),
            exit_code: EXIT_ERROR,
        }
    }

    /// Add context about why the error occurred
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }

    /// Add a suggestion for how to fix the error
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestions.push(suggestion.into());
        self
    }

    /// Set custom exit code
    pub fn with_exit_code(mut self, code: u8) -> Self {
        self.exit_code = code;
        self
    }

    /// Display the error to the user and exit
    pub fn exit(self) -> ! {
        eprintln!("{}", self);
        std::process::exit(self.exit_code as i32);
    }

    /// Get exit code
    pub fn exit_code(&self) -> ExitCode {
        ExitCode::from(self.exit_code)
    }
}

impl fmt::Display for UserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Error header
        writeln!(f, "{}", output::error(&self.message))?;

        // Optional context
        if let Some(ref context) = self.context {
            writeln!(f)?;
            writeln!(f, "  {}", context)?;
        }

        // Suggestions
        if !self.suggestions.is_empty() {
            writeln!(f)?;
            writeln!(f, "  Suggestions:")?;
            for suggestion in &self.suggestions {
                writeln!(f, "  {} {}", output::Symbols::BULLET, suggestion)?;
            }
        }

        Ok(())
    }
}

impl fmt::Debug for UserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl std::error::Error for UserError {}

/// Convert anyhow errors into user-friendly errors
pub fn handle_error(err: anyhow::Error) -> UserError {
    let err_str = err.to_string();

    // Model not found
    if err_str.contains("404") || err_str.contains("not found") {
        return UserError::new("Model not found")
            .with_context("The specified model could not be loaded from HuggingFace.")
            .with_suggestion("Check the model name spelling")
            .with_suggestion("Search for models at https://huggingface.co/models")
            .with_suggestion("For gated models (Llama), see docs/MODELS.md for authentication setup");
    }

    // Port already in use
    if err_str.contains("Address already in use") {
        return UserError::new("Port already in use")
            .with_context("Another process is already using the specified port.")
            .with_suggestion("Stop existing vllama/vLLM instances: pkill -9 vllm")
            .with_suggestion("Use a different port: --port 11435")
            .with_suggestion("Check what's using the port: lsof -i :11434");
    }

    // vLLM startup failed
    if err_str.contains("vLLM server failed to start") {
        return UserError::new("vLLM engine failed to start")
            .with_context("The vLLM inference engine could not initialize.")
            .with_suggestion("Check vllm.log for detailed error messages")
            .with_suggestion("Ensure CUDA is installed: nvidia-smi")
            .with_suggestion("Try with smaller model or lower GPU utilization: --gpu-memory-utilization 0.7")
            .with_suggestion("For 7B models, use --gpu-memory-utilization 0.9");
    }

    // OOM / insufficient memory
    if err_str.contains("out of memory") || err_str.contains("No available memory") {
        return UserError::new("Insufficient GPU memory")
            .with_context("The model is too large for available GPU memory.")
            .with_suggestion("Use a smaller model (e.g., Qwen/Qwen2.5-1.5B-Instruct)")
            .with_suggestion("Increase GPU utilization: --gpu-memory-utilization 0.9")
            .with_suggestion("Check GPU memory: nvidia-smi")
            .with_suggestion("See docs/MODELS.md for memory requirements");
    }

    // uv not installed
    if err_str.contains("uv") && (err_str.contains("not found") || err_str.contains("No such file")) {
        return UserError::new("uv package manager not found")
            .with_context("vllama requires uv to manage Python dependencies.")
            .with_suggestion("Install uv: curl -LsSf https://astral.sh/uv/install.sh | sh")
            .with_suggestion("After installing, restart your shell");
    }

    // CUDA not available
    if err_str.contains("CUDA") && err_str.contains("not available") {
        return UserError::new("CUDA not available")
            .with_context("vLLM requires NVIDIA GPU with CUDA support.")
            .with_suggestion("Check GPU is detected: nvidia-smi")
            .with_suggestion("Install CUDA drivers: See docs/FEDORA_SETUP.md")
            .with_suggestion("For CPU-only inference, use Ollama instead (faster on CPU)");
    }

    // Gated model / authentication required
    if err_str.contains("401") || err_str.contains("gated") || err_str.contains("authentication") {
        return UserError::new("Model requires authentication")
            .with_context("This model is gated and requires HuggingFace authentication.")
            .with_suggestion("Create HuggingFace token at https://huggingface.co/settings/tokens")
            .with_suggestion("Accept model license on HuggingFace")
            .with_suggestion("Set token: export HF_TOKEN=hf_...")
            .with_suggestion("See docs/MODELS.md for detailed setup");
    }

    // Generic fallback
    UserError::new("An error occurred")
        .with_context(&err_str)
        .with_suggestion("Check vllm.log for detailed error information")
        .with_suggestion("Report issues at https://github.com/nijaru/vllama/issues")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_error_display() {
        let err = UserError::new("Something went wrong")
            .with_context("Because of reasons")
            .with_suggestion("Try this first")
            .with_suggestion("Or try this");

        let output = format!("{}", err);
        assert!(output.contains("Something went wrong"));
        assert!(output.contains("Because of reasons"));
        assert!(output.contains("Try this first"));
    }

    #[test]
    fn test_exit_codes() {
        assert_eq!(EXIT_SUCCESS, 0);
        assert_eq!(EXIT_ERROR, 1);
        assert_eq!(EXIT_INVALID_INPUT, 2);
    }
}
