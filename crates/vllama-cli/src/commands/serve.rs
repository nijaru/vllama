use anyhow::{Context, Result};
use std::process::{Child, Command, Stdio};
use std::time::Duration;
use tokio::signal;
use tokio::time::sleep;
use tracing::{error, info, warn};
use vllama_server::Server;

pub async fn run(
    host: String,
    port: u16,
    model: Option<String>,
    vllm_port: u16,
    no_vllm: bool,
    max_num_seqs: usize,
    gpu_memory_utilization: f32,
) -> Result<()> {
    let mut vllm_process: Option<Child> = None;

    if !no_vllm {
        if let Some(model_name) = &model {
            info!("Starting vLLM OpenAI server on port {}", vllm_port);
            println!("🚀 Starting vLLM OpenAI server...");
            println!("   Model: {}", model_name);
            println!("   Port: {}", vllm_port);
            println!("   Max sequences: {}", max_num_seqs);
            println!("   Batched tokens: 16384 (optimized)");
            println!("   GPU memory: {:.0}%", gpu_memory_utilization * 100.0);
            println!("   Optimizations: chunked-prefill ✓ prefix-caching ✓");
            println!();

            vllm_process = Some(start_vllm_server(
                model_name,
                vllm_port,
                max_num_seqs,
                gpu_memory_utilization,
            )?);

            println!("⏳ Waiting for vLLM server to be ready...");
            if !wait_for_vllm_ready(vllm_port).await {
                error!("vLLM server failed to start");
                if let Some(mut child) = vllm_process {
                    let _ = child.kill();
                }
                anyhow::bail!("vLLM server failed to start within 60 seconds");
            }
            println!("✓ vLLM server ready!");
            println!();
        } else {
            warn!("No model specified, skipping vLLM server startup");
            println!("⚠️  No model specified. Use --model to auto-start vLLM server.");
            println!("   Or use --no-vllm if vLLM is already running.");
            println!();
        }
    } else {
        info!("Skipping vLLM server startup (--no-vllm flag)");
        println!("ℹ️  Connecting to existing vLLM server on port {}", vllm_port);
        println!();
    }

    info!("Starting vLLama server on {}:{}", host, port);
    println!("🚀 vLLama server starting on {}:{}", host, port);
    println!();
    println!("API Endpoints:");
    println!("  POST {}:{}/api/generate - Generate text", host, port);
    println!("  POST {}:{}/api/chat - Chat completions", host, port);
    println!("  POST {}:{}/v1/chat/completions - OpenAI chat", host, port);
    println!("  GET  {}:{}/api/tags - List models", host, port);
    println!("  GET  {}:{}/health - Health check", host, port);
    println!();
    println!("Press Ctrl+C to stop");
    println!();

    let server = Server::new(host, port).map_err(|e| anyhow::anyhow!("{}", e))?;

    let server_future = server.run();
    let shutdown_signal = shutdown_signal();

    tokio::select! {
        result = server_future => {
            result.map_err(|e| anyhow::anyhow!("{}", e))?;
        }
        _ = shutdown_signal => {
            println!("\n🛑 Shutting down...");
            info!("Received shutdown signal");
        }
    }

    if let Some(mut child) = vllm_process {
        info!("Stopping vLLM server");
        println!("🛑 Stopping vLLM server...");
        if let Err(e) = child.kill() {
            warn!("Failed to kill vLLM process: {}", e);
        } else {
            let _ = child.wait();
            println!("✓ vLLM server stopped");
        }
    }

    println!("✓ Shutdown complete");
    Ok(())
}

fn start_vllm_server(
    model: &str,
    port: u16,
    max_num_seqs: usize,
    gpu_memory_utilization: f32,
) -> Result<Child> {
    let child = Command::new("uv")
        .args([
            "run",
            "--directory",
            "python",
            "python",
            "-m",
            "vllm.entrypoints.openai.api_server",
            "--model",
            model,
            "--port",
            &port.to_string(),
            // Concurrency & Batching
            "--max-num-seqs",
            &max_num_seqs.to_string(),
            "--max-num-batched-tokens",
            "16384", // 32x increase from default (512) for better throughput
            // Context length
            "--max-model-len",
            "4096", // Typical context length for most workloads
            // Performance optimizations
            "--enable-chunked-prefill", // Better concurrent request handling
            "--enable-prefix-caching",  // Reuse KV cache for repeated prompts
            // Memory
            "--gpu-memory-utilization",
            &gpu_memory_utilization.to_string(),
        ])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .context("Failed to start vLLM server. Is uv installed? (curl -LsSf https://astral.sh/uv/install.sh | sh)")?;

    Ok(child)
}

async fn wait_for_vllm_ready(port: u16) -> bool {
    let client = reqwest::Client::new();
    let url = format!("http://127.0.0.1:{}/health", port);

    for _ in 0..60 {
        sleep(Duration::from_secs(1)).await;

        match client.get(&url).send().await {
            Ok(response) if response.status().is_success() => {
                return true;
            }
            _ => continue,
        }
    }

    false
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
