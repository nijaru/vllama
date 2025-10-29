//! Clean CLI output utilities
//!
//! Modern CLI UX following best practices from cargo, rustup, docker, etc.
//! - No emojis, clean symbols only
//! - Progress indicators for operations
//! - Structured, scannable output
//! - JSON mode for scripting

use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use serde::Serialize;
use std::time::Duration;

/// Output mode for CLI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputMode {
    /// Normal human-readable output
    Normal,
    /// Minimal output
    Quiet,
    /// JSON output for scripting
    Json,
}

/// Clean symbols for modern CLI output
pub struct Symbols;

impl Symbols {
    pub const ARROW: &'static str = "→";
    pub const BULLET: &'static str = "•";
    pub const SUCCESS: &'static str = "✓";
    pub const ERROR: &'static str = "✗";
    pub const WARNING: &'static str = "⚠";
    pub const INFO: &'static str = "ℹ";
}

/// Format a section header
pub fn section(title: &str) -> String {
    format!("{} {}", style(Symbols::ARROW).cyan().bold(), style(title).bold())
}

/// Format a success message
pub fn success(msg: &str) -> String {
    format!("{} {}", style(Symbols::SUCCESS).green().bold(), msg)
}

/// Format an error message
pub fn error(msg: &str) -> String {
    format!("{} {}", style(Symbols::ERROR).red().bold(), msg)
}

/// Format a warning message
pub fn warning(msg: &str) -> String {
    format!("{} {}", style(Symbols::WARNING).yellow().bold(), msg)
}

/// Format an info message
pub fn info(msg: &str) -> String {
    format!("{} {}", style(Symbols::INFO).blue().bold(), msg)
}

/// Format a bullet point
pub fn bullet(msg: &str) -> String {
    format!("  {} {}", style(Symbols::BULLET).dim(), msg)
}

/// Create a spinner for indeterminate operations
pub fn spinner(msg: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    pb.set_message(msg.to_string());
    pb.enable_steady_tick(Duration::from_millis(80));
    pb
}

/// Create a progress bar for operations with known total
pub fn progress_bar(total: u64, msg: &str) -> ProgressBar {
    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{msg}\n  [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
            .unwrap()
            .progress_chars("━━╺"),
    );
    pb.set_message(msg.to_string());
    pb
}

/// Output JSON for scripting
pub fn json<T: Serialize>(data: &T) {
    if let Ok(json) = serde_json::to_string(data) {
        println!("{}", json);
    }
}

/// Print key-value pair
pub fn kv(key: &str, value: &str) {
    println!("  {} {}: {}", style(Symbols::BULLET).dim(), style(key).dim(), value);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbols_no_emojis() {
        // Ensure we're not using emoji characters
        assert_ne!(Symbols::SUCCESS, "✅");
        assert_ne!(Symbols::ERROR, "❌");
        assert_ne!(Symbols::ARROW, "🚀");
    }
}
