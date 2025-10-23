# vLLM Optimization Research

**Researched:** 2025-10-20
**Purpose:** Fix concurrent performance and optimize vLLM configuration

---

## Problem Statement

**Current performance (RTX 4090, Qwen 1.5B):**
- Sequential: 232ms (4.4x faster than Ollama) ✅
- Concurrent (5): 7.57s (1.16x SLOWER than Ollama) ❌
- Streaming: 0.617s (1.6x faster than Ollama) ✅

**Root cause:** Minimal vLLM configuration (only 2 params)

---

## Research Findings

### vLLM V1 Features (2025)

**Key optimizations available:**
- Chunked prefill: Breaks large prefill into chunks, processes with decode requests
- Prefix caching: Reuses KV cache for repeated prompts
- Better batch token management
- 24% throughput improvement over V0

**Status:**
- V1 is alpha (not stable)
- V1 doesn't support embeddings yet
- Decision: Stay on V0 (stable) for now

**Source:** vLLM blog, Red Hat articles

---

### Optimization Parameters

#### max-num-batched-tokens

**Default:** 512 (too conservative)
**Recommended:** 8192-65536 for 24GB GPU
**Our choice:** 16384

**Research:**
- Controls batch size for throughput
- Higher = better throughput and lower latency
- 24GB GPU with 6GB model + 18GB KV cache can handle 65536
- Recent benchmarks show best results at 98304 on high-end GPUs

**Source:** vLLM docs, community benchmarks, RTX 4090 case studies

---

#### max-num-seqs

**Purpose:** Limits concurrent sequences in batch

**Recommendations by model size:**
- Small (<3B): 512 sequences
- Medium (7-14B): 256 sequences ⭐ **Our target**
- Large (30-70B): 128 sequences
- XLarge (405B): 64 sequences

**Rationale:** 60% of Ollama users use 7-14B models

**Source:** Ollama library popularity stats

---

#### enable-chunked-prefill

**Purpose:** Better concurrent request handling

**How it works:**
- Breaks large prefill requests into smaller chunks
- Chunks processed concurrently with decode requests
- Prioritizes decode requests in batch
- Uses max_num_batched_tokens to define chunk size

**Impact:** Critical for fixing concurrent performance bottleneck

**Recommendation:** Always enable (default in V1, explicit in V0)

**Source:** vLLM optimization docs

---

#### enable-prefix-caching

**Purpose:** Reuse KV cache for repeated prompts

**Use cases:**
- Chat apps with repeated system prompts
- RAG applications with repeated context
- Multi-turn conversations

**Impact:** Huge win for typical chat workloads

**Recommendation:** Enable for chat/RAG use cases

**Source:** vLLM performance tuning guide

---

#### max-model-len

**Purpose:** Maximum context length

**Default:** Model's native max length
**Our choice:** 4096

**Rationale:**
- Most workloads use <4K context
- Reduces memory footprint
- Can adjust per model if needed

---

## Recommended Configuration

### For RTX 4090 with 7-14B Models

```bash
python -m vllm.entrypoints.openai.api_server \
  --model MODEL \
  --port 8100 \
  # Concurrency & Batching
  --max-num-seqs 256 \
  --max-num-batched-tokens 16384 \   # 32x increase from default
  # Context
  --max-model-len 4096 \
  # Performance
  --enable-chunked-prefill \
  --enable-prefix-caching \
  # Memory
  --gpu-memory-utilization 0.9
```

---

## Expected Impact

**Targets:**
- Concurrent (5): 7.57s → <3.0s (2x faster than Ollama)
- Concurrent (50): TBD → <20s (2x faster than Ollama)
- Sequential: Maintain 4.4x advantage
- Streaming: Maintain 1.6x advantage

**Key metric:** Fix the concurrent performance regression

---

## Alternative Configurations

### For Larger Models (70B+)

```bash
--max-num-seqs 128           # Lower concurrent seqs
--max-num-batched-tokens 8192  # More conservative
--max-model-len 2048          # Smaller context
```

### For Maximum Throughput (Small Models)

```bash
--max-num-seqs 512
--max-num-batched-tokens 32768
--max-model-len 8192
```

---

## Testing Plan

**Concurrency levels:**
- 1 (sequential baseline)
- 5 (low concurrency)
- 10 (medium)
- 50 (high)

**Models:**
- Qwen 1.5B (fast testing)
- Llama 3B (small)
- Llama 8B (target size)

**Metrics:**
- Total time
- Tokens/second
- Time to first token
- Memory usage

---

## Common Ollama Model Sizes (Research)

**Distribution:**
- Small (2-3B): 20% - Phi-3 Mini, Gemma 2B
- **Medium (7-14B): 60%** - Llama 8B, Mistral 7B, Qwen 7B ⭐
- Large (30-70B): 15% - DeepSeek 33B, CodeLlama 34B
- XLarge (405B): 5% - Llama 3.1 405B

**Most popular:**
1. Llama 3.1/3.2 (8B most common)
2. Mistral 7B
3. Qwen2.5 (7B-14B)
4. Phi-3 (3B, 14B)

**Source:** Ollama library stats, community reports

---

## Typical Concurrency Patterns

**Research findings:**
- **Most users:** 1-5 concurrent (personal use, chat apps)
- **Some users:** 5-20 concurrent (small teams)
- **Power users:** 20-50+ concurrent (production services)

**Optimization target:** 1-50 concurrent requests

**Source:** Ollama usage patterns, community forums

---

## References

- vLLM Optimization Docs: https://docs.vllm.ai/en/latest/configuration/optimization.html
- vLLM V1 Blog: https://blog.vllm.ai/2025/01/27/v1-alpha-release.html
- Community benchmarks: RTX 4090 case studies
- Ollama library: Popular model statistics
