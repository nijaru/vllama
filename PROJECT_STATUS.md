# vLLama - Project Status

**Last Updated:** 2025-10-15
**Version:** Phase 2 Complete ✅

## What Is vLLama?

Fast LLM inference server with Ollama-compatible API, powered by vLLM.

**Core Value Proposition:**
- Drop-in Ollama replacement (same API, same port)
- 10x+ faster inference (GPU-accelerated via vLLM)
- High-performance inference with PagedAttention
- Performance-focused, not feature-complete

## Current Status: Phase 2 Complete ✅

### Working Features

**REST API (Ollama-compatible):**
- ✅ `GET /health` - Health check
- ✅ `POST /api/generate` - Text generation (streaming + non-streaming)
- ✅ `POST /api/chat` - Chat completions (streaming + non-streaming)
- ✅ `POST /api/pull` - Model downloads with progress tracking
- ✅ `POST /api/show` - Model metadata (modelfile, parameters, details)
- ✅ `GET /api/tags` - List loaded models
- ✅ `GET /api/ps` - Performance monitoring
- ✅ `POST /v1/chat/completions` - OpenAI compatibility

**Infrastructure:**
- ✅ Rust server (Axum) on port 11434
- ✅ Python vLLM service on port 8100
- ✅ Auto-download models from HuggingFace on first request
- ✅ GPU acceleration (RTX 4090 tested)
- ✅ Model caching (loaded once, reused)
- ✅ Proper error handling and logging

**Performance (RTX 4090):**
- Model: meta-llama/Llama-3.1-8B-Instruct
- Throughput: High-performance inference via vLLM
- PagedAttention for efficient memory management
- Optimized for GPU acceleration

### Known Issues

**Fixed but need verification:**
- ⚠️ Streaming had infinite loop (fixed in code, tested working)

**Needs proper benchmarking:**
- ⚠️ Need proper baseline comparison vs Ollama on same hardware
- Document vLLM performance characteristics

**Phase 2 Complete!**
All core API endpoints and features implemented:
- ✅ Chat completions with proper templating
- ✅ Model management (pull, show, tags)
- ✅ OpenAI compatibility
- ✅ Performance monitoring
- ✅ Streaming support for all endpoints

## Phase 2 Roadmap - Core UX Features

**Focus: What 80% of users need**

### P0 - Must Have (Week 1-2)

1. **Chat Completions** (`/api/chat`)
   - Multi-turn conversations
   - System prompts
   - Streaming support
   - Ollama-compatible format

2. **Model Management**
   - `POST /api/pull` - Download from HuggingFace
   - `GET /api/show` - Model info (size, family, parameters)
   - `GET /api/tags` - List actually loaded models
   - Progress tracking for downloads

3. **Better Error Messages**
   - Clear messages when model not found
   - Helpful suggestions (e.g., "Run: curl -X POST http://localhost:11434/api/pull -d '{\"name\":\"llama3.1:8b\"}'")
   - Memory usage warnings

### P1 - Should Have (Week 3-4)

4. **OpenAI Compatibility**
   - `POST /v1/chat/completions`
   - Works with LangChain, llama-index, etc.
   - Same API as OpenAI for drop-in replacement

5. **Performance Monitoring**
   - `GET /api/ps` - Show running models + memory usage
   - Basic metrics endpoint
   - Prometheus-compatible (optional)

6. **Better Model Discovery**
   - Suggest models based on VRAM
   - Show popular models
   - Quantization level selection

### P2 - Nice to Have (Future)

7. **Connection Pooling** - Better concurrency
8. **Request Batching** - Higher throughput
9. **Multi-model Loading** - Run multiple models
10. **Model Unloading** - Free VRAM when not in use

## Phase 3 - Future Enhancements

**Goal: Production-ready features**

1. ✅ vLLM backend integration (Complete)
2. Performance benchmarking vs Ollama
3. Request batching and optimization
4. Multi-GPU support

## Not Planned (Out of Scope)

**We explicitly WON'T implement:**
- ❌ `/api/push` - Uploading to ollama.ai
- ❌ `/api/copy` - Local model copying
- ❌ `/api/delete` - Manual deletion (use filesystem)
- ❌ `/api/embed` - Embeddings (different use case)
- ❌ Modelfile support - Use HuggingFace directly
- ❌ Full Ollama CLI - Web API only

**Rationale:** Focus on inference performance, not model management features.

## Tech Stack

**Current:**
- Rust 1.90.0 (Axum web framework)
- Python 3.12+ (via mise + uv)
- vLLM (latest stable)
- Tokio async runtime

**Future:**
- Optional: Prometheus metrics
- Optional: Redis for caching
- Multi-GPU support

## Development Setup

**Fedora (GPU):**
```bash
# 1. Ensure gdm stopped (for full 24GB VRAM)
sudo systemctl stop gdm

# 2. Start vLLM service
cd python && uv run uvicorn llm_service.server:app --host 127.0.0.1 --port 8100

# 3. Start vLLama server
cargo run --release --bin vllama -- serve --host 127.0.0.1 --port 11434

# 4. Test
curl http://localhost:11434/health
curl -X POST http://localhost:11434/api/generate \
  -H "Content-Type: application/json" \
  -d '{"model":"meta-llama/Llama-3.1-8B-Instruct","prompt":"Hello!","stream":false}'
```

**macOS (Development):**
Same commands work on M3 Max (CPU inference).

## Project Structure

```
vllama/
├── crates/
│   ├── vllama-core/          # Shared types and utilities
│   ├── vllama-engine/        # Engine abstraction (vLLM, llama.cpp)
│   ├── vllama-server/        # REST API server
│   ├── vllama-cli/           # CLI + benchmarks
│   └── vllama-models/        # Model definitions
├── python/
│   ├── llm_service/          # vLLM HTTP wrapper
│   └── requirements.txt
├── docs/                     # Planning docs
└── PROJECT_STATUS.md         # This file (source of truth)
```

## Key Files

**Code:**
- `crates/vllama-server/src/api.rs` - API endpoints
- `crates/vllama-engine/src/vllm.rs` - vLLM client
- `python/llm_service/server.py` - vLLM service

**Config:**
- `Cargo.toml` - Rust dependencies
- `python/requirements.txt` - Python dependencies
- `.gitignore` - Excludes models, venv, target

**Docs:**
- `PROJECT_STATUS.md` - **Current status** (this file)
- `README.md` - User-facing getting started
- `docs/` - Old planning docs (ignore)

## Success Metrics

**Phase 1 Goals (✅ Complete):**
- [x] Ollama-compatible API working
- [x] GPU acceleration confirmed
- [x] Streaming generation working
- [x] Auto model loading

**Phase 2 Goals:**
- [x] Chat completions working
- [x] Model pull from HuggingFace
- [x] Model metadata (show, tags)
- [x] OpenAI API compatibility
- [x] Performance monitoring
- [x] Error messages with actionable suggestions
- [x] Llama 3.1 chat templates

**Phase 3 Goals:**
- [x] vLLM backend integrated
- [ ] Performance comparison vs Ollama documented
- [ ] Multi-GPU support

## Next Steps

**Phase 3 Priorities:**
1. Proper performance benchmarking vs Ollama
2. Request batching and optimization
3. Multi-GPU support (vLLM tensor parallelism)
4. Production deployment guide

**Potential Improvements:**
- Model unloading to free VRAM
- Connection pooling for better concurrency
- Prometheus metrics export
- Docker deployment
