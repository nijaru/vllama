use reqwest;
use serde_json::json;
use std::time::Duration;

const BASE_URL: &str = "http://localhost:11435";
const TIMEOUT: Duration = Duration::from_secs(30);

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

#[tokio::test]
#[ignore] // Run with: cargo test -- --ignored --test-threads=1
async fn test_health_endpoint() {
    wait_for_server().await.expect("Server must be running");

    let client = get_client();
    let response = client
        .get(&format!("{}/health", BASE_URL))
        .send()
        .await
        .expect("Failed to send request");

    assert!(response.status().is_success());
    let json: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    assert_eq!(json["status"], "ok");
    assert!(json.get("vllm_status").is_some());
    assert!(json.get("gpu").is_some());
}

#[tokio::test]
#[ignore]
async fn test_version_endpoint() {
    wait_for_server().await.expect("Server must be running");

    let client = get_client();
    let response = client
        .get(&format!("{}/api/version", BASE_URL))
        .send()
        .await
        .expect("Failed to send request");

    assert!(response.status().is_success());

    let json: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    assert!(json.get("version").is_some());
    assert_eq!(json["version"], "0.0.5");
}

#[tokio::test]
#[ignore]
async fn test_ps_endpoint() {
    wait_for_server().await.expect("Server must be running");

    let client = get_client();
    let response = client
        .get(&format!("{}/api/ps", BASE_URL))
        .send()
        .await
        .expect("Failed to send request");

    assert!(response.status().is_success());

    let json: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    assert!(json.get("models").is_some());
    assert!(json["models"].is_array());

    // If vLLM is running with a model, we should have at least one model
    if json["models"].as_array().unwrap().len() > 0 {
        let model = &json["models"][0];
        assert!(model.get("name").is_some());
        assert!(model.get("model").is_some());
        assert!(model.get("details").is_some());
    }
}

#[tokio::test]
#[ignore]
async fn test_show_endpoint() {
    wait_for_server().await.expect("Server must be running");

    let client = get_client();

    // First get the list of models
    let ps_response = client
        .get(&format!("{}/api/ps", BASE_URL))
        .send()
        .await
        .expect("Failed to get models");

    let ps_json: serde_json::Value = ps_response.json().await.expect("Failed to parse JSON");
    let models = ps_json["models"].as_array().expect("models should be array");

    if models.is_empty() {
        println!("Skipping test_show_endpoint: no models running");
        return;
    }

    let model_name = models[0]["name"].as_str().expect("name should be string");

    let response = client
        .post(&format!("{}/api/show", BASE_URL))
        .json(&json!({
            "model": model_name
        }))
        .send()
        .await
        .expect("Failed to send request");

    assert!(response.status().is_success());

    let json: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    assert!(json.get("modelfile").is_some());
    assert!(json.get("parameters").is_some());
    assert!(json.get("details").is_some());

    let details = &json["details"];
    assert!(details.get("format").is_some());
    assert!(details.get("family").is_some());
    assert!(details.get("parameter_size").is_some());
}

#[tokio::test]
#[ignore]
async fn test_show_endpoint_not_found() {
    wait_for_server().await.expect("Server must be running");

    let client = get_client();
    let response = client
        .post(&format!("{}/api/show", BASE_URL))
        .json(&json!({
            "model": "nonexistent-model-12345"
        }))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 404);

    let json: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    assert!(json.get("error").is_some());
}

#[tokio::test]
#[ignore]
async fn test_generate_endpoint_non_streaming() {
    wait_for_server().await.expect("Server must be running");

    let client = get_client();

    // Get first available model
    let ps_response = client
        .get(&format!("{}/api/ps", BASE_URL))
        .send()
        .await
        .expect("Failed to get models");

    let ps_json: serde_json::Value = ps_response.json().await.expect("Failed to parse JSON");
    let models = ps_json["models"].as_array().expect("models should be array");

    if models.is_empty() {
        println!("Skipping test_generate_endpoint_non_streaming: no models running");
        return;
    }

    let model_name = models[0]["name"].as_str().expect("name should be string");

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

    assert!(response.status().is_success());

    let json: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    assert!(json.get("model").is_some());
    assert!(json.get("response").is_some());
    assert!(json.get("done").is_some());
    assert_eq!(json["done"], true);
    assert!(json["response"].as_str().unwrap().len() > 0);
}

#[tokio::test]
#[ignore]
async fn test_chat_endpoint_non_streaming() {
    wait_for_server().await.expect("Server must be running");

    let client = get_client();

    // Get first available model
    let ps_response = client
        .get(&format!("{}/api/ps", BASE_URL))
        .send()
        .await
        .expect("Failed to get models");

    let ps_json: serde_json::Value = ps_response.json().await.expect("Failed to parse JSON");
    let models = ps_json["models"].as_array().expect("models should be array");

    if models.is_empty() {
        println!("Skipping test_chat_endpoint_non_streaming: no models running");
        return;
    }

    let model_name = models[0]["name"].as_str().expect("name should be string");

    let response = client
        .post(&format!("{}/api/chat", BASE_URL))
        .json(&json!({
            "model": model_name,
            "messages": [
                {
                    "role": "user",
                    "content": "Say 'test' and nothing else."
                }
            ],
            "stream": false,
            "options": {
                "max_tokens": 10
            }
        }))
        .send()
        .await
        .expect("Failed to send request");

    // Some models (like facebook/opt-125m) don't have chat templates
    // This is expected and the API should return an error
    let status = response.status();
    if !status.is_success() {
        let json: serde_json::Value = response.json().await.expect("Failed to parse JSON");
        if let Some(error) = json.get("error").and_then(|e| e.as_str()) {
            if error.contains("chat template") {
                println!("Skipping test_chat_endpoint_non_streaming: model {} doesn't support chat", model_name);
                return;
            }
        }
        panic!("Chat request failed with status {}: {:?}", status, json);
    }

    let json: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    assert!(json.get("model").is_some());
    assert!(json.get("message").is_some());
    assert!(json.get("done").is_some());
    assert_eq!(json["done"], true);

    let message = &json["message"];
    assert!(message.get("role").is_some());
    assert!(message.get("content").is_some());
    assert_eq!(message["role"], "assistant");
    assert!(message["content"].as_str().unwrap().len() > 0);
}

#[tokio::test]
#[ignore]
async fn test_openai_chat_completions() {
    wait_for_server().await.expect("Server must be running");

    let client = get_client();

    // Get first available model
    let ps_response = client
        .get(&format!("{}/api/ps", BASE_URL))
        .send()
        .await
        .expect("Failed to get models");

    let ps_json: serde_json::Value = ps_response.json().await.expect("Failed to parse JSON");
    let models = ps_json["models"].as_array().expect("models should be array");

    if models.is_empty() {
        println!("Skipping test_openai_chat_completions: no models running");
        return;
    }

    let model_name = models[0]["name"].as_str().expect("name should be string");

    let response = client
        .post(&format!("{}/v1/chat/completions", BASE_URL))
        .json(&json!({
            "model": model_name,
            "messages": [
                {
                    "role": "user",
                    "content": "Say 'test' and nothing else."
                }
            ],
            "stream": false,
            "max_tokens": 10
        }))
        .send()
        .await
        .expect("Failed to send request");

    assert!(response.status().is_success());

    let json: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    assert!(json.get("id").is_some());
    assert!(json.get("object").is_some());
    assert_eq!(json["object"], "chat.completion");
    assert!(json.get("choices").is_some());

    let choices = json["choices"].as_array().expect("choices should be array");
    assert!(choices.len() > 0);

    let choice = &choices[0];
    assert!(choice.get("message").is_some());
    assert!(choice.get("finish_reason").is_some());

    let message = &choice["message"];
    assert!(message.get("role").is_some());
    assert!(message.get("content").is_some());
}
