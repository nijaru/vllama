# Batching Investigation

## Problem

Despite implementing AsyncLLMEngine with batching configuration, concurrent performance remains poor:

**Results at 10 concurrent requests:**
- vLLama (with AsyncLLMEngine + batching): **21.7s** (2.17s per request)
- Ollama: **6.4s** (0.64s per request)
- **Ollama 3.38x faster**

## Configuration Attempted

```python
engine_args = AsyncEngineArgs(
    model=request.model_path,
    max_model_len=request.max_length,
    tensor_parallel_size=1,
    gpu_memory_utilization=0.9,
    max_num_seqs=256,  # Enable batching up to 256 concurrent sequences
    max_num_batched_tokens=8192,  # Batch up to 8K tokens
)
```

**Result:** Batching config is recognized (logs show "Chunked prefill is enabled with max_num_batched_tokens=8192"), but performance improved only 8% (23.7s → 21.7s).

## Root Cause Analysis

### Current Implementation Issues

1. **Request Handler Pattern:**
```python
async def generate(request: GenerateRequest):
    request_id = random_uuid()
    results_generator = llm.generate(request.prompt, sampling_params, request_id)

    final_output = None
    async for output in results_generator:
        final_output = output  # Wait for entire generation
```

2. **Problem:** Each FastAPI handler waits for its own generator to complete
   - Multiple handlers run concurrently (FastAPI async)
   - But each calls `llm.generate()` and waits for completion
   - Engine may be queueing requests rather than batching them together

3. **Continuous Batching Not Utilized:**
   - vLLM's strength is continuous batching across multiple active requests
   - Our pattern doesn't allow requests to be truly batched
   - Each request gets its own generator that runs to completion

### Scaling Behavior

| Concurrent Requests | vLLama (Async) | Ollama | vLLama/Ollama |
|---------------------|----------------|---------|---------------|
| 5                   | 6.72s (1.34s/req) | 6.50s (1.30s/req) | 0.97x (nearly tied) |
| 10                  | 21.7s (2.17s/req) | 6.43s (0.64s/req) | 0.30x (3.38x slower) |

**Key observation:** As concurrency increases:
- vLLama per-request time INCREASES (1.34s → 2.17s)
- Ollama per-request time DECREASES (1.30s → 0.64s)

This is the opposite of what we'd expect with proper batching!

## Hypothesis

The issue is not AsyncLLMEngine itself, but how we're integrating it:
1. FastAPI handles concurrency at the HTTP layer
2. Each handler makes async calls to AsyncLLMEngine
3. But AsyncLLMEngine is designed for server-side request batching, not client-side async calls
4. We're effectively using it as a single-request-at-a-time API

## Recommended Solution

**Use vLLM's official OpenAI-compatible server** instead of custom wrapper:

### Benefits

1. **Production-ready batching:**
   - Designed specifically for continuous batching
   - Request queue managed by vLLM server
   - Proven scaling behavior

2. **Less code to maintain:**
   - Official implementation
   - Regular updates from vLLM team
   - Better documentation

3. **Better observability:**
   - Metrics endpoint
   - Proper logging
   - Batch statistics

### Implementation

Replace `python/llm_service/server.py` with:

```bash
# Start vLLM OpenAI server
python -m vllm.entrypoints.openai.api_server \
  --model Qwen/Qwen2.5-1.5B-Instruct \
  --port 8100 \
  --max-model-len 4096 \
  --gpu-memory-utilization 0.9 \
  --max-num-seqs 256
```

Then vLLama Rust code calls this server's `/v1/completions` endpoint.

### Tradeoffs

**Pros:**
- Proper continuous batching out of the box
- Production-tested implementation
- Active maintenance

**Cons:**
- OpenAI API format (need to translate Ollama → OpenAI)
- Less control over engine initialization
- Additional dependency on vLLM's server implementation

## Alternative: Fix Current Implementation

If we want to keep custom server, we need to:

1. **Implement request queue** in our FastAPI app
2. **Batch requests** before sending to AsyncLLMEngine
3. **Manage generation state** across multiple requests
4. **Handle partial outputs** and distribute to correct clients

This is essentially re-implementing what vLLM's OpenAI server already does.

## Recommendation

**Phase 4 Priority:** Migrate to vLLM OpenAI-compatible server for production use.

- Current AsyncLLMEngine implementation works for single-user/sequential workloads
- For multi-user/concurrent workloads, use official vLLM server
- Document both approaches in deployment guide

## Test Plan After Migration

1. Sequential (baseline): Should maintain 4.4x advantage
2. 10 concurrent: Target 2-3x faster than Ollama (vs current 0.3x)
3. 50+ concurrent: vLLM batching should show larger advantage

---
*Investigation Date: 2025-10-20*
*vLLM Version: 0.11.0*
