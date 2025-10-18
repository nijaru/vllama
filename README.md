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
- vLLM installed

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
- Optimized for GPU acceleration with PagedAttention
- Efficient memory management

**Benchmarking:**
Use `vllama bench` to compare vLLama vs Ollama on your hardware.
See [BENCHMARKS.md](BENCHMARKS.md) for detailed setup, methodology, and result templates.

## Supported APIs

**Ollama-Compatible:**
- ✅ `POST /api/generate` - Text generation (streaming + non-streaming)
- ✅ `POST /api/chat` - Chat completions (streaming + non-streaming)
- ✅ `POST /api/pull` - Download models from HuggingFace
- ✅ `GET /api/show` - Model metadata
- ✅ `GET /api/tags` - List loaded models
- ✅ `GET /api/ps` - Running models and performance
- ✅ `GET /health` - Health check

**OpenAI-Compatible:**
- ✅ `POST /v1/chat/completions` - OpenAI chat API

**Out of Scope:**
- ❌ `/api/push` - Model uploads
- ❌ `/api/embed` - Embeddings
- ❌ `/api/copy`, `/api/delete` - Manual model management
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
# Compare vLLama vs Ollama (Ollama must run on port 11435)
# Terminal 1: Start Ollama on alternate port
OLLAMA_HOST=127.0.0.1:11435 ollama serve

# Terminal 2: Run benchmark
cargo run --release --bin vllama -- bench \
  "meta-llama/Llama-3.1-8B-Instruct" \
  "Test prompt" \
  -i 10
```

**Format code:**
```bash
cargo fmt
```

## Documentation

- **[PROJECT_STATUS.md](PROJECT_STATUS.md)** - Current phase, roadmap, and feature status
- **[BENCHMARKS.md](BENCHMARKS.md)** - Performance testing methodology and templates
- **[DEPLOYMENT.md](DEPLOYMENT.md)** - Production deployment (systemd, Docker, security)
- **[FEDORA_SETUP.md](FEDORA_SETUP.md)** - Fedora + NVIDIA GPU setup guide

## Project Status

See [PROJECT_STATUS.md](PROJECT_STATUS.md) for detailed roadmap and current phase.

**Current:** Phase 2 Complete ✅
**Next:** Phase 3 - Performance Optimization & Production Readiness

## Contributing

Current Phase 3 focus areas:
1. Run benchmarks on real hardware (see [BENCHMARKS.md](BENCHMARKS.md))
2. Request batching and optimization
3. Multi-GPU support (vLLM tensor parallelism)
4. Documentation and examples

See [PROJECT_STATUS.md](PROJECT_STATUS.md) for complete priority list.

## License

[Add your license here]

## Credits

Built with:
- [vLLM](https://github.com/vllm-project/vllm)
- [Axum](https://github.com/tokio-rs/axum)
- [Tokio](https://tokio.rs/)
