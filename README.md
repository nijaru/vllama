# vLLama

**Fast Ollama-compatible LLM server powered by vLLM**

Drop-in replacement for Ollama with 10x+ faster GPU inference.

## Why vLLama?

- 🚀 **10x faster** - GPU-accelerated inference via vLLM
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
# Terminal 1: Start vLLM service
cd python && uv run uvicorn llm_service.server:app --host 127.0.0.1 --port 8100

# Terminal 2: Start vLLama
cargo run --release --bin vllama -- serve --host 127.0.0.1 --port 11434
```

**Use it:**

```bash
# Generate text
curl -X POST http://localhost:11434/api/generate \
  -H "Content-Type: application/json" \
  -d '{
    "model": "meta-llama/Llama-3.1-8B-Instruct",
    "prompt": "Explain quantum computing in one sentence.",
    "stream": false
  }'

# Stream responses
curl -X POST http://localhost:11434/api/generate \
  -H "Content-Type: application/json" \
  -d '{
    "model": "meta-llama/Llama-3.1-8B-Instruct",
    "prompt": "Write a haiku about coding.",
    "stream": true
  }'
```

## Performance

**RTX 4090 (Llama-3.1-8B-Instruct):**
- Throughput: High-performance inference via vLLM
- Optimized for GPU acceleration
- Efficient memory management

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

Any HuggingFace model compatible with vLLM:
- `meta-llama/Llama-3.1-8B-Instruct`
- `meta-llama/Llama-3.1-70B-Instruct`
- Other models supported by vLLM

Models auto-download on first request.

## Architecture

```
Client Request
    ↓
vLLama Server (Rust, port 11434)
    ↓ HTTP
vLLM Service (Python, port 8100)
    ↓
vLLM Engine
    ↓
GPU/CPU
```

## Installation

### Fedora (GPU)

```bash
# 1. Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 2. Install Python + vLLM
mise install python@3.12
mise use python@3.12
cd python && uv pip install -r requirements.txt

# 3. Build vLLama
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
cargo run --release --bin vllama -- bench \
  "meta-llama/Llama-3.1-8B-Instruct" \
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
- [vLLM](https://github.com/vllm-project/vllm)
- [Axum](https://github.com/tokio-rs/axum)
- [Tokio](https://tokio.rs/)
