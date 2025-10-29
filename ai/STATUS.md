# Status

_Last Updated: 2025-10-29_

## Current State

**Version:** 0.0.5 (tested - core functionality working!)
**Focus:** Linux + NVIDIA GPU deployments (core validated, deployment untested)

**Reality Check:** Core vllama server now tested and working, critical bugs fixed.
**Deployment:** Infrastructure configs moved to `deployment-configs` branch (untested)

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
- ✅ Linux + NVIDIA GPU (core tested, deployment untested)
- ⚠️ macOS CPU-only (experimental, slow - need llama.cpp)

## Testing Status (2025-10-29)

### What's Actually Tested ✅

**Unit Tests:** 14 tests, ALL PASSING
- Config file loading and merging
- Error message formatting
- CLI output formatting (symbols, no emojis)
- Model downloader creation
- OpenAI client creation
- Request serialization

**Integration Tests:** 8 tests, ALL PASSING
- test_health_endpoint
- test_version_endpoint
- test_ps_endpoint
- test_show_endpoint
- test_show_endpoint_not_found
- test_chat_endpoint_non_streaming
- test_generate_endpoint_non_streaming
- test_openai_chat_completions

**Manual Server Tests:** ✅ VERIFIED
- Server startup (Qwen/Qwen2.5-0.5B-Instruct, 64s < 120s timeout)
- Health endpoint returns JSON with status, vllm_status, GPU info
- Generation endpoint works (model responds correctly)
- Shutdown cleanup verified:
  - Both processes killed (uv parent + python vLLM child)
  - GPU memory released (back to 33 MiB baseline)
  - Ports released (11434 and 8100)

### Critical Bugs Found & Fixed

**Bug 1: Startup Timeout Too Short (60s → 120s)**
- vLLM first startup takes ~67s due to CUDA graph compilation
- Old timeout: 60s (caused false failures)
- New timeout: 120s (allows startup to complete)
- Fixed in commit 4cd44b8

**Bug 2: Orphaned vLLM Subprocess on Error**
- `uv run` spawns Python subprocess
- Old code only killed parent, leaving vLLM orphaned
- Required manual `kill -9` to clean up
- Fixed with process group killing (SIGTERM then SIGKILL)
- Fixed in commit 4cd44b8

### What's NOT Tested ❌

**Deployment Infrastructure** (moved to `deployment-configs` branch):
- Docker, docker-compose
- Systemd service
- Nginx/Caddy reverse proxy configs
- Prometheus/Grafana monitoring
- All deployment documentation

**Long-running stability:**
- Memory leaks
- Log file rotation (vllm.log grows unbounded)
- Multi-day uptime
- Error recovery under load

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

**0.0.7 (Production Infrastructure) - CREATED BUT UNTESTED:**

**⚠️ CRITICAL: These configs have NOT been tested. They are templates based on industry best practices but require validation before production use.**

What was created (a302f51):
- ⚠️ Docker deployment configs (NOT TESTED)
  - Dockerfile with NVIDIA CUDA support
  - docker-compose.yml with monitoring stack
  - Health checks and automatic restarts
  - Volume mounts for model caching
- ⚠️ Systemd service (NOT TESTED)
  - deployment/vllama.service for bare-metal
  - Security hardening, automatic restart
  - Proper logging to journald
- ⚠️ Reverse proxy configurations (NOT VALIDATED)
  - Nginx: HTTPS, rate limiting (10 req/s), auth
  - Caddy: Automatic HTTPS, simpler config
- ⚠️ Monitoring infrastructure (NOT TESTED)
  - Prometheus + Grafana configs
  - GPU monitoring (NVIDIA DCGM)
  - Alert rules (downtime, latency, OOM)
  - Log aggregation guides (Loki, ELK)
- ✅ Documentation (written, not validated)
  - docs/DEPLOYMENT.md (600+ lines)
  - docs/MONITORING.md (400+ lines)
  - docs/SECURITY.md (500+ lines)
  - docs/TESTING_DEPLOYMENT.md (comprehensive testing guide)

**Testing Status:**
- ❌ Docker build not tested (Docker not available on dev machine)
- ❌ docker-compose not tested
- ❌ Systemd service not tested
- ❌ Nginx/Caddy configs syntax not validated
- ❌ Monitoring stack not tested
- ❌ End-to-end deployment not tested

**Known Issues to Fix:**
- Python package structure (python/pyproject.toml) doesn't exist - Docker build will fail
- NVIDIA runtime configuration not validated
- SSL certificate paths in nginx.conf reference non-existent files
- User/directory setup not scripted (manual steps required)
- No CI/CD to validate configurations automatically

**Next (0.0.8 - Testing & Validation):**
- Create Python package structure for Docker build
- Set up CI to test Docker build
- Test deployment configs on clean VM
- Fix issues found during testing
- Document actual test results
- Get real user to test deployment
- Stay in 0.0.x until proven

## Blockers

**Need testing infrastructure:**
- Docker + NVIDIA runtime for build testing
- Clean VM/server for deployment testing
- GPU-enabled CI (expensive/complex)

**Can't fully test without:**
- Real deployment environment
- User feedback on configs

## Development Notes

### Port Conflicts (Fedora)

**Important for development on Fedora:**

vllama uses port 11434 (same as Ollama). You must stop Ollama before starting vllama:

```bash
# Stop Ollama service
sudo systemctl stop ollama

# Or disable auto-start
sudo systemctl disable ollama

# Verify port is free
lsof -i:11434  # Should return nothing
```

**For benchmarking (need both running):**
```bash
# vllama on default port
vllama serve --model <model> --port 11434

# Ollama on alternate port
OLLAMA_HOST=127.0.0.1:11435 ollama serve
```

This is documented in:
- [docs/FEDORA_SETUP.md](../docs/FEDORA_SETUP.md)
- [CLAUDE.md](../CLAUDE.md)
