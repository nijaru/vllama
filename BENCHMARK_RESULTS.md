# vLLama Benchmark Results

## Benchmark: vLLama (vLLM) vs Ollama

**Date**: 2025-10-20
**Test**: Qwen/Qwen2.5-1.5B-Instruct text generation

### Systems Compared

**vLLama:**
- Version: 0.1.0
- Backend: vLLM 0.11.0
- Configuration: Direct engine access via HTTP (port 8100), GPU acceleration enabled
- Python: 3.12.11

**Ollama:**
- Version: Latest (running on port 11435)
- Backend: Default Ollama runtime
- Configuration: Full HTTP API stack

### Hardware

- **GPU**: NVIDIA GeForce RTX 4090 (24GB VRAM)
- **CPU**: Intel i9-13900KF (32 cores)
- **RAM**: 32GB DDR5
- **OS**: Fedora Linux 6.16.10

### Model

- **Name**: Qwen/Qwen2.5-1.5B-Instruct
- **Size**: 986MB (quantized)
- **Quantization**: Default (GGUF for Ollama, safetensors for vLLM)
- **Context length**: 4096 tokens

### Workload

- **Prompt**: "Explain quantum computing in simple terms"
- **Iterations**: 10
- **Request pattern**: Sequential, single-threaded
- **Streaming**: Disabled
- **Max tokens**: 50 per response

### Results - Sequential Requests

| Metric | vLLama (vLLM) | Ollama | Speedup |
|--------|---------------|---------|---------|
| **Median latency** | 232ms | 1,010ms | **4.4x faster** |
| **Avg latency** | 233ms | 1,020ms | **4.4x faster** |
| **P99 latency** | 245ms | 1,090ms | **4.4x faster** |
| **Total time** | 2.34s | 10.20s | **4.4x faster** |

### Results - Concurrent Requests (5 parallel)

**Before fix (Sync LLM):**
| Metric | vLLama (Sync) | Ollama | Result |
|--------|---------------|---------|---------|
| **Total time** | 7.57s | 6.50s | **Ollama 1.16x faster** |
| **Avg per request** | 1.51s | 1.30s | **Ollama faster** |

⚠️ **Issue identified**: Synchronous `LLM.generate()` was blocking async event loop, preventing concurrent execution.

**After fix (AsyncLLMEngine):**
| Metric | vLLama (Async) | Ollama | Result |
|--------|---------------|---------|---------|
| **Total time** | 6.72s | 6.50s | **Nearly tied (Ollama 3% faster)** |
| **Avg per request** | 1.34s | 1.30s | **Competitive** |

✅ **Fix applied**: Replaced synchronous LLM with AsyncLLMEngine
- **11% improvement** over sync implementation (7.57s → 6.72s)
- **Now competitive** with Ollama (only 3% difference)

### Results - Streaming Performance

| Metric | vLLama (vLLM) | Ollama | Speedup |
|--------|---------------|---------|---------|
| **Time to complete** | 0.617s | 1.015s | **1.6x faster** |

✓ vLLama maintains advantage for streaming workloads.

### Caveats

⚠️ **What This Tests:**
- vLLama: Direct Python engine access (minimal HTTP overhead)
- Ollama: Full HTTP API on port 11435
- Both systems limited to 50 tokens per response
- Small model (1.5B parameters) on powerful hardware

⚠️ **Limitations:**
- Models already loaded (no cold-start penalty measured)
- Small prompt and response size (50 tokens)
- Limited concurrency testing (only 5 parallel requests)
- Token counting issue in vLLM integration (shows 0.00 tokens/sec)

⚠️ **What This Tests:**
- ✅ Sequential request performance (10 iterations)
- ✅ Concurrent request handling (5 parallel requests)
- ✅ Streaming performance
- ✅ Small model (1.5B parameters)

⚠️ **What This Doesn't Test:**
- Cold-start model loading time
- Memory usage under sustained load
- High concurrency (10+, 50+, 100+ parallel requests)
- Multi-GPU configurations
- Long-context performance (>4K tokens)
- Different quantization levels
- Larger models (7B, 13B, 70B)

### Honest Assessment

**Performance Summary (After AsyncLLMEngine Fix):**
- Sequential requests: vLLama **4.4x faster** (232ms vs 1010ms)
- Concurrent requests: **Nearly tied** (6.72s vs 6.50s, Ollama 3% faster)
- Streaming: vLLama **1.6x faster** (0.617s vs 1.015s)

**Key Findings:**

✅ **vLLama excels at:**
- Sequential request throughput (4.4x faster)
- Streaming responses (1.6x faster)
- Single-user or low-concurrency workloads
- GPU-accelerated inference with vLLM optimizations

✅ **Concurrency issue fixed:**
- Implemented AsyncLLMEngine (replaced blocking sync LLM)
- Improved concurrent performance by 11% (7.57s → 6.72s)
- Now competitive with Ollama (only 3% slower vs 16% before fix)
- Remaining 3% gap likely due to vLLM batch processing optimization opportunities

✅ **When to choose vLLama:**
- Single-user applications
- Batch processing (sequential)
- Streaming-heavy workloads
- Maximum throughput for individual requests

📊 **Concurrent workloads:**
- With AsyncLLMEngine, vLLama is now competitive (3% slower)
- For very high concurrency (50+), vLLM's batch processing may provide advantage
- Both systems handle moderate concurrency (5-20 requests) well

**Architecture difference:**
- vLLama uses vLLM's PagedAttention and optimized CUDA kernels
- Ollama uses llama.cpp with different optimization strategies
- Both are mature, production-ready inference engines
- Current vLLama implementation may need request batching optimization

**Note:** Initial benchmark showed 18x speedup but was incorrect - Ollama wasn't respecting the 50-token limit and was generating 130 tokens. After fixing the benchmark code to ensure fair comparison (same token count), the actual speedup is 4.4x.

### Next Steps

**Immediate priorities based on findings:**
1. ✅ ~~Fix concurrent request handling~~ - **COMPLETED**: AsyncLLMEngine implemented
2. **Test higher concurrency** - 10, 50, 100 parallel requests to verify vLLM batch processing advantage
3. **Fine-tune batch processing** - Optimize AsyncLLMEngine configuration for maximum throughput

**Additional validation needed:**
4. Larger models (7B, 13B parameters) - Ollama 7B download issues prevented testing
5. Longer context windows (4K, 8K, 16K tokens)
6. Cold-start performance (model loading time)
7. Sustained load (memory leaks, performance degradation)
8. Different models/architectures for generalization

---

*Generated with vLLama benchmark tool (`vllama bench`)*
*See [BENCHMARKS.md](BENCHMARKS.md) for methodology and guidelines*
