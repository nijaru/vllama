# Planning Questions & Answers

**Date:** 2025-10-20
**Purpose:** Critical questions to answer before optimization implementation

---

## 1. vLLM Version & Engine Questions

### Q1.1: Are we using vLLM V0 or V1?

**Current state:**
- `python/pyproject.toml`: `vllm>=0.5.0`
- No `VLLM_USE_V1=1` environment variable set
- **Answer: We're using V0 (stable)**

**Implications:**
- ✅ Embeddings ARE supported in V0
- ✅ All optimization features available
- ⚠️ V1 is faster but alpha (no embeddings yet)

**Decision needed:** Should we upgrade to V1 for performance, or stay on V0 for embeddings?

**My recommendation:**
- Stay on V0 for now (stable, has embeddings)
- Track V1 maturity for future upgrade
- V1 doesn't support embeddings, log probs, speculative decoding yet

---

### Q1.2: What vLLM optimization flags should we use on RTX 4090?

**Hardware:**
- RTX 4090: 24GB VRAM
- Model sizes: 1.5B - 8B parameters

**Research findings:**

For 24GB GPU with typical models:
```bash
# Example from research (24GB GPU, 6GB model, 18GB free for KV cache)
--max-num-batched-tokens 65536
--max-num-seqs 64
--max-model-len 1024
```

General recommendations:
- `max-num-batched-tokens`: 8192-65536 (higher = better throughput)
- `max-num-seqs`: 64-512 (depends on model size)
- Default is 512 for batched tokens (too conservative)

**Questions for user:**
1. What model sizes do you primarily use? (1.5B, 3B, 7B, 13B?)
2. What's typical context length needed? (512, 2048, 4096, 8192?)
3. Priority: throughput or latency?

**Proposed conservative config:**
```bash
# Good balance for 1.5-8B models on RTX 4090
--max-num-batched-tokens 16384    # 4x increase from default
--max-num-seqs 256                # Current value
--max-model-len 4096              # Typical context
--enable-chunked-prefill
--enable-prefix-caching
```

---

### Q1.3: Does vLLM embeddings endpoint work with LLM models?

**Research findings:**
- ❌ No - embeddings require **specific embedding models**
- vLLM checks `_EMBEDDING_MODELS` in `vllm/model_executor/models/registry.py`
- Must pass `task="embed"` for embedding models
- LLM models (Llama, Mistral, etc.) **cannot** generate embeddings via `/v1/embeddings`

**Implications:**
- Can't just add `/api/embeddings` endpoint to existing LLM models
- Need separate embedding model service OR use LLM hidden states (hacky)

**Questions for user:**
1. Do users need embeddings? (Yes/No/Maybe)
2. If yes, acceptable to require separate embedding model?
3. Or skip embeddings for now?

**My recommendation:**
- ⏸️ **Skip embeddings for MVP** (requires separate model/service)
- Focus on LLM performance first
- Add embeddings later as separate feature (Phase 2)

---

## 2. Ollama API Compatibility Questions

### Q2.1: Which Ollama endpoints are actually used in practice?

**Full Ollama API:**
- ✅ POST /api/generate (we have)
- ✅ POST /api/chat (we have)
- ✅ POST /api/pull (we have)
- ✅ GET /api/tags (we have)
- ⚠️ GET /api/show (partial)
- ⚠️ GET /api/ps (stub)
- ❌ POST /api/embeddings
- ❌ GET /api/version
- ❌ POST /api/copy
- ❌ DELETE /api/delete
- ❌ POST /api/create (Modelfile)
- ❌ POST /api/push

**Questions for user:**
1. Which missing endpoints do YOU actually use?
2. Which endpoints do typical users need?
3. Can we ship without /api/embeddings?

**My analysis based on common usage:**

**Critical (must have):**
- ✅ /api/generate
- ✅ /api/chat
- ✅ /api/pull
- ✅ /api/tags

**Important (nice to have):**
- /api/show (model metadata)
- /api/ps (running models)
- /api/version (basic info)

**Optional (advanced users):**
- /api/embeddings (needs separate model)
- /api/copy, /api/delete (management)
- /api/create, /api/push (power users)

**My recommendation:**
- ✅ Fix /api/show and /api/ps (easy, useful)
- ✅ Add /api/version (trivial)
- ⏸️ Skip embeddings (complex, separate model needed)
- ⏸️ Skip copy/delete/create/push (can add later)

---

### Q2.2: What does /api/show actually need to return?

**Current implementation:** Limited metadata

**Ollama's /api/show returns:**
- Model name
- Modified date
- Size
- Digest
- Model details (family, parameters, quantization)
- Template
- Parameters (temperature, etc.)
- License
- Modelfile

**Questions for user:**
1. What information do you actually use from /api/show?
2. Is model name + size enough, or need full metadata?

**What we can get from vLLM:**
- ✅ Model name
- ✅ Model type/architecture
- ⚠️ Parameters count (maybe)
- ❌ Modelfile (Ollama-specific)
- ❌ License (not from vLLM)

**My recommendation:**
Return basic useful info:
```json
{
  "model": "meta-llama/Llama-3.2-1B-Instruct",
  "details": {
    "family": "llama",
    "parameter_size": "1B",
    "quantization": "auto"
  }
}
```

---

### Q2.3: What should /api/ps return?

**Ollama's /api/ps returns:**
- Running models
- Size in memory
- Processor (GPU/CPU)
- Until (expiry time)

**What we can return:**
- ✅ Model name
- ✅ vLLM server status
- ⚠️ Memory usage (maybe from vLLM metrics)
- ❌ Per-model tracking (vLLM handles this)

**My recommendation:**
```json
{
  "models": [
    {
      "name": "meta-llama/Llama-3.2-1B-Instruct",
      "size": 1024000000,
      "processor": "cuda:0"
    }
  ]
}
```

---

## 3. Performance Target Questions

### Q3.1: What are acceptable performance targets vs Ollama?

**Current performance:**
- Sequential: 4.4x faster ✅
- Concurrent (5): 1.16x slower ❌
- Streaming: 1.6x faster ✅

**Questions for user:**
1. Is 2x faster than Ollama across all workloads good enough?
2. Or need 5x+ for marketing purposes?
3. What workloads matter most? (sequential, concurrent, streaming)

**My recommendation:**
- **Target:** 2-3x faster than Ollama on ALL workloads
- **Stretch:** 5x+ on sequential (already have 4.4x)
- **Priority:** Fix concurrent (currently slower!)

**Proposed targets:**
| Workload | Current | Target | vs Ollama |
|----------|---------|--------|-----------|
| Sequential | 232ms | 200ms | 5x faster |
| Concurrent (5) | 7.57s | 3.0s | 2x faster |
| Concurrent (50) | ? | ? | 2x faster |
| Streaming | 0.617s | 0.5s | 2x faster |

---

### Q3.2: What concurrency levels should we optimize for?

**Current test:** 5 parallel requests

**Questions for user:**
1. Typical production concurrency? (5, 10, 50, 100?)
2. Peak traffic expectations?
3. Single user or multi-user service?

**My recommendation:**
Test and optimize for:
- Low: 1-5 concurrent (chat apps)
- Medium: 10-20 concurrent (small teams)
- High: 50-100 concurrent (production services)

---

## 4. Resource & Testing Questions

### Q4.1: What hardware is available for testing?

**Known:**
- Fedora PC: RTX 4090, i9-13900KF, 32GB DDR5
- M3 Max: 128GB (for macOS testing)

**Questions for user:**
1. Is RTX 4090 the primary target GPU?
2. Test on other GPUs? (A100, 4060, etc.)
3. CPU-only testing needed?

**My recommendation:**
- Primary target: RTX 4090 (what you have)
- Document settings for other GPUs
- macOS testing on M3 Max (Phase 2)

---

### Q4.2: What models should we benchmark with?

**Current:** Qwen/Qwen2.5-1.5B-Instruct

**Questions for user:**
1. What models do you actually use?
2. Optimize for small (<3B), medium (7-13B), or large (70B+)?

**My recommendation:**
Test matrix:
- Small: Qwen/Qwen2.5-1.5B-Instruct (current)
- Medium: meta-llama/Llama-3.2-3B-Instruct
- Large: meta-llama/Llama-3.1-8B-Instruct

Focus optimization on your most-used size.

---

## 5. Implementation Priority Questions

### Q5.1: What's the priority order?

**Option A: Performance First**
1. Fix concurrent performance (vLLM flags)
2. Benchmark improvements
3. Add missing endpoints
4. macOS support

**Option B: Functionality First**
1. Add missing endpoints (/api/show, /api/ps, etc.)
2. Fix concurrent performance
3. Benchmark improvements
4. macOS support

**Questions for user:**
1. Performance or functionality first?
2. Can we ship without embeddings?
3. When do you need macOS support?

**My recommendation:** Option A (Performance First)
- Fix the "slower than Ollama" problem NOW
- Add endpoints incrementally
- macOS support in Phase 2

---

### Q5.2: Embeddings: Ship without it or wait?

**Options:**

**A: Skip embeddings (ship faster)**
- Focus on LLM performance
- Add embeddings later as Phase 2
- Requires separate embedding model anyway

**B: Add embeddings (more complete)**
- Need to run separate embedding model
- More complex architecture
- Delays other work

**Questions for user:**
1. Do you use embeddings in your workflow?
2. Can users wait for embeddings in v0.2?
3. Acceptable to run separate embedding service?

**My recommendation:** Skip for v0.1
- Most users use Ollama for LLM chat/completion
- Embeddings are separate use case
- Can add properly in Phase 2

---

## 6. Testing & Validation Questions

### Q6.1: What defines "done" for optimization?

**Questions for user:**
1. Minimum speedup required? (2x, 5x?)
2. Need to beat Ollama on all workloads or just most?
3. Acceptable to be equal on some metrics?

**My recommendation:**
**"Done" = Beat Ollama on all primary workloads:**
- ✅ Sequential: 2x+ faster
- ✅ Concurrent: 1.5x+ faster (currently slower!)
- ✅ Streaming: 1.5x+ faster

---

### Q6.2: How much testing before shipping?

**Options:**

**A: Ship fast, iterate**
- Basic benchmarks on RTX 4090
- Ship once faster than Ollama
- Fix bugs in production

**B: Thorough testing**
- Multiple GPUs
- Load testing (100+ concurrent)
- Memory profiling
- Multi-day stress tests

**Questions for user:**
1. Is this for production use or experimentation?
2. Risk tolerance for bugs?
3. Timeline pressure?

**My recommendation:** Middle ground
- Thorough testing on RTX 4090 (your hardware)
- Benchmark 1, 5, 10, 50 concurrent requests
- 24-hour stability test
- Ship with "tested on RTX 4090" caveat

---

## Summary of Questions Needing User Input

### Critical (Block Implementation)

1. **Embeddings:** Ship without them? (Yes/No)
   - My rec: Yes, add in Phase 2

2. **Performance target:** What's "good enough"?
   - My rec: 2x faster than Ollama on all workloads

3. **Priority:** Performance first or functionality first?
   - My rec: Performance (fix concurrent issue)

4. **Models:** What sizes do you primarily use? (1.5B, 3B, 7B, 13B, 70B?)
   - Needed for: tuning max-num-seqs and batched tokens

### Important (Affects Scope)

5. **Missing endpoints:** Which do you actually need?
   - My rec: Fix /api/show, /api/ps, /api/version; skip copy/delete/create/push

6. **Concurrency:** Typical production load? (5, 10, 50, 100+ requests?)
   - Needed for: tuning configuration

7. **Testing depth:** Ship fast or test thoroughly?
   - My rec: Thorough on RTX 4090, then ship

### Nice to Know (Optimization)

8. **Context length:** Typical prompt/response sizes? (512, 2048, 4096 tokens?)
9. **vLLM V1:** Worth testing alpha for 24% speedup? Or stick with stable?
10. **macOS timeline:** Phase 2 OK, or need sooner?

---

## My Proposed Answers (For Your Review)

**If you agree with these, we can proceed immediately:**

1. ✅ **Skip embeddings** - Add in Phase 2 (requires separate model)
2. ✅ **Performance target:** 2-3x faster than Ollama on all workloads
3. ✅ **Priority:** Performance first (fix concurrent issue NOW)
4. ❓ **Models:** Need your input - what sizes?
5. ✅ **Endpoints:** Fix /api/show, /api/ps, add /api/version; skip rest
6. ❓ **Concurrency:** Need your input - typical load?
7. ✅ **Testing:** Thorough on RTX 4090, 1-50 concurrent, 24hr stability
8. ✅ **vLLM config:** Conservative start (16384 batched tokens, 256 seqs)
9. ✅ **V1:** Stick with stable V0 for now
10. ✅ **macOS:** Phase 2 (Week 2)

---

## Next Steps

**If you approve proposed answers:**
1. I'll update serve.rs with optimized vLLM flags
2. Run concurrent benchmarks
3. Fix /api/ps and /api/show
4. Add /api/version
5. Full benchmark suite vs Ollama

**If you have different answers:**
Let me know which questions need different decisions!

---

*Ready for your input on the critical questions (especially #4 and #6)*
