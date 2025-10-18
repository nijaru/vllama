use anyhow::Result;
use vllama_core::{GenerateRequest, Hardware};
use vllama_engine::{InferenceEngine, MaxEngine};
use std::path::PathBuf;
use std::time::Instant;
use tracing::{info, warn};

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
    println!("  - vLLama: Direct engine access (minimal overhead)");
    println!("  - Ollama: HTTP API on port 11435 (not default 11434)");
    println!("  - Both limited to 50 tokens per response");
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
    let mut max_engine = MaxEngine::new()?;

    if !max_engine.health_check().await? {
        anyhow::bail!("Inference service not available (is the Python vLLM service running on port 8100?)");
    }

    info!("Loading model via inference engine");
    let model_path = PathBuf::from(model);
    let handle = max_engine.load_model(&model_path).await?;

    let model_id = max_engine
        .get_model_id(handle)
        .ok_or_else(|| anyhow::anyhow!("Model handle not found"))?;

    let mut latencies = Vec::new();
    let mut total_tokens = 0usize;
    let start = Instant::now();

    for i in 0..iterations {
        let request = GenerateRequest::new(i as u64, model_id.clone(), prompt.to_string())
            .with_max_tokens(50);

        let iter_start = Instant::now();
        let response = max_engine.generate(request).await?;
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

async fn test_ollama(model: &str, prompt: &str, iterations: usize) -> Result<BenchStats> {
    let client = reqwest::Client::new();

    #[derive(serde::Serialize)]
    struct OllamaRequest {
        model: String,
        prompt: String,
        stream: bool,
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
            model: model.to_string(),
            prompt: prompt.to_string(),
            stream: false,
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
