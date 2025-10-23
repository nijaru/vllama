# Status

_Last Updated: 2025-10-20_

## Current State

**Version:** 0.0.x Development
**Focus:** Performance optimization and endpoint completion

**Performance (measured with Qwen 1.5B on RTX 4090):**
- Sequential: 232ms (4.4x faster than Ollama) ✅
- Concurrent (5): 7.57s (1.16x SLOWER than Ollama) ❌
- Streaming: 0.617s (1.6x faster than Ollama) ✅

**Endpoints:**
- ✅ /api/generate (streaming + non-streaming)
- ✅ /api/chat (proper chat templates via vLLM)
- ✅ /api/pull (HuggingFace model downloads)
- ✅ /api/tags (list models)
- ⚠️ /api/ps (returns empty array - needs fix)
- ⚠️ /api/show (limited metadata - needs improvement)
- ❌ /api/version (missing)
- ❌ /api/embeddings (skipped for 0.0.x - RAG use case)

**Platform support:**
- ✅ Linux + NVIDIA GPU (production ready)
- ⚠️ macOS CPU-only (experimental, slow - need llama.cpp)

## What Worked

**Phase 4+ cleanup:**
- Removed all custom wrappers (1,165 lines deleted)
- Uses vLLM official OpenAI server directly
- Clean architecture: Client → vLLama (Rust) → vLLM OpenAI Server → GPU
- uv integration for Python environment management
- Proper chat completion endpoint (uses vLLM's /v1/chat/completions)

**Performance optimizations (just added):**
- Added --max-num-batched-tokens 16384 (32x increase from default)
- Added --enable-chunked-prefill for concurrent batching
- Added --enable-prefix-caching for KV cache reuse
- Expected impact: Fix concurrent performance bottleneck

## What Didn't Work

**Concurrent requests slower than Ollama:**
- Root cause: Using minimal vLLM configuration (only 2 params)
- Missing critical optimization flags
- Fix: Added optimization flags (testing needed)

**macOS performance:**
- vLLM CPU-only, no Metal support planned
- Need llama.cpp for Apple Silicon (Phase 2)

## Active Work

**Current session:**
- ✅ Optimized vLLM configuration (serve.rs)
- ✅ Reorganized docs per agent-contexts standard
- Next: Test optimizations, fix endpoints

## Blockers

- GPU testing requires stopping gdm (desktop environment)
- Need concurrent request testing to validate optimizations
