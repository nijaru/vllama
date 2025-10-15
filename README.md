# HyperLlama

**Fast Ollama-compatible LLM server powered by Modular MAX Engine**

Drop-in replacement for Ollama with 10x+ faster GPU inference.

## Why HyperLlama?

- 🚀 **10x faster** - GPU-accelerated inference via MAX Engine
- 🔌 **Drop-in compatible** - Same API as Ollama (port 11434)
- 🎯 **Performance-focused** - Optimized for throughput, not feature parity
- 🔧 **Easy setup** - One command to start

## Quick Start

**Prerequisites:**
- NVIDIA GPU (CUDA 13.0+) or CPU
- Rust 1.90+
- Python 3.12+
- MAX Engine installed

**Start the server:**

```bash
# Terminal 1: Start MAX Engine service
cd python && uv run uvicorn max_service.server:app --host 127.0.0.1 --port 8100

# Terminal 2: Start HyperLlama
cargo run --release --bin hyperllama -- serve --host 127.0.0.1 --port 11434
```

**Use it:**

```bash
# Generate text
curl -X POST http://localhost:11434/api/generate \
  -H "Content-Type: application/json" \
  -d '{
    "model": "modularai/Llama-3.1-8B-Instruct-GGUF",
    "prompt": "Explain quantum computing in one sentence.",
    "stream": false
  }'

# Stream responses
curl -X POST http://localhost:11434/api/generate \
  -H "Content-Type: application/json" \
  -d '{
    "model": "modularai/Llama-3.1-8B-Instruct-GGUF",
    "prompt": "Write a haiku about coding.",
    "stream": true
  }'
```

## Performance

**RTX 4090 (Llama-3.1-8B-Instruct-GGUF):**
- Throughput: **59.07 tokens/sec** (direct MAX Engine)
- Latency: **846ms** average
- VRAM: 22GB

**vs CPU baseline:**
- M3 Max CPU: ~6-8 tokens/sec
- Speedup: ~8-10x on GPU

## Supported APIs

**Current (Phase 1):**
- ✅ `POST /api/generate` - Text generation (streaming + non-streaming)
- ✅ `GET /api/tags` - List models
- ✅ `GET /health` - Health check

**Coming Soon (Phase 2):**
- 🔜 `POST /api/chat` - Chat completions
- 🔜 `POST /api/pull` - Download models
- 🔜 `GET /api/show` - Model info
- 🔜 `POST /v1/chat/completions` - OpenAI compatibility

**Out of Scope:**
- ❌ `/api/push` - Model uploads
- ❌ `/api/embed` - Embeddings
- ❌ Modelfiles - Use HuggingFace models directly

## Supported Models

Any HuggingFace model compatible with MAX Engine:
- `modularai/Llama-3.1-8B-Instruct-GGUF`
- `modularai/Llama-3.1-70B-Instruct-GGUF`
- Other GGUF models (experimental)

Models auto-download on first request.

## Architecture

```
Client Request
    ↓
HyperLlama Server (Rust, port 11434)
    ↓ HTTP
MAX Engine Service (Python, port 8100)
    ↓
MAX Engine (Modular)
    ↓
GPU/CPU
```

## Installation

### Fedora (GPU)

```bash
# 1. Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 2. Install Python + MAX Engine
mise install python@3.12
mise use python@3.12
cd python && uv pip install -r requirements.txt
uv pip install modular --index-url https://dl.modular.com/public/nightly/python/simple/

# 3. Build HyperLlama
cargo build --release

# 4. Run (see Quick Start above)
```

### macOS (CPU)

Same commands - works on M3 Max with CPU inference.

## Development

**Run tests:**
```bash
cargo test
```

**Benchmark:**
```bash
# Note: Current benchmark is misleading (compares Python direct vs REST API)
# Proper vLLM/Ollama comparison coming in Phase 2
cargo run --release --bin hyperllama -- bench \
  "modularai/Llama-3.1-8B-Instruct-GGUF" \
  "Test prompt" \
  -i 10
```

**Format code:**
```bash
cargo fmt
```

## Project Status

See [PROJECT_STATUS.md](PROJECT_STATUS.md) for detailed roadmap and current phase.

**Current:** Phase 1 Complete ✅
**Next:** Phase 2 - Chat Completions + Model Management

## Contributing

Focus areas:
1. Chat completions endpoint
2. Model download progress tracking
3. OpenAI API compatibility
4. Performance optimizations

See PROJECT_STATUS.md for priority list.

## License

[Add your license here]

## Credits

Built with:
- [Modular MAX Engine](https://www.modular.com/max)
- [Axum](https://github.com/tokio-rs/axum)
- [Tokio](https://tokio.rs/)
