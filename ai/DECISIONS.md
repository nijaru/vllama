# Decisions

_Architectural decisions and their rationale_

---

## 2025-10-20: Use vLLM for Linux, Plan llama.cpp for macOS

**Context:** Need optimal performance on both Linux (NVIDIA) and macOS (Apple Silicon)

**Decision:** Multi-engine architecture
- Linux/Windows + NVIDIA GPU → vLLM (current)
- macOS + Apple Silicon → llama.cpp (future - Phase 2)

**Rationale:**
- vLLM: Best for NVIDIA GPUs, industry standard (Amazon, LinkedIn, Red Hat)
- vLLM: 4.4x faster than Ollama on Linux (measured)
- vLLM: CPU-only on macOS, no Metal support planned (roadmap reviewed)
- llama.cpp: Only option with Metal acceleration for macOS
- llama.cpp: What Ollama uses, proven production backend
- llama.cpp: 26-65 tok/s on M3/M3 Max vs vLLM's ~10 tok/s CPU

**Tradeoffs:**
- Added complexity: Two engines, two model formats (HF vs GGUF)
- Mitigated by: Both have OpenAI-compatible servers, same client code

**Source:** See ai/research/engine-comparison.md, ai/research/macos-optimization.md

---

## 2025-10-20: Skip Embeddings for 0.0.x

**Context:** Deciding whether to implement /api/embeddings endpoint

**Decision:** Skip for 0.0.x, add later with RAG features

**Rationale:**
- Embeddings require separate embedding models (not LLMs)
- Use case is RAG applications (semantic search, knowledge bases)
- Most Ollama users use it for chat/completion, not embeddings
- vLLM V1 doesn't support embeddings yet (alpha status)
- Adds architectural complexity (separate model service)

**Tradeoffs:**
- Missing Ollama API compatibility for embedding use cases
- Can add later when implementing RAG features properly

---

## 2025-10-20: Optimize for 7-14B Models, 1-50 Concurrent

**Context:** Configuring vLLM optimization parameters

**Decision:**
- Target model size: 7-14B (Llama 8B, Mistral 7B, Qwen 7B)
- Target concurrency: 1-50 concurrent requests
- max-num-batched-tokens: 16384
- max-num-seqs: 256
- max-model-len: 4096

**Rationale:**
- Research shows 60% of Ollama users use 7-14B models
- Most users: 1-5 concurrent (personal), some 5-20 (teams), few 20-50+ (power users)
- RTX 4090 has 24GB VRAM, can handle 16384 batched tokens
- Default 512 batched tokens too conservative
- Research shows 8192-65536 optimal for 24GB GPU
- 16384 is conservative middle ground

**Tradeoffs:**
- Optimized for medium models, not large (70B+)
- Large models would need lower max-num-seqs (128 vs 256)

**Source:** See ai/research/planning-questions.md

---

## 2025-10-20: Version Strategy - Stay in 0.0.x

**Context:** When to release 0.1.0

**Decision:** Stay in 0.0.x for extended development, tag incrementally

**Rationale:**
- This is early development work
- No rush to ship 0.1.0
- Tag releases when features complete and stable
- 0.1.0 should represent: fully working, faster than Ollama everywhere, core endpoints complete

**Tradeoffs:**
- None - gives flexibility to iterate

---

## 2025-10-20: Remove Custom Wrappers, Use Official vLLM Server

**Context:** Phase 4 cleanup - simplify architecture

**Decision:** Remove all custom Python wrappers and HTTP abstractions

**Removed:**
- python/llm_service/ (280 lines) - custom vLLM wrapper
- python/max_service/ (258 lines) - custom MAX wrapper
- http_engine.rs (274 lines) - custom HTTP client
- max.rs (79 lines) - MAX engine implementation
- llama_cpp.rs (82 lines) - unimplemented stub

**Rationale:**
- vLLM has official OpenAI-compatible server
- Don't need custom wrappers around official tools
- Simpler architecture: Client → vLLama → vLLM OpenAI Server
- Less code to maintain
- Uses industry-standard OpenAI API

**Result:** 1,165 lines deleted, cleaner codebase

---

## 2025-10-13: Choose vLLM Over MAX for Initial Implementation

**Context:** Phase 0 tested MAX Engine, needed to decide on backend

**Decision:** Use vLLM as primary inference engine

**Rationale:**
- vLLM is industry standard (Amazon, LinkedIn, Red Hat)
- More mature ecosystem
- Better concurrency (512 concurrent vs MAX's 248)
- PagedAttention vs MAX's naive cache
- Proven production stability

**MAX consideration:**
- Slightly faster in some benchmarks (16%)
- Worth testing later as optimization
- Doesn't solve macOS problem (also CPU-only)

**Tradeoffs:**
- Could potentially be 16% slower than MAX in some workloads
- But more reliable, better scaling, proven in production

---
