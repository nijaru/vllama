use anyhow::Result;
use tracing::info;

pub async fn run(host: String, port: u16) -> Result<()> {
    info!("Starting HyperLlama server on {}:{}", host, port);
    println!("🚀 HyperLlama server starting on {}:{}", host, port);
    println!("📊 Press Ctrl+C to stop");

    Ok(())
}
