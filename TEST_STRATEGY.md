# vLLama Test Strategy

## Model Selection by Test Type

Use the right model for each test to minimize download time and maximize test coverage.

### Quick Tests (< 1GB downloads)

**Pull/Download Testing:**
- `Qwen/Qwen2.5-0.5B-Instruct` (494MB) - Fastest download
- `Qwen/Qwen2.5-1.5B-Instruct` (986MB) - Small, fast, non-gated

**API Smoke Tests:**
- `Qwen/Qwen2.5-1.5B-Instruct` - Already cached from benchmarks
- Fast inference, good for testing endpoints work correctly

**Error Handling Tests:**
- Non-existent model: `fake/model-does-not-exist`
- Gated model without auth: `meta-llama/Llama-3.1-8B-Instruct`
- Invalid format: `not-a-real-model-format`

### Performance Benchmarks

**Baseline (Small Model):**
- `Qwen/Qwen2.5-1.5B-Instruct` (986MB)
- Fast, consistent, good for iteration
- Use for testing optimizations/regressions

**Medium Models (Realistic Workloads):**
- `Qwen/Qwen2.5-7B-Instruct` (~4GB) - Publicly available
- `mistralai/Mistral-7B-Instruct-v0.3` (~4GB) - Publicly available
- Most common production size

**Large Models (Stress Testing):**
- `Qwen/Qwen2.5-14B-Instruct` (~8GB) - If VRAM permits
- Test VRAM limits, memory management

### Streaming Tests

**Fast Response Models:**
- `Qwen/Qwen2.5-1.5B-Instruct` - Quick tokens, test stream chunking

**Long Context:**
- `Qwen/Qwen2.5-7B-Instruct` with 8K context
- Test streaming with large inputs/outputs

### Concurrency Tests

**Use Smallest Model:**
- `Qwen/Qwen2.5-0.5B-Instruct` or `Qwen/Qwen2.5-1.5B-Instruct`
- Minimize inference time to focus on concurrency handling
- Can run more parallel requests with less VRAM

### Chat Template Tests

**Different Chat Formats:**
- `Qwen/Qwen2.5-1.5B-Instruct` - Qwen chat format
- `meta-llama/Llama-3.1-8B-Instruct` - Llama chat format (requires HF auth)
- `mistralai/Mistral-7B-Instruct-v0.3` - Mistral format

### VRAM/Memory Tests

**Progressive Loading:**
1. Start: `Qwen/Qwen2.5-0.5B-Instruct` (494MB)
2. Add: `Qwen/Qwen2.5-1.5B-Instruct` (986MB)
3. Add: `Qwen/Qwen2.5-7B-Instruct` (~4GB)
4. Monitor VRAM usage at each step

**Unload Testing:**
- Load large model, unload, verify VRAM freed
- Use `Qwen/Qwen2.5-7B-Instruct` for visible VRAM impact

## Test Workflow Examples

### Fast Development Iteration
```bash
# Use cached small model
vllama bench "Qwen/Qwen2.5-1.5B-Instruct" "test prompt" -i 3
```

### Pull Command Testing
```bash
# Smallest model for speed
curl -X POST http://localhost:11434/api/pull \
  -d '{"name":"Qwen/Qwen2.5-0.5B-Instruct"}'
```

### Real-World Performance Testing
```bash
# Medium model, longer test
vllama bench "Qwen/Qwen2.5-7B-Instruct" "complex prompt here" -i 10
```

### Multi-Model Comparison
```bash
# Compare across sizes
for model in \
  "Qwen/Qwen2.5-0.5B-Instruct" \
  "Qwen/Qwen2.5-1.5B-Instruct" \
  "Qwen/Qwen2.5-7B-Instruct"; do
    vllama bench "$model" "test prompt" -i 5
done
```

## Model Cache Strategy

**Keep Cached:**
- `Qwen/Qwen2.5-1.5B-Instruct` - Primary test model
- One 7B model for realistic testing

**Download On-Demand:**
- Larger models (14B+)
- Specialized formats
- Gated models (requires auth)

**Clean Up:**
- Remove models > 7B after testing
- Keep cache under 20GB total

## Ollama Model Mapping

When testing against Ollama, use equivalent models:

| vLLM (HuggingFace) | Ollama | Size |
|-------------------|--------|------|
| `Qwen/Qwen2.5-0.5B-Instruct` | `qwen2.5:0.5b` | 494MB |
| `Qwen/Qwen2.5-1.5B-Instruct` | `qwen2.5:1.5b` | 986MB |
| `Qwen/Qwen2.5-7B-Instruct` | `qwen2.5:7b` | ~4GB |
| `meta-llama/Llama-3.1-8B-Instruct` | `llama3.1:8b` | ~5GB |

## Test Priority by Model

**High Priority (Always Test):**
1. `Qwen/Qwen2.5-1.5B-Instruct` - Fast, reliable, cached
2. `Qwen/Qwen2.5-7B-Instruct` - Realistic workload size

**Medium Priority (Regular Testing):**
3. `Qwen/Qwen2.5-0.5B-Instruct` - Stress concurrency
4. `mistralai/Mistral-7B-Instruct-v0.3` - Different architecture

**Low Priority (Occasional Testing):**
5. `Qwen/Qwen2.5-14B-Instruct` - VRAM limits
6. Larger models - Special cases only

## Anti-Patterns to Avoid

❌ **Don't:**
- Use 70B models for quick tests
- Download gated models without HF auth configured
- Test on models that aren't cached (wastes time)
- Use different models in before/after comparisons

✅ **Do:**
- Start small, scale up as needed
- Keep fast model cached for iteration
- Use consistent models for benchmarking
- Document which model each test uses
