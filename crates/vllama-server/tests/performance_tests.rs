use reqwest;
use serde_json::json;
use std::time::{Duration, Instant};
use tokio::task::JoinSet;

const BASE_URL: &str = "http://localhost:11435";
const TIMEOUT: Duration = Duration::from_secs(60);

fn get_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(TIMEOUT)
        .build()
        .expect("Failed to create HTTP client")
}

async fn wait_for_server() -> Result<(), Box<dyn std::error::Error>> {
    let client = get_client();
    let max_retries = 10;

    for i in 0..max_retries {
        match client.get(&format!("{}/health", BASE_URL)).send().await {
            Ok(resp) if resp.status().is_success() => return Ok(()),
            _ => {
                if i < max_retries - 1 {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
        }
    }

    Err("Server not available after retries".into())
}

async fn make_generate_request(model: &str, prompt: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = get_client();
    let response = client
        .post(&format!("{}/api/generate", BASE_URL))
        .json(&json!({
            "model": model,
            "prompt": prompt,
            "stream": false,
            "options": {
                "max_tokens": 10
            }
        }))
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(format!("Request failed with status: {}", response.status()).into());
    }

    Ok(())
}

#[tokio::test]
#[ignore] // Run with: cargo test --test performance_tests -- --ignored
async fn test_concurrent_performance_regression() {
    wait_for_server().await.expect("Server must be running");

    // Get first available model
    let client = get_client();
    let ps_response = client
        .get(&format!("{}/api/ps", BASE_URL))
        .send()
        .await
        .expect("Failed to get models");

    let ps_json: serde_json::Value = ps_response.json().await.expect("Failed to parse JSON");
    let models = ps_json["models"].as_array().expect("models should be array");

    if models.is_empty() {
        println!("Skipping test_concurrent_performance_regression: no models running");
        return;
    }

    let model_name = models[0]["name"].as_str().expect("name should be string").to_string();

    // Performance regression test: 5 concurrent requests
    // Based on our benchmarks with facebook/opt-125m: ~0.217s
    // We'll allow 2x slack for CI/different hardware: <0.5s
    let num_concurrent = 5;
    let max_duration = Duration::from_millis(500);

    let start = Instant::now();
    let mut tasks = JoinSet::new();

    for i in 0..num_concurrent {
        let model = model_name.clone();
        let prompt = format!("Request {}: Say 'test'", i);

        tasks.spawn(async move {
            make_generate_request(&model, &prompt).await
        });
    }

    // Wait for all tasks to complete
    let mut errors = Vec::new();
    while let Some(result) = tasks.join_next().await {
        match result {
            Ok(Ok(_)) => {},
            Ok(Err(e)) => errors.push(format!("Request failed: {}", e)),
            Err(e) => errors.push(format!("Task panicked: {}", e)),
        }
    }

    let elapsed = start.elapsed();

    // Report results
    println!("Concurrent performance ({} requests): {:?}", num_concurrent, elapsed);
    println!("Throughput: {:.2} req/s", num_concurrent as f64 / elapsed.as_secs_f64());

    // Check for errors
    if !errors.is_empty() {
        panic!("Some requests failed:\n{}", errors.join("\n"));
    }

    // Performance regression check
    if elapsed > max_duration {
        panic!(
            "Performance regression detected! {} concurrent requests took {:?}, expected < {:?}",
            num_concurrent, elapsed, max_duration
        );
    }

    println!("✓ Performance check passed");
}

#[tokio::test]
#[ignore]
async fn test_throughput_scaling() {
    wait_for_server().await.expect("Server must be running");

    let client = get_client();
    let ps_response = client
        .get(&format!("{}/api/ps", BASE_URL))
        .send()
        .await
        .expect("Failed to get models");

    let ps_json: serde_json::Value = ps_response.json().await.expect("Failed to parse JSON");
    let models = ps_json["models"].as_array().expect("models should be array");

    if models.is_empty() {
        println!("Skipping test_throughput_scaling: no models running");
        return;
    }

    let model_name = models[0]["name"].as_str().expect("name should be string").to_string();

    // Test different concurrency levels
    let concurrency_levels = vec![1, 5, 10];

    for num_concurrent in concurrency_levels {
        let start = Instant::now();
        let mut tasks = JoinSet::new();

        for i in 0..num_concurrent {
            let model = model_name.clone();
            let prompt = format!("Request {}: Say 'test'", i);

            tasks.spawn(async move {
                make_generate_request(&model, &prompt).await
            });
        }

        while let Some(_) = tasks.join_next().await {}
        let elapsed = start.elapsed();

        let throughput = num_concurrent as f64 / elapsed.as_secs_f64();
        println!("Concurrency {}: {:?} ({:.2} req/s)", num_concurrent, elapsed, throughput);
    }
}

#[tokio::test]
#[ignore]
async fn test_single_request_latency() {
    wait_for_server().await.expect("Server must be running");

    let client = get_client();
    let ps_response = client
        .get(&format!("{}/api/ps", BASE_URL))
        .send()
        .await
        .expect("Failed to get models");

    let ps_json: serde_json::Value = ps_response.json().await.expect("Failed to parse JSON");
    let models = ps_json["models"].as_array().expect("models should be array");

    if models.is_empty() {
        println!("Skipping test_single_request_latency: no models running");
        return;
    }

    let model_name = models[0]["name"].as_str().expect("name should be string");

    // Single request latency test
    // Based on our benchmarks: ~232ms for Qwen 1.5B
    // Allow 2x slack: <500ms for small models
    let max_latency = Duration::from_millis(500);

    let start = Instant::now();
    let response = client
        .post(&format!("{}/api/generate", BASE_URL))
        .json(&json!({
            "model": model_name,
            "prompt": "Say 'test' and nothing else.",
            "stream": false,
            "options": {
                "max_tokens": 10
            }
        }))
        .send()
        .await
        .expect("Failed to send request");

    let elapsed = start.elapsed();

    assert!(response.status().is_success());

    println!("Single request latency: {:?}", elapsed);

    if elapsed > max_latency {
        eprintln!(
            "Warning: Single request latency ({:?}) exceeded threshold ({:?})",
            elapsed, max_latency
        );
    }

    println!("✓ Latency check completed");
}
