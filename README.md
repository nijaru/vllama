# vLLama

**Ollama's DX with vLLM's performance**

The fastest LLM inference server for Linux + NVIDIA GPUs.

## Why vLLama?

- 🚀 **29.95x faster** - Concurrent requests obliterate Ollama (vLLM's PagedAttention)
- 🔌 **Ollama-compatible** - Drop-in replacement, same API (port 11434)
- 🎯 **Production-ready** - Built for Linux deployments with NVIDIA GPUs
- 🔧 **Simple setup** - Easier than raw vLLM, faster than Ollama
- 📊 **Proven performance** - Industry-standard vLLM engine (Amazon, LinkedIn, Red Hat)

**Target users:** Production deployments, high-throughput APIs, multi-user applications

**Current Status:** 0.0.3 (development) - See [ai/STATUS.md](ai/STATUS.md) for progress

## Platform Support

| Platform | Status | Notes |
|----------|--------|-------|
| **Linux + NVIDIA GPU** | ✅ Supported | Recommended - 29.95x faster than Ollama |
| **macOS / Windows** | ❌ Not yet | Use [Ollama](https://ollama.com) (great for these platforms!) |

**Why Linux-only?** vLLM is architecturally superior to llama.cpp on NVIDIA GPUs. macOS would use llama.cpp (same as Ollama) with only modest gains. We focus on where we can truly dominate: Linux production deployments.

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
cargo run --release --bin vllama -- serve --model Qwen/Qwen2.5-1.5B-Instruct

# Or use 7B model with custom settings
cargo run --release --bin vllama -- serve \
  --model Qwen/Qwen2.5-7B-Instruct \
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
    "model": "Qwen/Qwen2.5-1.5B-Instruct",
    "prompt": "Explain quantum computing in one sentence.",
    "stream": false
  }'

# Stream responses
curl -X POST http://localhost:11434/api/generate \
  -H "Content-Type: application/json" \
  -d '{
    "model": "Qwen/Qwen2.5-1.5B-Instruct",
    "prompt": "Write a haiku about coding.",
    "stream": true
  }'
```

## Performance

**RTX 4090 Performance:**
- **Sequential:** 232ms (4.4x faster than Ollama)
- **Concurrent (5):** 0.217s (29.95x faster than Ollama)
- **Concurrent (50):** 2.115s (23.6 req/s throughput)
- Optimized for GPU acceleration with PagedAttention
- Efficient memory management

See [docs/MODELS.md](docs/MODELS.md) for per-model performance characteristics.

**Benchmarking:**
Use `vllama bench` to compare vLLama vs Ollama on your hardware.
See [BENCHMARKS.md](BENCHMARKS.md) for detailed setup, methodology, and result templates.

## Supported APIs

**Ollama-Compatible:**
- ✅ `POST /api/generate` - Text generation (streaming + non-streaming)
- ✅ `POST /api/chat` - Chat completions (streaming + non-streaming)
- ✅ `POST /api/pull` - Download models from HuggingFace
- ✅ `POST /api/show` - Model metadata
- ✅ `GET /api/tags` - List loaded models
- ✅ `GET /api/ps` - Running models and performance
- ✅ `GET /api/version` - Version information
- ✅ `GET /health` - Health check

**OpenAI-Compatible:**
- ✅ `POST /v1/chat/completions` - OpenAI chat API

**Out of Scope:**
- ❌ `/api/push` - Model uploads
- ❌ `/api/embed` - Embeddings
- ❌ `/api/copy`, `/api/delete` - Manual model management
- ❌ Modelfiles - Use HuggingFace models directly

## Supported Models

See [docs/MODELS.md](docs/MODELS.md) for full compatibility matrix and hardware requirements.

**Tested & Working:**
- **Qwen 2.5** (0.5B, 1.5B, 7B) - Best for testing, open access
- **Mistral 7B v0.3** - Great for coding/chat

**Requires Authentication:**
- **Meta Llama models** - See [docs/MODELS.md](docs/MODELS.md) for setup

Models auto-download on first request. Any HuggingFace model compatible with vLLM should work.

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
  "Qwen/Qwen2.5-1.5B-Instruct" \
  "Test prompt" \
  -i 10
```

**Format code:**
```bash
cargo fmt
```

## Documentation

**User Documentation:**
- [docs/MODELS.md](docs/MODELS.md) - Model compatibility and requirements
- [docs/BENCHMARKS.md](docs/BENCHMARKS.md) - Benchmarking guide
- [docs/FEDORA_SETUP.md](docs/FEDORA_SETUP.md) - Fedora installation guide
- [docs/TESTING.md](docs/TESTING.md) - Testing guide

**Development Context (AI-optimized):**
- [ai/STATUS.md](ai/STATUS.md) - Current project state
- [ai/TODO.md](ai/TODO.md) - Active tasks and priorities
- [ai/DECISIONS.md](ai/DECISIONS.md) - Architectural decisions
- [ai/RESEARCH.md](ai/RESEARCH.md) - Research findings index

## Contributing

**Current focus (0.0.x development):**
- ✅ Model validation (Qwen 2.5, Mistral tested - see docs/MODELS.md)
- 🎯 Production polish (errors, CLI, monitoring)
- 🎯 Performance documentation
- 🎯 First production user

**Strategy:** Linux-only, vLLM-based, production-focused

See [ai/TODO.md](ai/TODO.md), [ai/STATUS.md](ai/STATUS.md), and [CLAUDE.md](CLAUDE.md) for details.

## License

[Add your license here]

## Credits

Built with:
- [vLLM](https://github.com/vllm-project/vllm)
- [Axum](https://github.com/tokio-rs/axum)
- [Tokio](https://tokio.rs/)
