# Implementation Plan - Performance Optimization

**Version:** 0.0.x Development
**Date:** 2025-10-20
**Goal:** Beat Ollama performance while matching core functionality

---

## Research Summary

### Common Ollama Model Sizes (2025)

**Most popular models:**
- Llama 3.1/3.2: 8B, 70B (8B most common)
- Mistral 7B
- Qwen2.5: 7B-14B
- Phi-3: 3B, 14B
- Gemma 2: 2B, 9B
- DeepSeek-Coder: 33B
- CodeLlama: 34B

**Distribution:**
- **Small (2-3B):** ~20% (Phi-3 Mini, Gemma 2B)
- **Medium (7-14B):** ~60% ⭐ **MOST COMMON**
- **Large (30-70B):** ~15% (power users)
- **XLarge (405B):** ~5% (specialized)

**Optimization target:** 7-14B models (Llama 8B, Mistral 7B, Qwen 7B)

---

### Typical Concurrency

**Most Ollama users:**
- Personal use: 1-5 concurrent (majority)
- Small teams: 5-20 concurrent (common)
- Power users: 20-50 concurrent (rare)

**Optimization target:** 1-50 concurrent requests

---

### Embeddings Assessment

**What are embeddings for?**
- RAG (Retrieval-Augmented Generation)
- Company knowledge bases
- Semantic search
- Document Q&A

**Are they necessary?**
- ❌ Not for basic chat/completion (most users)
- ✅ Important for RAG applications (specialized use case)
- ✅ Can be added later when RAG features needed

**Decision:** ⏸️ Skip for 0.0.x, add in future when implementing RAG features

---

## Implementation Plan

### Phase 1: vLLM Performance Optimization

**Goal:** Beat Ollama on all workloads

**Current performance:**
- Sequential: 232ms (4.4x faster than Ollama) ✅
- Concurrent (5): 7.57s (1.16x SLOWER than Ollama) ❌
- Streaming: 0.617s (1.6x faster than Ollama) ✅

**Target performance:**
- Sequential: <200ms (5x faster than Ollama)
- Concurrent (5): <3.0s (2x faster than Ollama)
- Concurrent (50): <20s (2x faster than Ollama)
- Streaming: <0.5s (2x faster than Ollama)

**Changes:**

**1. Update vLLM configuration** (`serve.rs`)
```rust
// Current (minimal)
--max-num-seqs 256
--gpu-memory-utilization 0.9

// New (optimized for 7-14B models)
--max-num-batched-tokens 16384       // 32x increase from default (512)
--max-num-seqs 256                   // Good for medium models
--max-model-len 4096                 // Typical context length
--enable-chunked-prefill             // Better concurrent batching
--enable-prefix-caching              // Reuse KV cache for repeated prompts
--gpu-memory-utilization 0.9         // Keep existing
```

**Why these values:**
- `max-num-batched-tokens 16384`: Research shows 8192-65536 optimal for 24GB GPU, 16384 is conservative
- `max-num-seqs 256`: Good for 7-14B models on RTX 4090
- `enable-chunked-prefill`: Default in V1, critical for concurrent performance
- `enable-prefix-caching`: Helps with repeated system prompts (chat apps)

**Effort:** 30 minutes code + 2 hours testing

---

### Phase 2: Fix Missing Endpoints

**Goal:** Core Ollama compatibility

**Changes:**

**1. Fix /api/ps** (currently returns empty array)
```rust
// Query vLLM /v1/models endpoint
// Return actual loaded models with metadata
{
  "models": [
    {
      "name": "meta-llama/Llama-3.2-8B-Instruct",
      "model": "meta-llama/Llama-3.2-8B-Instruct",
      "size": 8000000000,
      "digest": "...",
      "processor": "cuda:0"
    }
  ]
}
```

**2. Improve /api/show** (currently limited)
```rust
// Query vLLM for model info
// Return useful metadata
{
  "model": "meta-llama/Llama-3.2-8B-Instruct",
  "details": {
    "family": "llama",
    "parameter_size": "8B",
    "quantization_level": "auto"
  },
  "modelfile": "..."
}
```

**3. Add /api/version**
```rust
{
  "version": "0.0.3",
  "backend": "vLLM",
  "backend_version": "0.11.0"
}
```

**Skip for 0.0.x:**
- `/api/embeddings` - Needs separate embedding model, RAG use case
- `/api/copy`, `/api/delete` - Model management, can add later
- `/api/create`, `/api/push` - Advanced features

**Effort:** 3-4 hours total

---

### Phase 3: Comprehensive Benchmarking

**Test matrix:**

**Models:**
- Qwen/Qwen2.5-1.5B-Instruct (baseline, fast)
- meta-llama/Llama-3.2-3B-Instruct (small)
- meta-llama/Llama-3.1-8B-Instruct (target size)

**Concurrency levels:**
- 1 (sequential)
- 5 (low)
- 10 (medium)
- 50 (high)

**Workloads:**
- Non-streaming generation
- Streaming generation
- Chat completions
- Mixed prompts (short 50 tokens, medium 200 tokens, long 500 tokens)

**Metrics:**
- Throughput (tokens/sec)
- Latency (p50, p95, p99)
- Time to first token
- Memory usage (nvidia-smi)

**Effort:** 4-6 hours

---

## Success Criteria

**Must have:**
- ✅ 2x+ faster than Ollama on concurrent requests (currently slower!)
- ✅ 4x+ faster than Ollama on sequential (already have, maintain)
- ✅ /api/ps returns actual data
- ✅ /api/show returns useful metadata
- ✅ /api/version endpoint

**Nice to have:**
- 5x+ faster on sequential
- 100+ concurrent requests tested
- Memory profiling

---

## Timeline

**Day 1 (Today): vLLM Optimization**
- [ ] Update serve.rs with optimized flags
- [ ] Test with 1, 5, 10, 50 concurrent requests
- [ ] Benchmark vs Ollama with Qwen 1.5B
- [ ] Document results

**Day 2: Endpoint Fixes**
- [ ] Fix /api/ps implementation
- [ ] Improve /api/show
- [ ] Add /api/version
- [ ] Test all endpoints

**Day 3: Comprehensive Benchmarking**
- [ ] Test with 3B and 8B models
- [ ] Run full test matrix
- [ ] Document performance improvements
- [ ] Update README and PROJECT_STATUS

**Total:** ~3 days

---

## Configuration Reference

### vLLM Optimization Parameters

**max-num-batched-tokens:**
- Default: 512
- Recommended: 8192-65536 (higher = better throughput)
- Our choice: 16384 (conservative for 7-14B models)

**max-num-seqs:**
- Small models (<3B): 512
- Medium (7-14B): 256 ⭐ **Our target**
- Large (30-70B): 128
- XLarge (405B): 64

**enable-chunked-prefill:**
- Breaks large prefill into chunks
- Processes concurrent with decode requests
- Critical for concurrent performance

**enable-prefix-caching:**
- Reuses KV cache for repeated prompts
- Huge win for chat apps (repeated system prompts)
- Important for RAG (repeated context)

---

## Notes

**Version strategy:**
- This is 0.0.x development work
- No rush to ship 0.1.0
- Tag releases when features are complete and stable
- Focus on incremental improvements

**What we're NOT doing (for now):**
- Embeddings (separate use case, needs RAG features)
- Multi-GPU (single RTX 4090 for now)
- Model management (copy/delete/create/push)
- macOS support (Phase 2, after performance is solid)

**What we ARE doing:**
- Fix concurrent performance (critical!)
- Match core Ollama endpoints
- Optimize for common workloads (7-14B models, 1-50 concurrent)
- Comprehensive benchmarking

---

## Next Steps

1. Update `crates/vllama-cli/src/commands/serve.rs` with vLLM flags
2. Test concurrent performance
3. If performance is good, move to endpoint fixes
4. Comprehensive benchmarking
5. Document results

**Ready to start implementation!**
