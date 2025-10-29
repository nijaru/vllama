# vllama Benchmarking Guide

## Platform-Specific Performance

**Expected performance varies significantly by platform:**

| Platform | vllama Performance | Notes |
|----------|-------------------|-------|
| **Linux + NVIDIA GPU** | 10x+ faster than Ollama | Production performance, GPU acceleration via vLLM |
| **macOS (Apple Silicon)** | Similar to Ollama | Both run CPU-only, vLLM advantage minimal |
| **macOS (Intel)** | Similar to Ollama | Both run CPU-only, vLLM advantage minimal |
| **Linux (CPU-only)** | Slightly faster | vLLM optimizations help but limited without GPU |

**Key Insight:** vllama's speed advantage comes from vLLM's GPU acceleration. On CPU-only platforms (macOS), expect similar performance to Ollama.

## Running Benchmarks

vllama includes a benchmark tool to compare performance against Ollama.

### Setup

**Prerequisites:**
- vllama running on port 11434 (default)
- Ollama running on port 11435 (alternate port to avoid conflict)
- Same model loaded in both systems

**Start Ollama on alternate port:**
```bash
OLLAMA_HOST=127.0.0.1:11435 ollama serve
```

**Pull model in Ollama:**
```bash
# In another terminal
OLLAMA_HOST=127.0.0.1:11435 ollama pull llama3.1:8b
```

**Start vllama:**
```bash
# One command - auto-starts everything
cargo run --release --bin vllama -- serve \
  --model meta-llama/Llama-3.1-8B-Instruct \
  --port 11434
```

**Run benchmark:**
```bash
cargo run --release --bin vllama -- bench \
  "meta-llama/Llama-3.1-8B-Instruct" \
  "Explain quantum computing in simple terms" \
  -i 10
```

### Output

The benchmark reports:
- **Median latency**: 50th percentile (typical response time)
- **Average latency**: Mean across all requests
- **P99 latency**: 99th percentile (worst-case in 100 requests)
- **Tokens/sec**: Actual throughput from measured token counts
- **Total time**: Complete test duration

### Caveats

⚠️ **What This Tests:**
- vllama: Ollama API → vLLM OpenAI server (port 8100 by default)
- Ollama: Ollama API on port 11435

⚠️ **Limitations:**
- Both systems limited to **50 tokens per response** (max_tokens=50)
- Single-threaded sequential requests (no concurrency testing)
- No warmup runs (first request may be slower)
- Both systems must have same model loaded for fair comparison
- Performance advantage requires GPU - CPU-only platforms show minimal difference

⚠️ **What This Doesn't Test:**
- Concurrent request handling
- Streaming performance
- Memory usage under load
- Multi-GPU configurations
- Different quantization levels
- Long-context performance

## Honest Comparisons Only

Following best practices from the benchmark guidelines:

✅ **Valid Comparisons:**
- Same model on both systems
- Same prompt and iteration count
- Documented hardware and configuration
- Median + P99 reported (not just average)
- Caveats clearly stated

❌ **Invalid Comparisons:**
- Different models or quantization levels
- One system on GPU, other on CPU
- Cherry-picked best-case prompts
- Claiming "Nx faster" without context
- Hiding architectural differences

## Benchmark Template

When publishing benchmark results, use this template:

```markdown
## Benchmark: vllama vs Ollama

**Systems Compared:**
- vllama: [version, vLLM backend version, configuration]
- Ollama: [version, backend, configuration]

**Hardware:**
- Platform: [Linux + NVIDIA GPU / macOS (Apple Silicon) / etc.]
- GPU: [model, VRAM] or "CPU-only"
- CPU: [model, cores]
- RAM: [total GB]
- OS: [Linux/macOS version]

**Model:**
- Name: [e.g., meta-llama/Llama-3.1-8B-Instruct]
- Quantization: [if applicable]
- Context length: [e.g., 2048 tokens]

**Workload:**
- Prompt: "[exact prompt used]"
- Iterations: [number]
- Request pattern: Sequential, single-threaded
- Streaming: Disabled

**Results:**
| Metric | vllama | Ollama | Difference |
|--------|--------|--------|------------|
| Median latency | X ms | Y ms | Z.Zx |
| Avg latency | X ms | Y ms | Z.Zx |
| P99 latency | X ms | Y ms | Z.Zx |
| Tokens/sec | X (est.) | Y (est.) | Z.Zx |

**Caveats:**
- Token counts estimated, not measured
- vllama uses vLLM OpenAI server backend
- Ollama uses its own inference backend
- No concurrent requests tested
- Performance differences depend heavily on platform (GPU vs CPU)
- [Any other limitations]

**Honest Assessment:**
[What this means in practice, when speedup applies, what wasn't tested]
```

## Contributing Benchmarks

If you run benchmarks on your hardware:

1. Use the template above
2. Report median and P99 (not just average)
3. Document all configuration details
4. State caveats clearly
5. Run at least 10 iterations
6. Include hardware specifications
7. Note any architectural differences

**Before claiming "Nx faster":**
- Verify same features enabled on both systems
- Test on realistic workloads, not best-case scenarios
- Document what IS and ISN'T tested
- Explain WHY there's a difference

See [CLAUDE.md benchmarking guidelines](https://github.com/nijaru/agent-contexts/blob/main/standards/AI_CODE_PATTERNS.md) for detailed best practices.
