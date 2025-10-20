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

| Metric | vLLama (vLLM) | Ollama | Result |
|--------|---------------|---------|---------|
| **Total time** | 7.57s | 6.50s | **Ollama 1.16x faster** |
| **Avg per request** | 1.51s | 1.30s | **Ollama faster** |

⚠️ **Surprising finding**: Under concurrency, Ollama handled load better than vLLama. This suggests vLLama's current implementation may be serializing requests rather than batching them efficiently.

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

**Performance Summary:**
- Sequential requests: vLLama **4.4x faster** (232ms vs 1010ms)
- Concurrent requests: Ollama **1.16x faster** (6.5s vs 7.6s for 5 parallel)
- Streaming: vLLama **1.6x faster** (0.617s vs 1.015s)

**Key Findings:**

✅ **vLLama excels at:**
- Sequential request throughput (4.4x faster)
- Streaming responses (1.6x faster)
- Single-user or low-concurrency workloads
- GPU-accelerated inference with vLLM optimizations

⚠️ **vLLama needs improvement:**
- Concurrent request handling (Ollama 16% faster)
- Request batching appears to be serializing rather than parallelizing
- This suggests vLLM engine integration needs optimization for multi-request scenarios

✅ **When to choose vLLama:**
- Single-user applications
- Batch processing (sequential)
- Streaming-heavy workloads
- Maximum throughput for individual requests

⚠️ **When Ollama may be better:**
- Multi-user web services (high concurrency)
- Chat applications with many simultaneous users
- Production scenarios with bursty concurrent traffic

**Architecture difference:**
- vLLama uses vLLM's PagedAttention and optimized CUDA kernels
- Ollama uses llama.cpp with different optimization strategies
- Both are mature, production-ready inference engines
- Current vLLama implementation may need request batching optimization

**Note:** Initial benchmark showed 18x speedup but was incorrect - Ollama wasn't respecting the 50-token limit and was generating 130 tokens. After fixing the benchmark code to ensure fair comparison (same token count), the actual speedup is 4.4x.

### Next Steps

**Immediate priorities based on findings:**
1. **Fix concurrent request handling** - vLLM should batch concurrent requests, not serialize them
2. **Test higher concurrency** - 10, 50, 100 parallel requests to quantify scaling
3. **Profile request pipeline** - Identify serialization bottlenecks in vLLama server

**Additional validation needed:**
4. Larger models (7B, 13B parameters) - Ollama 7B download issues prevented testing
5. Longer context windows (4K, 8K, 16K tokens)
6. Cold-start performance (model loading time)
7. Sustained load (memory leaks, performance degradation)
8. Different models/architectures for generalization

---

*Generated with vLLama benchmark tool (`vllama bench`)*
*See [BENCHMARKS.md](BENCHMARKS.md) for methodology and guidelines*
