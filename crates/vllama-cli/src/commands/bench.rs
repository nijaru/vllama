use anyhow::Result;
use vllama_core::{GenerateRequest, Hardware};
use vllama_engine::{InferenceEngine, VllmOpenAIEngine};
use std::time::Instant;
use tracing::warn;

pub async fn execute(model: String, prompt: String, iterations: usize) -> Result<()> {
    println!("vLLama Benchmark");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Model: {}", model);
    println!("Prompt: {}", prompt);
    println!("Iterations: {}", iterations);
    println!();

    let hw = Hardware::detect();
    println!("Hardware: {:?}", hw.hw_type);
    println!("CPU Cores: {}", hw.cpu_cores);
    println!("RAM: {} MB", hw.ram_total_mb);
    println!();

    println!("⚠️  Benchmark Caveats:");
    println!("  - vLLama: OpenAI API on port 8100");
    println!("  - Ollama: HTTP API on port 11435 (not default 11434)");
    println!("  - Both limited to 50 tokens per response");
    println!("  - Run vLLM: python -m vllm.entrypoints.openai.api_server --model <model> --port 8100");
    println!("  - Run Ollama on port 11435: OLLAMA_HOST=127.0.0.1:11435 ollama serve");
    println!();

    println!("Testing vLLama (via inference engine)...");
    match test_max_engine(&model, &prompt, iterations).await {
        Ok(stats) => {
            println!("✓ vLLama Results:");
            println!("  Median latency: {:.2}ms", stats.median_latency_ms);
            println!("  Average latency: {:.2}ms", stats.avg_latency_ms);
            println!("  P99 latency: {:.2}ms", stats.p99_latency_ms);
            println!("  Tokens/sec: {:.2}", stats.tokens_per_sec);
            println!("  Total time: {:.2}s", stats.total_time_secs);
        }
        Err(e) => {
            warn!("✗ vLLama test failed: {}", e);
            println!("✗ vLLama: {}", e);
        }
    }

    println!();
    println!("Testing Ollama (for comparison)...");
    match test_ollama(&model, &prompt, iterations).await {
        Ok(stats) => {
            println!("✓ Ollama Results:");
            println!("  Median latency: {:.2}ms", stats.median_latency_ms);
            println!("  Average latency: {:.2}ms", stats.avg_latency_ms);
            println!("  P99 latency: {:.2}ms", stats.p99_latency_ms);
            println!("  Tokens/sec: {:.2}", stats.tokens_per_sec);
            println!("  Total time: {:.2}s", stats.total_time_secs);
        }
        Err(e) => {
            warn!("✗ Ollama test failed: {}", e);
            println!("✗ Ollama: Not available (run on port 11435)");
        }
    }

    Ok(())
}

#[derive(Debug)]
struct BenchStats {
    median_latency_ms: f64,
    avg_latency_ms: f64,
    p99_latency_ms: f64,
    tokens_per_sec: f64,
    total_time_secs: f64,
}

fn calculate_percentile(sorted_values: &[f64], percentile: f64) -> f64 {
    let index = (percentile / 100.0 * (sorted_values.len() - 1) as f64).round() as usize;
    sorted_values[index]
}

async fn test_max_engine(model: &str, prompt: &str, iterations: usize) -> Result<BenchStats> {
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

    Ok(BenchStats {
        median_latency_ms: median_latency,
        avg_latency_ms: avg_latency,
        p99_latency_ms: p99_latency,
        tokens_per_sec,
        total_time_secs: total_duration.as_secs_f64(),
    })
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

async fn test_ollama(model: &str, prompt: &str, iterations: usize) -> Result<BenchStats> {
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

    Ok(BenchStats {
        median_latency_ms: median_latency,
        avg_latency_ms: avg_latency,
        p99_latency_ms: p99_latency,
        tokens_per_sec,
        total_time_secs: total_duration.as_secs_f64(),
    })
}
