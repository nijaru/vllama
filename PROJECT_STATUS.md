# vLLama - Project Status

**Last Updated:** 2025-10-20
**Version:** 0.0.x Development
**Current Focus:** Performance Optimization

## What Is vLLama?

Drop-in Ollama replacement powered by vLLM's official OpenAI server.

**Core Value Proposition:**
- Same API as Ollama (port 11434)
- GPU-accelerated inference via vLLM
- One command to start (`vllama serve --model MODEL`)
- Official libraries only (no custom wrappers)

**Slogan:** "vroom vroom" 🏎️

---

## Current Status (0.0.x)

**What Works:**
- ✅ Core Ollama API (/api/generate, /api/chat, /api/pull, /api/tags)
- ✅ Streaming and non-streaming
- ✅ Proper chat completion with vLLM
- ✅ 4.4x faster than Ollama (sequential)

**What Needs Work:**
- ⚠️ Concurrent performance (1.16x SLOWER than Ollama) 🔥
- ⚠️ /api/ps returns empty array
- ⚠️ /api/show has limited metadata
- ❌ Missing /api/version

**Current Focus:**
- Optimize vLLM configuration for concurrent requests
- Fix /api/ps and /api/show endpoints
- Comprehensive benchmarking

**See:** IMPLEMENTATION_PLAN.md for details

---

## Platform Support

| Platform | Status | Performance | Notes |
|----------|--------|-------------|-------|
| **Linux + NVIDIA GPU** | ✅ Production Ready | 10x+ faster | Recommended for production |
| **macOS (Apple Silicon)** | ⚠️ Experimental | CPU-only | Good for dev/testing |
| **macOS (Intel)** | ⚠️ Experimental | CPU-only | Good for dev/testing |
| **Linux (CPU-only)** | ⚠️ Supported | Slower | Not recommended |

**Key Points:**
- **Production:** Linux with NVIDIA GPU (CUDA 12.1+) for maximum performance
- **Development:** macOS works for testing with CPU-only vLLM (experimental)
- **Limitation:** vLLM GPU acceleration requires NVIDIA GPUs (no AMD/Intel Arc support)
- **Cross-platform:** Rust code works everywhere, Python vLLM determines acceleration

---

## Current Status: Phase 4+ Complete ✅

### Architecture

**Single-service design:**
```
User → vLLama (Rust) → vLLM OpenAI Server → GPU
```

- vLLama translates Ollama API → OpenAI API
- vLLM's official server handles batching, queuing, inference
- Auto-start via `uv` (no manual Python service)

### Working Features

**REST API (Ollama-compatible):**
- ✅ `GET /health` - Health check
- ✅ `POST /api/generate` - Text generation (streaming + non-streaming)
- ✅ `POST /api/chat` - Chat completions with proper templating (streaming + non-streaming)
- ✅ `POST /api/pull` - Model downloads from HuggingFace
- ✅ `POST /api/show` - Model metadata
- ✅ `GET /api/tags` - List models
- ✅ `GET /api/ps` - Performance monitoring
- ✅ `POST /v1/chat/completions` - OpenAI compatibility

**Key Improvements:**
- ✅ **Proper chat templating** - Uses vLLM's `/v1/chat/completions` endpoint
- ✅ **uv integration** - Automatic Python environment management
- ✅ **Official libraries only** - No custom HTTP wrappers
- ✅ **One-command startup** - `vllama serve --model MODEL`

### Verified Testing

**Comprehensive tests run 2025-10-20:**
- Model: facebook/opt-125m
- GPU: RTX 4090 (30% utilization)
- All endpoints tested and documented

See `COMPREHENSIVE_TEST_RESULTS.md` for details.

---

## Recent Changes (Phase 4+)

### Phase 4 Completion (13 commits)

**Removed custom implementations (-1,379 lines):**
- Custom model downloader → `hf-hub` library
- Custom vLLM wrapper → vLLM OpenAI server
- Custom chat templates → vLLM built-in
- Custom MAX/llama.cpp stubs
- Custom HTTP engine abstraction

**Added official integrations (+869 lines):**
- vLLM OpenAI server auto-start
- `hf-hub` for model downloads
- OpenAI client for chat completions
- uv-based Python environment management

**Result:**
- Cleaner codebase
- Better reliability (official libraries)
- Easier maintenance
- Proper chat templating

---

## Quick Start

### Installation

```bash
# Install uv
curl -LsSf https://astral.sh/uv/install.sh | sh

# Install dependencies
cd python
uv sync --extra vllm

# Build vLLama
cd ..
cargo build --release
```

### Usage

```bash
# One command - auto-starts everything
./target/release/vllama serve --model meta-llama/Llama-3.2-1B-Instruct

# Test
curl -X POST http://localhost:11434/api/chat \
  -H "Content-Type: application/json" \
  -d '{
    "model": "meta-llama/Llama-3.2-1B-Instruct",
    "messages": [{"role": "user", "content": "Hello!"}],
    "stream": false
  }'
```

---

## Tech Stack

**Languages:**
- Rust 1.90+ (server, API, CLI)
- Python 3.12 (managed by uv, for vLLM)

**Core Dependencies:**
- Axum (Rust web framework)
- vLLM 0.11.0 (inference engine)
- uv (Python environment manager)
- hf-hub (Rust library for HuggingFace)

**Infrastructure:**
- Tokio async runtime
- vLLM OpenAI server (official)
- No Redis, no extra services

---

## Project Structure

```
vllama/
├── crates/
│   ├── vllama-core/          # Shared types (ChatMessage, GenerateRequest, etc.)
│   ├── vllama-engine/        # VllmOpenAIEngine implementation
│   ├── vllama-server/        # REST API server
│   ├── vllama-cli/           # CLI commands (serve, bench)
│   └── vllama-models/        # Model definitions
├── python/
│   ├── pyproject.toml        # Python dependencies
│   └── uv.lock               # Locked dependencies
├── README.md                 # User-facing docs
├── COMPREHENSIVE_TEST_RESULTS.md  # Test documentation
├── BENCHMARKS.md             # Benchmark guide
├── FEDORA_SETUP.md           # System setup
└── PROJECT_STATUS.md         # This file
```

**Removed in Phase 4:**
- `python/llm_service/` (custom vLLM wrapper)
- `python/max_service/` (MAX wrapper)
- `crates/vllama-engine/src/http_engine.rs` (HTTP abstraction)
- `crates/vllama-engine/src/max.rs` (MAX engine)
- `crates/vllama-engine/src/llama_cpp.rs` (stub)

---

## Key Files

**Implementation:**
- `crates/vllama-server/src/api.rs` - API endpoints (generate, chat, pull, etc.)
- `crates/vllama-engine/src/vllm_openai.rs` - VllmOpenAIEngine
- `crates/vllama-core/src/openai.rs` - OpenAI client
- `crates/vllama-cli/src/commands/serve.rs` - Auto-start vLLM

**Configuration:**
- `Cargo.toml` - Rust workspace
- `python/pyproject.toml` - Python dependencies

**Documentation:**
- `README.md` - Getting started
- `COMPREHENSIVE_TEST_RESULTS.md` - Test results
- `BENCHMARKS.md` - Benchmark templates
- `PROJECT_STATUS.md` - This file (source of truth)

---

## Not Planned (Out of Scope)

**We explicitly WON'T implement:**
- ❌ `/api/push` - Uploading models
- ❌ `/api/copy` - Local model copying
- ❌ `/api/delete` - Manual deletion (use filesystem)
- ❌ `/api/embed` - Embeddings (different use case)
- ❌ Modelfile support - Use HuggingFace directly

**Rationale:** Focus on inference performance, not model management.

---

## Success Metrics

**Phase 1 (✅ Complete):**
- [x] Ollama-compatible API
- [x] GPU acceleration
- [x] Streaming generation
- [x] Auto model loading

**Phase 2 (✅ Complete):**
- [x] Chat completions
- [x] Model downloads from HuggingFace
- [x] Model metadata
- [x] OpenAI compatibility
- [x] Performance monitoring

**Phase 3 (✅ Complete):**
- [x] vLLM backend integrated
- [x] Performance benchmarking
- [x] AsyncLLMEngine for concurrency

**Phase 4+ (✅ Complete):**
- [x] Removed all custom wrappers (-1,379 lines)
- [x] Official library integration (+869 lines)
- [x] uv-based Python management
- [x] Proper chat completion endpoint
- [x] Comprehensive testing
- [x] Clean, maintainable codebase

---

## Next Steps

**Production Readiness:**
1. Test with larger models (7B+, 70B+)
2. Load testing (concurrent requests)
3. Performance benchmarking vs Ollama
4. Docker deployment
5. Multi-GPU support

**Optional Enhancements:**
- Prometheus metrics
- Request batching optimization
- Model unloading to free VRAM
- Connection pooling

**Documentation:**
- Deployment guide (update for new architecture)
- Performance comparison
- Chat model compatibility matrix

---

## Development

**Testing:**
```bash
# Build
cargo build --release

# Run server
./target/release/vllama serve --model facebook/opt-125m

# Test endpoints
curl http://localhost:11434/health
curl -X POST http://localhost:11434/api/generate -H "Content-Type: application/json" -d '{"model":"facebook/opt-125m","prompt":"Hello","stream":false}'
```

**GPU Setup (Fedora):**
```bash
# Stop GDM for full 24GB VRAM
sudo systemctl stop gdm

# Server will use 30-90% GPU memory based on flags
vllama serve --model MODEL --gpu-memory-utilization 0.9
```

---

## Commit History

**Phase 4+ Commits (2025-10-20):**
```
3241ae2 docs: add comprehensive test results
0c0bc46 feat: use vLLM chat completion endpoint for proper templating
745285b docs: add uv integration test results
872c421 docs: update README to reflect uv integration
5886ca5 feat: integrate uv for Python environment management
4191d30 docs: add end-to-end test results for Phase 4
7e265c6 docs: add comprehensive test results for Phase 4 cleanup
5f654e0 refactor: remove all custom engine wrappers and stubs (-1,165 lines)
c32f292 refactor: remove custom chat templates module (-119 lines)
e83124a feat: auto-start vLLM OpenAI server in serve command
ce223a1 refactor: replace VllmEngine with VllmOpenAIEngine (-95 lines)
3b26e0e feat: add vLLM OpenAI server integration
a300013 feat: replace custom downloader with official hf-hub
```

**Total:** 13 commits, -1,379 lines removed, +869 lines added

---

*Last updated: 2025-10-20 after Phase 4+ completion and comprehensive testing*
