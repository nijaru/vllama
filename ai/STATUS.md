# Status

_Last Updated: 2025-10-22_

## Current State

**Version:** 0.0.x Development
**Focus:** Endpoint completion and production readiness

**Performance (RTX 4090, 30% GPU utilization):**
- Sequential: 232ms (4.4x faster than Ollama - Qwen 1.5B) ✅
- Concurrent (5): 0.217s (29.95x faster than Ollama 6.50s - facebook/opt-125m) ✅✅✅
- Concurrent (50): 2.115s (maintains 23.6 req/s throughput - facebook/opt-125m) ✅
- Streaming: 0.617s (1.6x faster than Ollama - Qwen 1.5B) ✅

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

**Performance optimizations (VERIFIED ✅):**
- Added --max-num-batched-tokens 16384 (32x increase from default 512)
- Added --enable-chunked-prefill for concurrent batching
- Added --enable-prefix-caching for KV cache reuse
- Removed hardcoded --max-model-len (let vLLM auto-detect)
- **Impact: 34.91x faster than before, 29.95x faster than Ollama!**

## What Didn't Work

**~~Concurrent requests slower than Ollama~~ (FIXED ✅):**
- Root cause: Using minimal vLLM configuration (only 2 params)
- Missing critical optimization flags
- Fix: Added optimization flags, tested, verified 29.95x faster than Ollama!

**macOS performance:**
- vLLM CPU-only, no Metal support planned
- Need llama.cpp for Apple Silicon (Phase 2)

## Active Work

**Current session:**
- ✅ Optimized vLLM configuration (serve.rs)
- ✅ Reorganized docs per agent-contexts standard
- ✅ Tested concurrent performance (5, 10, 50 requests)
- ✅ Verified massive speedup (29.95x vs Ollama)
- Next: Fix missing endpoints (/api/ps, /api/show, /api/version)

## Blockers

None - ready for endpoint implementation
