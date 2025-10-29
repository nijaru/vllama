use anyhow::Result;
use serde::Serialize;
use vllama_core::{GenerateRequest, Hardware};
use vllama_engine::{InferenceEngine, VllmOpenAIEngine};
use std::time::Instant;
use tokio::task::JoinSet;
use tracing::warn;

use crate::output::{self, OutputMode};

#[derive(Debug, Serialize)]
pub struct BenchmarkResult {
    model: String,
    concurrency: usize,
    iterations: usize,
    vllama: Option<EngineStats>,
    ollama: Option<EngineStats>,
    hardware: HardwareInfo,
}

#[derive(Debug, Serialize)]
pub struct EngineStats {
    median_latency_ms: f64,
    avg_latency_ms: f64,
    p99_latency_ms: f64,
    tokens_per_sec: f64,
    total_time_secs: f64,
    requests_per_sec: f64,
}

#[derive(Debug, Serialize)]
pub struct HardwareInfo {
    hw_type: String,
    cpu_cores: usize,
    ram_mb: u64,
}

pub async fn execute(
    model: String,
    prompt: String,
    iterations: usize,
    concurrency: usize,
    output_mode: OutputMode,
) -> Result<()> {
    let hw = Hardware::detect();
    let hw_info = HardwareInfo {
        hw_type: format!("{:?}", hw.hw_type),
        cpu_cores: hw.cpu_cores,
        ram_mb: hw.ram_total_mb,
    };

    // Show header in normal mode
    if output_mode == OutputMode::Normal {
        println!("{}", output::section("vllama Benchmark"));
        println!();
        output::kv("Model", &model);
        output::kv("Concurrency", &concurrency.to_string());
        output::kv("Iterations", &iterations.to_string());
        output::kv("Hardware", &format!("{:?}", hw.hw_type));
        output::kv("CPU Cores", &hw.cpu_cores.to_string());
        output::kv("RAM", &format!("{} MB", hw.ram_total_mb));
        println!();
    }

    // Test vLLM
    if output_mode == OutputMode::Normal {
        println!("{}", output::info("Testing vllama..."));
    }

    let vllama_result = if concurrency == 1 {
        test_vllm_sequential(&model, &prompt, iterations).await
    } else {
        test_vllm_concurrent(&model, &prompt, iterations, concurrency).await
    };

    let vllama_stats = match vllama_result {
        Ok(stats) => {
            if output_mode == OutputMode::Normal {
                println!("{}", output::success("vllama results:"));
                output::kv("Median latency", &format!("{:.2} ms", stats.median_latency_ms));
                output::kv("P99 latency", &format!("{:.2} ms", stats.p99_latency_ms));
                output::kv("Throughput", &format!("{:.2} req/s", stats.requests_per_sec));
                output::kv("Tokens/sec", &format!("{:.2}", stats.tokens_per_sec));
                println!();
            }
            Some(stats)
        }
        Err(e) => {
            warn!("vllama test failed: {}", e);
            if output_mode == OutputMode::Normal {
                println!("{}", output::error(&format!("vllama test failed: {}", e)));
                println!();
            }
            None
        }
    };

    // Test Ollama
    if output_mode == OutputMode::Normal {
        println!("{}", output::info("Testing Ollama (port 11435)..."));
    }

    let ollama_result = if concurrency == 1 {
        test_ollama_sequential(&model, &prompt, iterations).await
    } else {
        test_ollama_concurrent(&model, &prompt, iterations, concurrency).await
    };

    let ollama_stats = match ollama_result {
        Ok(stats) => {
            if output_mode == OutputMode::Normal {
                println!("{}", output::success("Ollama results:"));
                output::kv("Median latency", &format!("{:.2} ms", stats.median_latency_ms));
                output::kv("P99 latency", &format!("{:.2} ms", stats.p99_latency_ms));
                output::kv("Throughput", &format!("{:.2} req/s", stats.requests_per_sec));
                output::kv("Tokens/sec", &format!("{:.2}", stats.tokens_per_sec));
                println!();
            }
            Some(stats)
        }
        Err(e) => {
            warn!("Ollama test failed: {}", e);
            if output_mode == OutputMode::Normal {
                println!("{}", output::warning("Ollama not available (start on port 11435)"));
                println!();
            }
            None
        }
    };

    // Show comparison
    if output_mode == OutputMode::Normal {
        if let (Some(vllama), Some(ollama)) = (&vllama_stats, &ollama_stats) {
            let speedup = ollama.total_time_secs / vllama.total_time_secs;
            println!("{}", output::section("Comparison"));
            println!();
            output::kv("Speedup", &format!("{:.2}x faster", speedup));
            let latency_improvement = (ollama.median_latency_ms - vllama.median_latency_ms) / ollama.median_latency_ms * 100.0;
            output::kv("Latency improvement", &format!("{:.1}% lower", latency_improvement));
        }
    }

    // Output structured results
    let result = BenchmarkResult {
        model,
        concurrency,
        iterations,
        vllama: vllama_stats,
        ollama: ollama_stats,
        hardware: hw_info,
    };

    if output_mode == OutputMode::Json {
        output::json(&result);
    }

    Ok(())
}

#[derive(Debug, Clone)]
struct BenchStats {
    median_latency_ms: f64,
    avg_latency_ms: f64,
    p99_latency_ms: f64,
    tokens_per_sec: f64,
    total_time_secs: f64,
    requests_per_sec: f64,
}

impl From<BenchStats> for EngineStats {
    fn from(stats: BenchStats) -> Self {
        Self {
            median_latency_ms: stats.median_latency_ms,
            avg_latency_ms: stats.avg_latency_ms,
            p99_latency_ms: stats.p99_latency_ms,
            tokens_per_sec: stats.tokens_per_sec,
            total_time_secs: stats.total_time_secs,
            requests_per_sec: stats.requests_per_sec,
        }
    }
}

fn calculate_percentile(sorted_values: &[f64], percentile: f64) -> f64 {
    let index = (percentile / 100.0 * (sorted_values.len() - 1) as f64).round() as usize;
    sorted_values[index]
}

async fn test_vllm_sequential(model: &str, prompt: &str, iterations: usize) -> Result<EngineStats> {
    let vllm_engine = VllmOpenAIEngine::new("http://127.0.0.1:8100");

    if !vllm_engine.health_check().await? {
        anyhow::bail!("vLLM OpenAI server not available (run: python -m vllm.entrypoints.openai.api_server --model {} --port 8100)", model);
    }

    let mut latencies = Vec::new();
    let mut total_tokens = 0usize;
    let start = Instant::now();

    for i in 0..iterations {
        let request = GenerateRequest::new(i as u64, model.to_string(), prompt.to_string())
            .with_max_tokens(50);

        let iter_start = Instant::now();
        let response = vllm_engine.generate(request).await?;
        let iter_duration = iter_start.elapsed();

        latencies.push(iter_duration.as_millis() as f64);
        total_tokens += response.stats.generated_tokens;
    }

    let total_duration = start.elapsed();

    let mut sorted_latencies = latencies.clone();
    sorted_latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let median_latency = calculate_percentile(&sorted_latencies, 50.0);
    let p99_latency = calculate_percentile(&sorted_latencies, 99.0);
    let avg_latency = latencies.iter().sum::<f64>() / latencies.len() as f64;
    let tokens_per_sec = total_tokens as f64 / total_duration.as_secs_f64();
    let requests_per_sec = iterations as f64 / total_duration.as_secs_f64();

    Ok(BenchStats {
        median_latency_ms: median_latency,
        avg_latency_ms: avg_latency,
        p99_latency_ms: p99_latency,
        tokens_per_sec,
        total_time_secs: total_duration.as_secs_f64(),
        requests_per_sec,
    }.into())
}

async fn test_vllm_concurrent(model: &str, prompt: &str, total_requests: usize, concurrency: usize) -> Result<EngineStats> {
    let vllm_engine = VllmOpenAIEngine::new("http://127.0.0.1:8100");

    if !vllm_engine.health_check().await? {
        anyhow::bail!("vLLM OpenAI server not available");
    }

    let mut latencies = Vec::new();
    let mut total_tokens = 0usize;
    let start = Instant::now();

    let mut tasks = JoinSet::new();
    let mut request_id = 0u64;

    // Launch concurrent requests
    for batch_start in (0..total_requests).step_by(concurrency) {
        let batch_end = (batch_start + concurrency).min(total_requests);

        for _ in batch_start..batch_end {
            let model_clone = model.to_string();
            let prompt_clone = prompt.to_string();
            let req_id = request_id;
            request_id += 1;

            tasks.spawn(async move {
                let engine = VllmOpenAIEngine::new("http://127.0.0.1:8100");
                let request = GenerateRequest::new(req_id, model_clone, prompt_clone)
                    .with_max_tokens(50);

                let iter_start = Instant::now();
                let response = engine.generate(request).await?;
                let iter_duration = iter_start.elapsed();

                Ok::<(f64, usize), anyhow::Error>((iter_duration.as_millis() as f64, response.stats.generated_tokens))
            });
        }

        // Wait for this batch to complete
        while let Some(result) = tasks.join_next().await {
            match result {
                Ok(Ok((latency, tokens))) => {
                    latencies.push(latency);
                    total_tokens += tokens;
                }
                Ok(Err(e)) => return Err(e),
                Err(e) => return Err(anyhow::anyhow!("Task join error: {}", e)),
            }
        }
    }

    let total_duration = start.elapsed();

    let mut sorted_latencies = latencies.clone();
    sorted_latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let median_latency = calculate_percentile(&sorted_latencies, 50.0);
    let p99_latency = calculate_percentile(&sorted_latencies, 99.0);
    let avg_latency = latencies.iter().sum::<f64>() / latencies.len() as f64;
    let tokens_per_sec = total_tokens as f64 / total_duration.as_secs_f64();
    let requests_per_sec = total_requests as f64 / total_duration.as_secs_f64();

    Ok(BenchStats {
        median_latency_ms: median_latency,
        avg_latency_ms: avg_latency,
        p99_latency_ms: p99_latency,
        tokens_per_sec,
        total_time_secs: total_duration.as_secs_f64(),
        requests_per_sec,
    }.into())
}

fn translate_to_ollama_model(hf_model: &str) -> &str {
    match hf_model {
        "meta-llama/Llama-3.1-8B-Instruct" => "llama3.1:8b",
        "meta-llama/Llama-3.2-3B-Instruct" => "llama3.2:3b",
        "meta-llama/Llama-3.2-1B-Instruct" => "llama3.2:1b",
        "Qwen/Qwen2.5-1.5B-Instruct" => "qwen2.5:1.5b",
        "Qwen/Qwen2.5-7B-Instruct" => "qwen2.5:7b",
        _ => hf_model,
    }
}

async fn test_ollama_sequential(model: &str, prompt: &str, iterations: usize) -> Result<EngineStats> {
    let client = reqwest::Client::new();
    let ollama_model = translate_to_ollama_model(model);

    #[derive(serde::Serialize)]
    struct OllamaRequest {
        model: String,
        prompt: String,
        stream: bool,
        options: OllamaOptions,
    }

    #[derive(serde::Serialize)]
    struct OllamaOptions {
        num_predict: i32,
    }

    #[derive(serde::Deserialize)]
    struct OllamaResponse {
        eval_count: Option<usize>,
    }

    let mut latencies = Vec::new();
    let mut total_tokens = 0usize;
    let start = Instant::now();

    for _ in 0..iterations {
        let request = OllamaRequest {
            model: ollama_model.to_string(),
            prompt: prompt.to_string(),
            stream: false,
            options: OllamaOptions {
                num_predict: 50,
            },
        };

        let iter_start = Instant::now();
        let response = client
            .post("http://localhost:11435/api/generate")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Ollama request failed: {}", response.status());
        }

        let ollama_resp: OllamaResponse = response.json().await?;
        let iter_duration = iter_start.elapsed();

        latencies.push(iter_duration.as_millis() as f64);
        if let Some(count) = ollama_resp.eval_count {
            total_tokens += count;
        }
    }

    let total_duration = start.elapsed();

    let mut sorted_latencies = latencies.clone();
    sorted_latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let median_latency = calculate_percentile(&sorted_latencies, 50.0);
    let p99_latency = calculate_percentile(&sorted_latencies, 99.0);
    let avg_latency = latencies.iter().sum::<f64>() / latencies.len() as f64;
    let tokens_per_sec = total_tokens as f64 / total_duration.as_secs_f64();
    let requests_per_sec = iterations as f64 / total_duration.as_secs_f64();

    Ok(BenchStats {
        median_latency_ms: median_latency,
        avg_latency_ms: avg_latency,
        p99_latency_ms: p99_latency,
        tokens_per_sec,
        total_time_secs: total_duration.as_secs_f64(),
        requests_per_sec,
    }.into())
}

async fn test_ollama_concurrent(model: &str, prompt: &str, total_requests: usize, concurrency: usize) -> Result<EngineStats> {
    let client = reqwest::Client::new();
    let ollama_model = translate_to_ollama_model(model);

    #[derive(serde::Serialize)]
    struct OllamaRequest {
        model: String,
        prompt: String,
        stream: bool,
        options: OllamaOptions,
    }

    #[derive(serde::Serialize)]
    struct OllamaOptions {
        num_predict: i32,
    }

    #[derive(serde::Deserialize)]
    struct OllamaResponse {
        eval_count: Option<usize>,
    }

    let mut latencies = Vec::new();
    let mut total_tokens = 0usize;
    let start = Instant::now();

    let mut tasks = JoinSet::new();

    // Launch concurrent requests
    for batch_start in (0..total_requests).step_by(concurrency) {
        let batch_end = (batch_start + concurrency).min(total_requests);

        for _ in batch_start..batch_end {
            let client_clone = client.clone();
            let model_clone = ollama_model.to_string();
            let prompt_clone = prompt.to_string();

            tasks.spawn(async move {
                let request = OllamaRequest {
                    model: model_clone,
                    prompt: prompt_clone,
                    stream: false,
                    options: OllamaOptions {
                        num_predict: 50,
                    },
                };

                let iter_start = Instant::now();
                let response = client_clone
                    .post("http://localhost:11435/api/generate")
                    .json(&request)
                    .send()
                    .await?;

                if !response.status().is_success() {
                    anyhow::bail!("Ollama request failed: {}", response.status());
                }

                let ollama_resp: OllamaResponse = response.json().await?;
                let iter_duration = iter_start.elapsed();

                let tokens = ollama_resp.eval_count.unwrap_or(0);

                Ok::<(f64, usize), anyhow::Error>((iter_duration.as_millis() as f64, tokens))
            });
        }

        // Wait for this batch to complete
        while let Some(result) = tasks.join_next().await {
            match result {
                Ok(Ok((latency, tokens))) => {
                    latencies.push(latency);
                    total_tokens += tokens;
                }
                Ok(Err(e)) => return Err(e),
                Err(e) => return Err(anyhow::anyhow!("Task join error: {}", e)),
            }
        }
    }

    let total_duration = start.elapsed();

    let mut sorted_latencies = latencies.clone();
    sorted_latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let median_latency = calculate_percentile(&sorted_latencies, 50.0);
    let p99_latency = calculate_percentile(&sorted_latencies, 99.0);
    let avg_latency = latencies.iter().sum::<f64>() / latencies.len() as f64;
    let tokens_per_sec = total_tokens as f64 / total_duration.as_secs_f64();
    let requests_per_sec = total_requests as f64 / total_duration.as_secs_f64();

    Ok(BenchStats {
        median_latency_ms: median_latency,
        avg_latency_ms: avg_latency,
        p99_latency_ms: p99_latency,
        tokens_per_sec,
        total_time_secs: total_duration.as_secs_f64(),
        requests_per_sec,
    }.into())
}
