# vLLama Benchmarking Guide

## Running Benchmarks

vLLama includes a benchmark tool to compare performance against Ollama.

### Setup

**Prerequisites:**
- vLLama running on port 11434 (default)
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

**Start vLLama services:**
```bash
# Terminal 1: vLLM service
cd python && uv run uvicorn llm_service.server:app --host 127.0.0.1 --port 8100

# Terminal 2: vLLama server
cargo run --release --bin vllama -- serve --host 127.0.0.1 --port 11434
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
- **Tokens/sec**: Estimated throughput (assumes 50 tokens per response)
- **Total time**: Complete test duration

### Caveats

⚠️ **What This Tests:**
- vLLama: Direct Python engine access (minimal HTTP overhead)
- Ollama: Full HTTP API stack on port 11435

⚠️ **Limitations:**
- Token counts are **estimated at 50 per response**, not exact
- Comparing different levels: direct engine vs HTTP API
- Single-threaded sequential requests (no concurrency testing)
- No warmup runs (first request may be slower)
- Both systems must have same model loaded for fair comparison

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
## Benchmark: vLLama vs Ollama

**Systems Compared:**
- vLLama: [version, vLLM backend version, configuration]
- Ollama: [version, backend, configuration]

**Hardware:**
- GPU: [model, VRAM]
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
| Metric | vLLama | Ollama | Difference |
|--------|--------|--------|------------|
| Median latency | X ms | Y ms | Z.Zx |
| Avg latency | X ms | Y ms | Z.Zx |
| P99 latency | X ms | Y ms | Z.Zx |
| Tokens/sec | X (est.) | Y (est.) | Z.Zx |

**Caveats:**
- Token counts estimated, not measured
- vLLama uses direct engine access
- Ollama uses HTTP API
- No concurrent requests tested
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
