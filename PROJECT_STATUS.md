# HyperLlama - Project Status

**Last Updated:** 2025-10-15
**Version:** Phase 2 P0 Complete ✅

## What Is HyperLlama?

Fast LLM inference server with Ollama-compatible API, powered by Modular MAX Engine.

**Core Value Proposition:**
- Drop-in Ollama replacement (same API, same port)
- 10x+ faster inference (GPU-accelerated via MAX Engine)
- Support for MAX + vLLM backends
- Performance-focused, not feature-complete

## Current Status: Phase 1 ✅

### Working Features

**REST API (Ollama-compatible):**
- ✅ `GET /health` - Health check
- ✅ `POST /api/generate` - Text generation (streaming + non-streaming)
- ✅ `POST /api/chat` - Chat completions (streaming + non-streaming)
- ✅ `POST /api/pull` - Model downloads with progress tracking
- ✅ `POST /api/show` - Model metadata (modelfile, parameters, details)
- ✅ `GET /api/tags` - List loaded models

**Infrastructure:**
- ✅ Rust server (Axum) on port 11434
- ✅ Python MAX Engine service on port 8100
- ✅ Auto-download models from HuggingFace on first request
- ✅ GPU acceleration (RTX 4090 tested)
- ✅ Model caching (loaded once, reused)
- ✅ Proper error handling and logging

**Performance (RTX 4090):**
- Model: Llama-3.1-8B-Instruct-GGUF
- Throughput: 59.07 tokens/sec (direct MAX Engine)
- Latency: 846ms average
- VRAM: 22GB for 8B model

### Known Issues

**Fixed but need verification:**
- ⚠️ Streaming had infinite loop (fixed in code, tested working)

**Misleading metrics:**
- ⚠️ Benchmark compares direct Python calls vs REST API (not vs real Ollama)
- Need proper baseline: vLLM or actual Ollama on same hardware

**Phase 2 P0 - Complete!**
All core model management endpoints implemented.

**Remaining features (P1/P2):**
- ⏸️ OpenAI `/v1/chat/completions` compatibility (P1)
- ⏸️ Performance monitoring `/api/ps` (P1)
- ⏸️ Better prompt templates for chat (improvement)

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

## Phase 3 - vLLM Backend

**Goal: Performance comparison MAX vs vLLM**

1. Python vLLM service (same pattern as MAX service)
2. Automatic backend selection based on model
3. Side-by-side benchmarks
4. Documentation on when to use which

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
- Python 3.12.11 (via mise + uv)
- MAX Engine 25.5.0 (nightly)
- Tokio async runtime

**Future:**
- vLLM (TBD version)
- Optional: Prometheus metrics
- Optional: Redis for caching

## Development Setup

**Fedora (GPU):**
```bash
# 1. Ensure gdm stopped (for full 24GB VRAM)
sudo systemctl stop gdm

# 2. Start MAX Engine service
cd python && uv run uvicorn max_service.server:app --host 127.0.0.1 --port 8100

# 3. Start HyperLlama server
cargo run --release --bin hyperllama -- serve --host 127.0.0.1 --port 11434

# 4. Test
curl http://localhost:11434/health
curl -X POST http://localhost:11434/api/generate \
  -H "Content-Type: application/json" \
  -d '{"model":"modularai/Llama-3.1-8B-Instruct-GGUF","prompt":"Hello!","stream":false}'
```

**macOS (Development):**
Same commands work on M3 Max (CPU inference).

## Project Structure

```
hyperllama/
├── crates/
│   ├── hyperllama-core/      # Shared types
│   ├── hyperllama-engine/    # MAX Engine integration
│   ├── hyperllama-server/    # REST API server
│   ├── hyperllama-cli/       # CLI + benchmarks
│   └── hyperllama-models/    # Model definitions (unused)
├── python/
│   ├── max_service/          # MAX Engine HTTP wrapper
│   └── requirements.txt
├── docs/                     # Planning docs (mostly stale)
├── external/modular/         # Modular source for reference
└── PROJECT_STATUS.md         # This file (source of truth)
```

## Key Files

**Code:**
- `crates/hyperllama-server/src/api.rs` - API endpoints
- `crates/hyperllama-engine/src/max.rs` - MAX Engine client
- `python/max_service/server.py` - MAX Engine service

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

**Phase 2 Goals (P0):**
- [x] Chat completions working
- [x] Model pull from HuggingFace
- [x] Model metadata (show, tags)
- [x] 50+ tok/s on RTX 4090 (8B model)

**Phase 2 Goals (P1):**
- [ ] OpenAI API compatibility
- [ ] Performance monitoring

**Phase 3 Goals:**
- [ ] vLLM backend integrated
- [ ] Performance comparison documented
- [ ] Clear docs on MAX vs vLLM tradeoffs

## Getting Help

**Issues to fix:**
- Benchmark comparing wrong things (see bench.rs)
- Docs scattered across 12+ files
- No clear project vision until now

**Questions for user:**
- Which HuggingFace repos to support? (modularai/* only? or any?)
- Should we auto-quantize models, or require pre-quantized?
- Default to streaming or non-streaming?

## Next Session

**Immediate priorities:**
1. ✅ Implement `/api/chat` endpoint - DONE
2. Fix `/api/tags` to show loaded models
3. Add `/api/pull` with progress tracking
4. Add `/api/show` for model metadata
5. Improve chat prompt formatting (use Llama 3.1 chat template)
