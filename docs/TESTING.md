# Testing Guide

This document describes the testing infrastructure and how to run tests for vllama.

## Test Types

### 1. Unit Tests

Basic unit tests for individual components. Run with:

```bash
cargo test
```

**Coverage:**
- ✅ Object creation and initialization
- ✅ Serialization/deserialization
- ⚠️ Limited business logic coverage (needs improvement)

### 2. Integration Tests

Tests that verify API endpoints work correctly with a running server.

**Location:** `crates/vllama-server/tests/api_tests.rs`

**Prerequisites:**
- vllama server must be running
- At least one model loaded in vLLM

**Run:**
```bash
# Start server first
cargo build --release
./target/release/vllama serve --model facebook/opt-125m

# In another terminal, run tests
cargo test --package vllama-server --test api_tests -- --ignored --test-threads=1
```

**Test Coverage:**
- ✅ `/health` - Health check endpoint
- ✅ `/api/version` - Version information
- ✅ `/api/ps` - List running models
- ✅ `/api/show` - Model metadata
- ✅ `/api/show` (error case) - Model not found
- ✅ `/api/generate` (non-streaming) - Text generation
- ✅ `/api/chat` (non-streaming) - Chat completions
- ✅ `/v1/chat/completions` - OpenAI-compatible API

### 3. Performance Regression Tests

Automated tests that verify performance doesn't degrade.

**Location:** `crates/vllama-server/tests/performance_tests.rs`

**Prerequisites:**
- vllama server must be running
- At least one model loaded in vLLM
- GPU available for best results

**Run:**
```bash
cargo test --package vllama-server --test performance_tests -- --ignored --test-threads=1
```

**Test Coverage:**
- ✅ **Concurrent Performance Regression** - Ensures 5 concurrent requests complete in <500ms
  - Baseline: ~0.217s on RTX 4090 with facebook/opt-125m
  - Threshold: <500ms (2x slack for CI/different hardware)
- ✅ **Throughput Scaling** - Tests 1, 5, 10 concurrent requests
  - Verifies throughput scales with concurrency
- ✅ **Single Request Latency** - Measures latency of a single request
  - Warning threshold: <500ms for small models

## Running All Tests

```bash
# Run unit tests
cargo test

# Start server
cargo build --release
./target/release/vllama serve --model facebook/opt-125m

# In another terminal: Run integration tests
cargo test --package vllama-server --test api_tests -- --ignored --test-threads=1

# Run performance tests
cargo test --package vllama-server --test performance_tests -- --ignored --test-threads=1
```

## Test Results

**Current Status (2025-10-22):**
- Unit Tests: 8 passed
- Integration Tests: 8 passed
- Performance Tests: 3 passed

**Total:** 19 tests passing ✅

## CI/CD Integration (TODO)

Future improvements needed:

1. **GitHub Actions Workflow:**
   - Run unit tests on every PR
   - Run integration tests with mock vLLM server
   - Performance benchmarks on release

2. **Test Coverage:**
   - Add code coverage reporting (tarpaulin or grcov)
   - Target: >70% coverage

3. **Additional Tests:**
   - Streaming endpoint tests
   - Model download tests (/api/pull)
   - Error handling edge cases
   - Concurrent stress tests (50+, 100+ requests)

## Writing New Tests

### Integration Test Template

```rust
#[tokio::test]
#[ignore]
async fn test_my_endpoint() {
    wait_for_server().await.expect("Server must be running");

    let client = get_client();
    let response = client
        .get(&format!("{}/my/endpoint", BASE_URL))
        .send()
        .await
        .expect("Failed to send request");

    assert!(response.status().is_success());

    let json: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    // Add assertions here
}
```

### Performance Test Template

```rust
#[tokio::test]
#[ignore]
async fn test_my_performance() {
    wait_for_server().await.expect("Server must be running");

    let start = Instant::now();
    // Perform operation
    let elapsed = start.elapsed();

    let threshold = Duration::from_millis(100);
    if elapsed > threshold {
        panic!("Performance regression: {:?} > {:?}", elapsed, threshold);
    }
}
```

## Troubleshooting

### "Server not available after retries"
- Make sure the server is running: `curl http://localhost:11435/health`
- Check server logs for errors
- Verify vLLM is running on port 8100

### "No models running"
- Tests require at least one model loaded
- Start server with: `./target/release/vllama serve --model <model>`
- Verify with: `curl http://localhost:11435/api/ps`

### Performance tests failing
- Performance thresholds are tuned for RTX 4090
- May need to adjust thresholds for different hardware
- Run tests with `--nocapture` to see timing details:
  ```bash
  cargo test --test performance_tests -- --ignored --nocapture
  ```

### Chat endpoint test skipped
- Some models (e.g., facebook/opt-125m) don't have chat templates
- This is expected - the test gracefully skips
- Use a model with chat support to test chat functionality
  (e.g., Llama, Qwen, Mistral)

## Test Organization

```
vllama/
├── crates/
│   └── vllama-server/
│       ├── src/
│       │   └── lib.rs        # Unit tests inline with code
│       └── tests/
│           ├── api_tests.rs  # Integration tests (8 tests)
│           └── performance_tests.rs  # Performance regression (3 tests)
└── docs/
    └── TESTING.md  # This file
```

## Performance Baselines

**Hardware:** RTX 4090, i9-13900KF, 32GB DDR5
**Model:** facebook/opt-125m

| Metric | Value | Threshold |
|--------|-------|-----------|
| Single request latency | ~50-100ms | <500ms |
| 5 concurrent requests | ~0.217s | <500ms |
| Throughput (5 concurrent) | ~23 req/s | >10 req/s |

**Note:** Thresholds have 2-5x slack for CI/different hardware.

## Future Improvements

- [ ] Add streaming endpoint tests
- [ ] Mock vLLM server for CI
- [ ] Test /api/pull with mock HuggingFace
- [ ] Error injection tests
- [ ] Load testing (100+, 1000+ requests)
- [ ] Memory leak detection
- [ ] Long-running stability tests
- [ ] Cross-platform testing (macOS, Windows)
