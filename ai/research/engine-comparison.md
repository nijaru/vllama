# Engine Comparison: vLLM vs MAX vs llama.cpp

**Date:** 2025-10-20
**Purpose:** Validate engine choices for vllama

---

## TL;DR: Recommendation

**✅ Final Architecture:**
- **Linux/NVIDIA:** vLLM (best production choice)
- **macOS/Apple Silicon:** llama.cpp (only viable option)
- **MAX:** Worth testing later, not a priority

---

## vLLM on macOS: The Verdict

### Current Status (2025)
- ❌ CPU-only support (experimental)
- ❌ No Metal/GPU support
- ❌ ~10 tok/s vs llama.cpp's 26-65 tok/s

### Future Plans (2025-2026 Roadmap)
**⚠️ NO Metal support planned**

Reviewed official roadmaps:
- Q1 2025 (Issue #11862): No Metal mention
- Q2 2025 (Issue #15735): AMD, TPU, Neuron - no Metal
- Q3 2025 (Issue #20336): No Metal mention
- 2024/2025 Vision Blog: No Metal in roadmap

**Community requests exist (Issues #212, #2081, #19073) but no official commitment.**

### Conclusion
**vLLM will NOT support macOS GPU in foreseeable future.** llama.cpp is the only viable option.

---

## vLLM vs MAX: Detailed Comparison

### Performance

| Metric | vLLM | MAX | Winner |
|--------|------|-----|--------|
| 500 prompts completion | 58.9s | 50.6s | MAX (16% faster) |
| Time to First Token (TTFT) | Less consistent | More consistent | MAX |
| Throughput (official) | Baseline | Within 2% | Tie |
| Concurrent requests | 512 max | 248 max | **vLLM** |

**Key difference:** MAX uses naive KV cache vs vLLM's PagedAttention

---

### Maturity & Ecosystem

#### vLLM ✅

**Adoption:**
- Powers Amazon Rufus, LinkedIn AI
- Most widely adopted inference framework
- Endorsed by NVIDIA and AMD
- Joined PyTorch ecosystem (Dec 2024)
- Backed by Red Hat

**Development:**
- UC Berkeley Sky Computing Lab origin
- Community-driven project
- V1 engine (2025): 24% throughput improvement
- Core of Red Hat AI Inference Server

**Proven production stability**

#### MAX ⚠️

**Adoption:**
- Available on AWS Marketplace
- Smaller ecosystem
- Less production track record

**Development:**
- Modular (company-backed)
- Newer to market
- Smaller docker images (700MB vs vLLM)

**Promising but less proven**

---

### Feature Comparison

| Feature | vLLM | MAX | Notes |
|---------|------|-----|-------|
| PagedAttention | ✅ | ❌ (naive cache) | vLLM advantage |
| Concurrent requests | 512 | 248 | vLLM scales better |
| OpenAI API | ✅ | ✅ | Both support |
| Ecosystem | Huge | Growing | vLLM advantage |
| Docker size | Larger | 700MB (90% smaller) | MAX advantage |
| Production track record | ✅✅✅ | ⚠️ | vLLM clear winner |

---

## vLLM vs Alternatives (NVIDIA GPU)

### Industry Consensus (2025)

**vLLM is the recommended choice for:**
- Startups and scale-ups (best balance)
- Flexible, rapid deployments
- Easy multi-GPU with Ray
- Good developer experience
- Production stability

**When to consider alternatives:**

1. **TensorRT-LLM:**
   - Need absolute peak performance (INT4/FP8)
   - Hardware-specific optimization
   - Willing to accept poor developer experience
   - Large enterprise with stable workloads

2. **SGLang:**
   - Similar to vLLM, slightly different trade-offs
   - Good developer experience

3. **TGI (Hugging Face):**
   - Equivalent speed to vLLM
   - Tight Hugging Face integration

4. **LMDeploy:**
   - Up to 1.8x faster than vLLM on A100
   - Less mature ecosystem

---

## Recommendation by Platform

### Linux + NVIDIA GPU: vLLM ✅

**Why:**
- ✅ Most widely adopted (production proven)
- ✅ Best ecosystem (Red Hat, PyTorch, community)
- ✅ Great developer experience
- ✅ Excellent performance (4.4x faster than Ollama)
- ✅ PagedAttention for efficiency
- ✅ Handles 512 concurrent requests

**Trade-off:**
- ⚠️ Slightly slower than TensorRT-LLM for peak performance
- ⚠️ MAX might be 16% faster (but less mature)

**Verdict:** Best balance of performance, stability, and DX

---

### macOS + Apple Silicon: llama.cpp ✅

**Why:**
- ✅ Only option with Metal acceleration
- ✅ 26-65 tok/s vs vLLM's ~10 tok/s
- ✅ What Ollama uses (proven)
- ✅ Can beat Ollama by removing overhead

**No alternatives:**
- ❌ vLLM: No Metal support planned
- ❌ MAX: CPU-only on macOS
- ❌ TensorRT-LLM: NVIDIA-only

**Verdict:** Only viable option, and it's excellent

---

### Windows + NVIDIA GPU: vLLM

**Status:** vLLM supports Windows (experimental)
**Recommendation:** Same as Linux - use vLLM

---

## MAX: Should We Test It?

### Arguments FOR Testing

1. **Performance:** 16% faster than vLLM in some benchmarks
2. **Consistency:** Better TTFT consistency
3. **Docker size:** 90% smaller images
4. **Worth knowing:** Could be future option

### Arguments AGAINST Priority

1. **Less mature:** Fewer production deployments
2. **Lower concurrency:** 248 vs 512 concurrent requests
3. **Naive cache:** No PagedAttention equivalent
4. **Ecosystem:** Much smaller community
5. **Doesn't solve macOS:** Still need llama.cpp

### Recommendation

**⏸️ Test later, not now**

**Timeline:**
1. Implement Phase 1: llama.cpp for macOS (priority)
2. Stabilize multi-engine architecture
3. THEN: Experiment with MAX on RTX 4090
4. Compare against vLLM benchmarks
5. If significantly better AND stable, consider switching

**Why not now:**
- Doesn't help macOS (the blocker)
- vLLM is working great (4.4x faster than Ollama)
- Limited benefit (16% speedup) vs risk (less mature)
- Should focus on getting macOS working first

---

## Final Architecture

```
┌─────────────────────────┐
│   Platform Detection    │
└───────────┬─────────────┘
            │
    ┌───────┴────────┐
    ↓                ↓
Linux/Windows   macOS/Apple Silicon
NVIDIA GPU
    ↓                ↓
  vLLM         llama.cpp
(PagedAttention)  (Metal)
    ↓                ↓
4.4x faster     Remove Ollama
than Ollama     overhead (40%)
```

**Engine Selection Logic:**
```rust
match (os, gpu) {
    (Linux | Windows, NvidiaGpu) => vLLM,  // Production proven
    (MacOS, AppleSilicon) => LlamaCpp,      // Only option with Metal
    (_, Cpu) => LlamaCpp,                   // Better CPU than vLLM
}
```

---

## Performance Claims (Post-Implementation)

### Linux + NVIDIA
> "4.4x faster than Ollama for sequential workloads, 1.6x faster for streaming. Powered by vLLM, the industry-standard inference engine trusted by Amazon, LinkedIn, and Red Hat."

### macOS + Apple Silicon
> "Faster than Ollama by removing abstraction overhead. Direct llama.cpp integration with Metal acceleration delivers 26-65 tok/s on M3/M3 Max."

### Cross-Platform
> "One tool for dev (macOS) and production (Linux). Best inference engine for each platform."

---

## Open Question: AMD GPUs?

**Current support:**
- vLLM: AMD support in roadmap (Q2/Q3 2025)
- llama.cpp: AMD via ROCm (experimental)

**Recommendation:**
Wait for vLLM AMD support to mature, then add Platform::LinuxAmd => vLLM

---

## Conclusion

**Stick with vLLM everywhere except macOS:**
- ✅ YES - vLLM is the best choice for Linux/Windows + NVIDIA
- ✅ YES - llama.cpp is only option for macOS
- ⏸️ MAYBE - Test MAX later as optimization experiment

**Priority:**
1. Implement llama.cpp for macOS (Phase 1)
2. Get cross-platform architecture stable
3. Experiment with MAX if interested

**No need to second-guess vLLM choice** - it's industry-standard for good reason.

---

*Sources: vLLM GitHub, Modular docs, industry benchmarks, production deployments*
