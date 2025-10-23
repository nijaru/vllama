# Research

_Index of external research findings_

---

## Engine Comparison (researched 2025-10-20)
**Key Finding:** vLLM best for Linux + NVIDIA, llama.cpp only option for macOS Metal
**Relevance:** Informs multi-engine architecture decision
**Decision:** Use vLLM (Linux) + llama.cpp (macOS future)
→ Details: ai/research/engine-comparison.md

---

## macOS Optimization Strategy (researched 2025-10-20)
**Key Finding:** vLLM has no macOS Metal support planned, llama.cpp is what Ollama uses
**Relevance:** Developer experience on macOS critical (M3 Max primary dev machine)
**Decision:** Add llama.cpp in Phase 2
→ Details: ai/research/macos-optimization.md

---

## vLLM Optimization Parameters (researched 2025-10-20)
**Sources:** vLLM docs, community benchmarks, RTX 4090 case studies
**Key Finding:** Default max-num-batched-tokens (512) too conservative, 8192-65536 optimal for 24GB GPU
**Relevance:** Fix concurrent performance (currently 1.16x slower than Ollama)
**Decision:** Use 16384 batched tokens, enable chunked-prefill and prefix-caching
→ Details: ai/research/vllm-optimization.md

---

## Planning Questions & Ollama Usage Patterns (researched 2025-10-20)
**Sources:** Ollama library stats, model popularity, embeddings use cases
**Key Findings:**
- 60% of users use 7-14B models (Llama 8B, Mistral 7B)
- Most users: 1-5 concurrent requests (personal use)
- Embeddings needed for RAG, not basic chat
**Relevance:** Informs optimization targets and API priorities
**Decision:** Optimize for 7-14B models, skip embeddings for 0.0.x
→ Details: ai/research/planning-questions.md

---

## Open Questions

- [ ] What's the optimal max-num-batched-tokens for 70B+ models on RTX 4090?
- [ ] Does vLLM V1 offer significant benefits for our use case?
- [ ] Should we test MAX Engine as future optimization?
- [ ] What quantization levels should we support for GGUF models (macOS)?
