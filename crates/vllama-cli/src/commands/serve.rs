use anyhow::{Context, Result};
use std::process::{Child, Command, Stdio};
use std::time::Duration;
use tokio::signal;
use tokio::time::sleep;
use tracing::{error, info, warn};
use vllama_server::Server;
use crate::output::{self, OutputMode};
use serde_json::json;

#[cfg(unix)]
use std::os::unix::process::CommandExt;

pub async fn run(
    host: String,
    port: u16,
    model: Option<String>,
    vllm_port: u16,
    no_vllm: bool,
    max_num_seqs: usize,
    gpu_memory_utilization: f32,
    output_mode: OutputMode,
) -> Result<()> {
    let mut vllm_process: Option<Child> = None;

    // Show header in normal mode
    if output_mode == OutputMode::Normal {
        println!("vllama v{}\n", env!("CARGO_PKG_VERSION"));
    }

    if !no_vllm {
        if let Some(model_name) = &model {
            info!("Starting vLLM OpenAI server on port {}", vllm_port);

            match output_mode {
                OutputMode::Normal => {
                    println!("{}", output::section("Loading model"));
                    output::kv("Model", model_name);
                    output::kv("Port", &vllm_port.to_string());
                    output::kv("Max sequences", &max_num_seqs.to_string());
                    output::kv("Batched tokens", "16,384");
                    output::kv("GPU memory", &format!("{:.0}%", gpu_memory_utilization * 100.0));
                    output::kv("Optimizations", "chunked-prefill, prefix-caching");
                    output::kv("Logs", "vllm.log");
                    println!();
                }
                OutputMode::Json => {
                    output::json(&json!({
                        "event": "vllm_starting",
                        "model": model_name,
                        "port": vllm_port,
                        "max_sequences": max_num_seqs,
                        "gpu_memory_utilization": gpu_memory_utilization
                    }));
                }
                OutputMode::Quiet => {}
            }

            vllm_process = Some(start_vllm_server(
                model_name,
                vllm_port,
                max_num_seqs,
                gpu_memory_utilization,
            )?);

            // Wait for vLLM with spinner
            let spinner = if output_mode == OutputMode::Normal {
                Some(output::spinner("Starting vLLM engine..."))
            } else {
                None
            };

            if !wait_for_vllm_ready(vllm_port).await {
                if let Some(sp) = spinner {
                    sp.finish_and_clear();
                }
                error!("vLLM server failed to start");
                if let Some(mut child) = vllm_process {
                    // Kill entire process tree to avoid orphaned subprocesses
                    let _ = kill_process_tree(&mut child);
                }

                if output_mode == OutputMode::Json {
                    output::json(&json!({"event": "error", "message": "vLLM server failed to start"}));
                }

                anyhow::bail!("vLLM server failed to start within 120 seconds");
            }

            if let Some(sp) = spinner {
                sp.finish_with_message(output::success("vLLM engine ready"));
            }

            if output_mode == OutputMode::Json {
                output::json(&json!({"event": "vllm_ready"}));
            }

            if output_mode == OutputMode::Normal {
                println!();
            }
        } else {
            warn!("No model specified, skipping vLLM server startup");

            match output_mode {
                OutputMode::Normal => {
                    println!("{}", output::warning("No model specified"));
                    println!("{}", output::bullet("Use --model to auto-start vLLM server"));
                    println!("{}", output::bullet("Or use --no-vllm if vLLM is already running"));
                    println!();
                }
                OutputMode::Json => {
                    output::json(&json!({
                        "event": "warning",
                        "message": "No model specified, vLLM not started"
                    }));
                }
                OutputMode::Quiet => {}
            }
        }
    } else {
        info!("Skipping vLLM server startup (--no-vllm flag)");

        match output_mode {
            OutputMode::Normal => {
                println!("{}", output::info(&format!("Connecting to existing vLLM server on port {}", vllm_port)));
                println!();
            }
            OutputMode::Json => {
                output::json(&json!({
                    "event": "info",
                    "message": "Using existing vLLM server",
                    "port": vllm_port
                }));
            }
            OutputMode::Quiet => {}
        }
    }

    info!("Starting vLLama server on {}:{}", host, port);

    match output_mode {
        OutputMode::Normal => {
            println!("{}", output::section("Starting Ollama API"));
            println!("{}", output::success(&format!("Listening on http://{}:{}", host, port)));
            println!();
            println!("  Endpoints:");
            println!("{}", output::bullet("POST /api/generate"));
            println!("{}", output::bullet("POST /api/chat"));
            println!("{}", output::bullet("POST /v1/chat/completions"));
            println!("{}", output::bullet("GET  /api/ps"));
            println!();
            println!("Press Ctrl+C to stop");
            println!();
        }
        OutputMode::Quiet => {
            println!("{}", output::success(&format!("Listening on http://{}:{}", host, port)));
        }
        OutputMode::Json => {
            output::json(&json!({
                "event": "server_started",
                "host": host,
                "port": port,
                "endpoints": [
                    "/api/generate",
                    "/api/chat",
                    "/v1/chat/completions",
                    "/api/ps"
                ]
            }));
        }
    }

    let server = Server::new(host, port).map_err(|e| anyhow::anyhow!("{}", e))?;

    let server_future = server.run();
    let shutdown_signal = shutdown_signal();

    tokio::select! {
        result = server_future => {
            result.map_err(|e| anyhow::anyhow!("{}", e))?;
        }
        _ = shutdown_signal => {
            if output_mode == OutputMode::Normal {
                println!();
                println!("{}", output::info("Shutting down..."));
            }
            info!("Received shutdown signal");
        }
    }

    if let Some(mut child) = vllm_process {
        info!("Stopping vLLM server");

        if output_mode == OutputMode::Normal {
            let spinner = output::spinner("Stopping vLLM engine...");
            if let Err(e) = kill_process_tree(&mut child) {
                warn!("Failed to kill vLLM process tree: {}", e);
                spinner.finish_with_message(output::warning("vLLM process cleanup failed"));
            } else {
                let _ = child.wait();
                spinner.finish_with_message(output::success("vLLM engine stopped"));
            }
        } else {
            let _ = kill_process_tree(&mut child);
            let _ = child.wait();
        }

        if output_mode == OutputMode::Json {
            output::json(&json!({"event": "vllm_stopped"}));
        }
    }

    match output_mode {
        OutputMode::Normal => println!("{}", output::success("Shutdown complete")),
        OutputMode::Json => output::json(&json!({"event": "shutdown_complete"})),
        OutputMode::Quiet => {}
    }

    Ok(())
}

/// Kill a child process and all its descendants
///
/// This is necessary because `uv run` spawns Python as a subprocess,
/// and calling `child.kill()` only kills the parent `uv` process,
/// leaving the Python vLLM server orphaned.
#[cfg(unix)]
fn kill_process_tree(child: &mut Child) -> std::io::Result<()> {
    let pid = child.id();

    // Try graceful termination first with SIGTERM to process group
    // The negative PID signals the entire process group
    unsafe {
        libc::kill(-(pid as i32), libc::SIGTERM);
    }

    // Give it 2 seconds to clean up
    std::thread::sleep(Duration::from_secs(2));

    // Force kill with SIGKILL if still running
    unsafe {
        libc::kill(-(pid as i32), libc::SIGKILL);
    }

    Ok(())
}

#[cfg(not(unix))]
fn kill_process_tree(child: &mut Child) -> std::io::Result<()> {
    // On Windows, just kill the process
    child.kill()
}

fn start_vllm_server(
    model: &str,
    port: u16,
    max_num_seqs: usize,
    gpu_memory_utilization: f32,
) -> Result<Child> {
    // Redirect vLLM output to log file for clean CLI UX
    use std::fs::OpenOptions;

    let log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("vllm.log")
        .context("Failed to create vllm.log file")?;

    #[cfg(unix)]
    let child = {
        Command::new("uv")
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
                // Performance optimizations
                "--enable-chunked-prefill", // Better concurrent request handling
                "--enable-prefix-caching",  // Reuse KV cache for repeated prompts
                // Memory
                "--gpu-memory-utilization",
                &gpu_memory_utilization.to_string(),
            ])
            .stdout(Stdio::from(log_file.try_clone()?))
            .stderr(Stdio::from(log_file))
            // Create new process group so we can kill the entire tree
            .process_group(0)
            .spawn()
            .context("Failed to start vLLM server. Is uv installed? (curl -LsSf https://astral.sh/uv/install.sh | sh)")?
    };

    #[cfg(not(unix))]
    let child = {
        Command::new("uv")
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
                "16384",
                // Performance optimizations
                "--enable-chunked-prefill",
                "--enable-prefix-caching",
                // Memory
                "--gpu-memory-utilization",
                &gpu_memory_utilization.to_string(),
            ])
            .stdout(Stdio::from(log_file.try_clone()?))
            .stderr(Stdio::from(log_file))
            .spawn()
            .context("Failed to start vLLM server. Is uv installed? (curl -LsSf https://astral.sh/uv/install.sh | sh)")?
    };

    Ok(child)
}

async fn wait_for_vllm_ready(port: u16) -> bool {
    let client = reqwest::Client::new();
    let url = format!("http://127.0.0.1:{}/health", port);

    // Wait up to 120 seconds for vLLM to start
    // First startup takes ~67s due to CUDA graph compilation
    // (captures 67 mixed prefill-decode graphs + 35 decode graphs)
    for _ in 0..120 {
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
