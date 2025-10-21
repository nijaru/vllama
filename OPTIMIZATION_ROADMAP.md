# vLLama Optimization Roadmap

**Date:** 2025-10-20
**Goal:** Beat Ollama everywhere with matched functionality

---

## Current Status

### Performance (from benchmarks)

| Workload | vLLama (vLLM) | Ollama | Status |
|----------|---------------|---------|---------|
| Sequential | 232ms | 1,010ms | ✅ **4.4x faster** |
| Concurrent (5 parallel) | 7.57s | 6.50s | ❌ **1.16x slower** |
| Streaming | 0.617s | 1.015s | ✅ **1.6x faster** |

**Problem:** Concurrent requests are slower than Ollama!

### Functionality (Ollama API compatibility)

| Endpoint | Status | Notes |
|----------|--------|-------|
| POST /api/generate | ✅ | Streaming + non-streaming |
| POST /api/chat | ✅ | With proper chat templates |
| POST /api/pull | ✅ | HuggingFace downloads |
| GET /api/tags | ✅ | List models |
| GET /api/show | ⚠️ | Limited metadata |
| GET /api/ps | ⚠️ | Stub only |
| POST /v1/chat/completions | ✅ | OpenAI compat |
| POST /api/embeddings | ❌ | **Missing** |
| POST /api/copy | ❌ | Missing |
| DELETE /api/delete | ❌ | Missing |
| POST /api/create | ❌ | Missing (Modelfile) |
| POST /api/push | ❌ | Missing |
| GET /api/version | ❌ | Missing |

**Problem:** Missing 6 endpoints that Ollama has!

---

## Root Cause Analysis

### 1. Concurrent Performance Issue

**Current vLLM config:**
```bash
python -m vllm.entrypoints.openai.api_server \
  --model MODEL \
  --port 8100 \
  --max-num-seqs 256 \           # Default
  --gpu-memory-utilization 0.9   # Only 2 params!
```

**Missing critical optimization params:**
- ❌ No `--max-num-batched-tokens` (controls batch size)
- ❌ No `--enable-chunked-prefill` (V1 default, but good to be explicit)
- ❌ No `--enable-prefix-caching` (reuse KV cache)
- ❌ No `--tensor-parallel-size` (multi-GPU)
- ❌ No `--max-model-len` tuning

**vLLM V1 (2025) key features we're not using:**
- Chunked prefill for better concurrent handling
- Prefix caching for repeated prompts
- Better batch token management

---

### 2. Missing Functionality

**High priority (common use cases):**
1. `/api/embeddings` - Many apps need embeddings
2. `/api/version` - Basic info
3. `/api/ps` - Show running models properly

**Medium priority:**
4. `/api/copy` - Model management
5. `/api/delete` - Model cleanup

**Low priority (advanced features):**
6. `/api/create` - Custom Modelfiles
7. `/api/push` - Model sharing

---

## Optimization Plan

### Phase 1: Fix Concurrent Performance (Priority 1) 🔥

**Goal:** Beat Ollama on concurrent requests

**Changes:**

1. **Add chunked prefill params**
```rust
// serve.rs
.args([
    "--enable-chunked-prefill",
    "--max-num-batched-tokens", "8192",  // Higher = better throughput
])
```

2. **Add prefix caching**
```rust
.args([
    "--enable-prefix-caching",  // Reuse KV cache for repeated prompts
])
```

3. **Tune max-num-seqs based on model**
```rust
// Current: hardcoded 256
// Better: scale with model size
let max_seqs = match model_size {
    Small => 512,     // < 3B params
    Medium => 256,    // 3-13B params
    Large => 128,     // 13-70B params
    XLarge => 64,     // > 70B params
};
```

4. **Add max-model-len tuning**
```rust
.args([
    "--max-model-len", "4096",  // Optimize for typical use
])
```

**Expected impact:** 2-3x faster concurrent requests (beat Ollama)

**Effort:** 1 day

---

### Phase 2: Add Embeddings Support (Priority 2)

**Goal:** Support `/api/embeddings` endpoint

**Implementation:**
```rust
// crates/vllama-server/src/api.rs
pub async fn embeddings(
    State(state): State<ServerState>,
    Json(req): Json<EmbeddingsRequest>,
) -> Result<Json<EmbeddingsResponse>, (StatusCode, String)> {
    // Call vLLM embeddings endpoint
    let client = reqwest::Client::new();
    let response = client
        .post("http://127.0.0.1:8100/v1/embeddings")
        .json(&req)
        .send()
        .await?;

    // Transform to Ollama format
    Ok(Json(transform_embeddings(response)))
}
```

**Required:**
- Add EmbeddingsRequest/Response types
- Route to vLLM's `/v1/embeddings`
- Transform OpenAI format → Ollama format
- Tests

**Effort:** 0.5 day

---

### Phase 3: Improve /api/ps and /api/show (Priority 3)

**Current `/api/ps` implementation:**
```rust
pub async fn ps(State(_state): State<ServerState>) -> Response {
    (StatusCode::OK, Json(json!({ "models": [] }))).into_response()
}
```

**Problem:** Returns empty array, not useful!

**Better implementation:**
```rust
pub async fn ps(State(state): State<ServerState>) -> Response {
    // Query vLLM for loaded models
    let client = reqwest::Client::new();
    let models = client
        .get("http://127.0.0.1:8100/v1/models")
        .send()
        .await
        .and_then(|r| r.json())
        .unwrap_or_default();

    // Transform to Ollama format with actual data
    (StatusCode::OK, Json(transform_models_list(models))).into_response()
}
```

**For `/api/show`:**
- Query vLLM model info
- Return proper model card, parameters, template, etc.

**Effort:** 0.5 day

---

### Phase 4: Add Version and Model Management (Priority 4)

**Simple additions:**

1. **GET /api/version**
```rust
pub async fn version() -> Json<VersionResponse> {
    Json(VersionResponse {
        version: env!("CARGO_PKG_VERSION").to_string(),
        backend: "vLLM".to_string(),
        backend_version: "0.11.0".to_string(),
    })
}
```

2. **POST /api/copy**
```rust
// Copy model in HuggingFace cache
// Relatively simple file operations
```

3. **DELETE /api/delete**
```rust
// Remove model from cache
// File system operations
```

**Effort:** 1 day total

---

### Phase 5: macOS Support with llama.cpp (Priority 5)

**Goal:** Great macOS developer experience

**Implementation:** See ARCHITECTURE_DECISION.md

**Effort:** 3-4 days

---

### Phase 6: Advanced Features (Optional)

1. **POST /api/create** - Modelfile support
2. **POST /api/push** - Model sharing
3. Multi-GPU tensor parallelism
4. Speculative decoding

**Effort:** 2-3 days each

---

## Detailed vLLM Configuration Optimization

### Current Configuration (Minimal)
```bash
--model MODEL
--port 8100
--max-num-seqs 256
--gpu-memory-utilization 0.9
```

### Optimized Configuration (Recommended)
```bash
--model MODEL
--port 8100

# Concurrency & Batching
--max-num-seqs 256                    # Concurrent sequences (tune per model)
--max-num-batched-tokens 8192         # Higher = better throughput (default 2048)
--enable-chunked-prefill              # Better concurrent handling (V1 default)

# Memory & Caching
--gpu-memory-utilization 0.9          # Keep high
--enable-prefix-caching               # Reuse KV cache for repeated prompts
--max-model-len 4096                  # Optimize for typical context

# Performance
--swap-space 4                        # CPU offload buffer (GB)
--dtype auto                          # Automatic precision selection

# Optional: Multi-GPU
--tensor-parallel-size 2              # If multiple GPUs

# Optional: Monitoring
--disable-log-requests                # Reduce logging overhead in prod
```

### Parameter Tuning Guide

**max-num-batched-tokens:**
- 2048: Default, conservative
- 8192: **Recommended** for better throughput
- 16384+: Best throughput if memory allows
- Higher values = better concurrent performance

**max-num-seqs (by model size):**
- Small (<3B): 512 sequences
- Medium (3-13B): 256 sequences
- Large (13-70B): 128 sequences
- XLarge (>70B): 64 sequences

**enable-prefix-caching:**
- ✅ Enable for chat workloads (repeated system prompts)
- ✅ Enable for RAG applications (repeated context)
- ⚠️ Skip for fully unique prompts

---

## Implementation Priority

### Week 1: Performance (Critical) 🔥

**Day 1-2:**
- Add vLLM optimization flags
- Test concurrent performance
- Benchmark vs Ollama

**Day 3:**
- Add `/api/embeddings`
- Fix `/api/ps` and `/api/show`

**Day 4:**
- Add `/api/version`
- Documentation updates

**Day 5:**
- Comprehensive benchmarking
- Validate 4.4x faster sequential + concurrent

**Goal:** Beat Ollama on ALL workloads

---

### Week 2: Platform Support

**Day 1-4:**
- Add llama.cpp for macOS (see ARCHITECTURE_DECISION.md)
- Platform detection
- Model format handling

**Day 5:**
- Cross-platform testing
- Documentation

**Goal:** Great macOS developer experience

---

### Week 3: Polish

**Day 1-2:**
- Add `/api/copy` and `/api/delete`
- Model management improvements

**Day 3-4:**
- Performance testing at scale
- Memory profiling

**Day 5:**
- Documentation polish
- Release prep

---

## Success Criteria

### Performance Targets

| Workload | Current | Target | Ollama | Result |
|----------|---------|--------|--------|--------|
| Sequential | 232ms | 200ms | 1,010ms | **5x faster** |
| Concurrent (5) | 7.57s | 3.0s | 6.50s | **2x faster** |
| Concurrent (50) | TBD | TBD | TBD | **Faster** |
| Streaming | 0.617s | 0.5s | 1.015s | **2x faster** |

### Functionality Targets

**Must have:**
- ✅ /api/generate (done)
- ✅ /api/chat (done)
- ✅ /api/pull (done)
- ✅ /api/tags (done)
- 🎯 /api/embeddings (add)
- 🎯 /api/ps (fix)
- 🎯 /api/show (improve)
- 🎯 /api/version (add)

**Nice to have:**
- /api/copy
- /api/delete
- /api/create (Modelfile)
- /api/push

---

## Benchmarking Plan

### Test Matrix

**Models:**
- Small: Qwen/Qwen2.5-1.5B-Instruct
- Medium: meta-llama/Llama-3.2-3B-Instruct
- Large: meta-llama/Llama-3.1-8B-Instruct

**Workloads:**
- Sequential (1 request at a time)
- Concurrent (5, 10, 50, 100 parallel)
- Streaming vs non-streaming
- Short prompts (50 tokens) vs long (500 tokens)

**Metrics:**
- Throughput (tokens/sec)
- Latency (p50, p95, p99)
- Time to first token
- Memory usage

---

## Open Questions

1. **Embeddings:** Does vLLM support embeddings out of box, or need separate model?
2. **Model metadata:** How to get detailed model info from vLLM?
3. **Concurrent scaling:** What's the limit on RTX 4090 for concurrent requests?
4. **Modelfile:** Do we need Modelfile support, or is it Ollama-specific?

---

## Next Actions

**Immediate (this week):**
1. Update `serve.rs` with optimized vLLM flags
2. Run concurrent benchmarks with new config
3. Add `/api/embeddings` endpoint
4. Fix `/api/ps` implementation

**Want to proceed with Phase 1 optimization?**

---

*Created: 2025-10-20*
*Status: Ready for implementation*
