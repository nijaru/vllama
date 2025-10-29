# Status

_Last Updated: 2025-10-29_

## Current State

**Version:** 0.0.5 (experimental)
**Focus:** Linux + NVIDIA GPU deployments (experimental)

**Reality Check:** Still in 0.0.x - extremely experimental until real-world proven

**Strategy:** "Ollama's DX with vLLM's performance"
- Target: Production Linux with NVIDIA GPUs (when proven)
- NOT targeting: macOS/hobbyists (Ollama great there)
- NOT targeting: Researchers (use raw vLLM)
- NOT claiming: Production-ready (needs real users first)

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
- ✅ /api/ps (queries vLLM for running models)
- ✅ /api/show (queries vLLM for model metadata)
- ✅ /api/version (returns vllama version)
- ❌ /api/embeddings (skipped for 0.0.x - RAG use case)

**Platform support:**
- ✅ Linux + NVIDIA GPU (production ready)
- ⚠️ macOS CPU-only (experimental, slow - need llama.cpp)

## What Worked

**Phase 4+ cleanup:**
- Removed all custom wrappers (1,165 lines deleted)
- Uses vLLM official OpenAI server directly
- Clean architecture: Client → vllama (Rust) → vLLM OpenAI Server → GPU
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

**0.0.4 Completed:**
- ✅ Tested popular models (Qwen 2.5: 0.5B, 1.5B, 7B; Mistral 7B)
- ✅ Created comprehensive docs/MODELS.md
- ✅ Updated README.md with model references
- ✅ Documented GPU memory requirements (7B needs 90% utilization)
- ✅ Documented authentication requirements for Llama models

**0.0.5 Complete (Production Polish):**
- ✅ Modern CLI UX with clean symbols (→ • ✓ ✗), no emojis
- ✅ Progress indicators (spinner for vLLM startup)
- ✅ Output modes: --quiet, --json for scripting
- ✅ vLLM output redirected to vllm.log (clean terminal)
- ✅ Consistent branding (vllama lowercase everywhere)
- ✅ User-friendly error messages with helpful suggestions
- ✅ Enhanced /health endpoint (GPU, memory, models, vLLM status)
- ✅ Structured JSON logging (VLLAMA_LOG_FORMAT=json)
- ✅ Request tracking (UUIDs, latency, status codes)

**0.0.5.5 (Model Management & Config) - Complete:**
- ✅ Model management improvements (4b89145)
  - vllama pull: Modern progress bars, output modes (normal/quiet/json)
  - vllama list: Show cached models with sizes and paths
  - vllama rm: Delete models with size tracking
  - All commands support --quiet and --json flags
- ✅ Config file support (00f2846)
  - TOML config files: ~/.config/vllama/config.toml or ./vllama.toml
  - Config precedence: CLI flags > ./vllama.toml > ~/.config/vllama/config.toml > defaults
  - vllama config: Generate example config
  - vllama config --show: Display current loaded config
  - Server, model, logging, output settings

**0.0.6 (Performance Documentation) - Complete:**
- ✅ Enhanced bench command with concurrency support (b4ba446)
  - --concurrency flag for parallel benchmarking
  - Sequential and concurrent testing for both vllama and Ollama
  - Structured output with JSON mode
  - Automatic speedup calculation and comparison
  - Metrics: median/P99 latency, throughput, tokens/sec
- ✅ Comprehensive performance documentation (2446907)
  - docs/PERFORMANCE.md with benchmarking methodology
  - Performance comparison tables (sequential, concurrent 5, concurrent 50)
  - Real-world impact examples (chatbot, content generation)
  - Hardware recommendations (dev, production, enterprise)
  - When to use vllama vs Ollama decision guide
  - GPU memory requirements per model
  - FAQ and troubleshooting

**Competitive Analysis:**
- Positioning: "Ollama's DX with vLLM's performance"
- Moat: 29.95x faster concurrent (PagedAttention), production focus
- Real-world impact: 6 GPU cost savings (chatbots), 24x speedup (content gen)
- Target: Linux production deployments, high-throughput APIs, observability
- NOT competing: Cross-platform, GUI, beginner ease

**Next (0.0.7 - First Production User):**
- Deployment documentation (Docker, systemd, nginx)
- Security guidance (input validation, rate limiting, auth)
- Prometheus/Grafana monitoring setup
- Share on r/LocalLLaMA, r/rust, HN
- Get real user feedback and bug reports
- Stay in 0.0.x until production-proven

## Blockers

None
