use anyhow::Result;
use hyperllama_server::Server;
use tracing::info;

pub async fn run(host: String, port: u16) -> Result<()> {
    info!("Starting HyperLlama server on {}:{}", host, port);
    println!("HyperLlama server starting on {}:{}", host, port);
    println!("Press Ctrl+C to stop");
    println!();
    println!("API Endpoints:");
    println!("  POST {}:{}/api/generate - Generate text", host, port);
    println!("  GET  {}:{}/api/tags - List models", host, port);
    println!("  GET  {}:{}/health - Health check", host, port);
    println!();

    let server = Server::new(host, port).map_err(|e| anyhow::anyhow!("{}", e))?;
    server.run().await.map_err(|e| anyhow::anyhow!("{}", e))?;

    Ok(())
}
