# vLLama

**Fast Ollama-compatible LLM server powered by vLLM**

Drop-in replacement for Ollama with 10x+ faster GPU inference.

## Why vLLama?

- 🚀 **4.4x faster** - GPU-accelerated inference via vLLM (NVIDIA GPUs, sequential workloads)
- 🔌 **Ollama-compatible** - Same API as Ollama (port 11434)
- 🎯 **Performance-focused** - Optimized for throughput with official vLLM server
- 🔧 **Easy setup** - One command to start
- 🍎 **Cross-platform ready** - Linux + NVIDIA production, macOS dev support planned

**Current Status:** 0.0.x development - See [ai/STATUS.md](ai/STATUS.md) for progress

## Platform Support

| Platform | Status | Performance | Notes |
|----------|--------|-------------|-------|
| **Linux + NVIDIA GPU** | ✅ Production Ready | 10x+ faster | Recommended for production |
| **macOS (Apple Silicon)** | ⚠️ Experimental | CPU-only | Good for dev/testing |
| **macOS (Intel)** | ⚠️ Experimental | CPU-only | Good for dev/testing |
| **Linux (CPU-only)** | ⚠️ Supported | Slower | Not recommended |

## Quick Start

**Prerequisites:**
- **For production (Linux):** NVIDIA GPU + CUDA 12.1+
- **For development (macOS):** Apple Silicon or Intel CPU
- **All platforms:** Rust 1.90+, [uv](https://docs.astral.sh/uv/), Python 3.12.x

**Install:**

```bash
# Install uv (if not already installed)
curl -LsSf https://astral.sh/uv/install.sh | sh

# Install Python dependencies
cd python
uv sync --extra vllm  # Installs vLLM and dependencies

# Build vLLama
cd ..
cargo build --release
```

**Start the server:**

```bash
# One command - auto-starts vLLM OpenAI server
cargo run --release --bin vllama -- serve --model meta-llama/Llama-3.2-1B-Instruct

# Or use custom settings
cargo run --release --bin vllama -- serve \
  --model meta-llama/Llama-3.1-8B-Instruct \
  --port 11434 \
  --vllm-port 8100 \
  --max-num-seqs 256 \
  --gpu-memory-utilization 0.9
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
    ↓ OpenAI API
vLLM OpenAI Server (Python, port 8100)
    ↓
GPU/CPU
```

**Clean & Simple:**
- One Rust server (Ollama-compatible API)
- One Python process (vLLM's official OpenAI server)
- Standard OpenAI protocol in between
- No custom wrappers or abstractions

## Installation

### Fedora (GPU)

```bash
# 1. Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 2. Install Python dependencies
mise install python@3.12
mise use python@3.12
cd python && uv sync && uv pip install vllm

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

**User Documentation:**
- [docs/BENCHMARKS.md](docs/BENCHMARKS.md) - Benchmarking guide
- [docs/FEDORA_SETUP.md](docs/FEDORA_SETUP.md) - Fedora installation guide

**Development Context (AI-optimized):**
- [ai/STATUS.md](ai/STATUS.md) - Current project state
- [ai/TODO.md](ai/TODO.md) - Active tasks and priorities
- [ai/DECISIONS.md](ai/DECISIONS.md) - Architectural decisions
- [ai/RESEARCH.md](ai/RESEARCH.md) - Research findings index

## Contributing

**Current focus (0.0.x development):**
1. Fix concurrent performance (currently 1.16x slower than Ollama)
2. Complete core Ollama endpoints (/api/ps, /api/show, /api/version)
3. Comprehensive benchmarking
4. macOS support (llama.cpp integration)

See [ai/TODO.md](ai/TODO.md) and [ai/STATUS.md](ai/STATUS.md) for details.

## License

[Add your license here]

## Credits

Built with:
- [vLLM](https://github.com/vllm-project/vllm)
- [Axum](https://github.com/tokio-rs/axum)
- [Tokio](https://tokio.rs/)
