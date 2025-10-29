# vllama

**Ollama's DX with vLLM's performance**

The fastest LLM inference server for Linux + NVIDIA GPUs.

## Why vllama?

- **29.95x faster** - Concurrent requests obliterate Ollama (vLLM's PagedAttention)
- **Ollama-compatible** - Drop-in replacement, same API (port 11434)
- **Production-focused** - Enhanced monitoring, JSON logging, error handling
- **Simple setup** - Easier than raw vLLM, faster than Ollama
- **Proven performance** - Industry-standard vLLM engine (Amazon, LinkedIn, Red Hat)
- **Observability** - Request tracking, latency metrics, GPU monitoring

**Target users:** Production deployments, high-throughput APIs, multi-user applications

**Current Status:** 0.0.5 - Core functionality tested and working (22 tests passing). Deployment configs available in `deployment-configs` branch. See [ai/STATUS.md](ai/STATUS.md) and [TESTING_STATUS.md](TESTING_STATUS.md) for details.

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

# Build vllama
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

**Why vllama is faster:**
- **29.95x faster** concurrent inference (vLLM's PagedAttention)
- **Continuous batching** - GPU never idles
- **Optimized CUDA kernels** - FlashAttention-2
- **2-3x better memory efficiency** - serve more users per GPU

**Benchmark results (RTX 4090):**
- **Sequential:** 4.4x faster than Ollama (232ms vs 1024ms)
- **Concurrent (5):** 29.95x faster than Ollama (0.217s vs 6.50s)
- **High concurrency (50):** 23.6 req/s sustained throughput

**Real-world impact:**
- **Chatbot (100 users):** 1 GPU instead of 7 GPUs (6 GPU cost savings)
- **Content generation (1,000 items):** <1 minute instead of 17 minutes

See [docs/PERFORMANCE.md](docs/PERFORMANCE.md) for comprehensive benchmarks, methodology, and hardware recommendations.

**Run benchmarks yourself:**
```bash
# Sequential
vllama bench <model> --iterations 10

# Concurrent (5 requests)
vllama bench <model> --iterations 50 --concurrency 5

# Save results as JSON
vllama bench <model> --iterations 50 --concurrency 5 --json > results.json
```

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
vllama Server (Rust, port 11434)
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

# 3. Build vllama
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
# Compare vllama vs Ollama (Ollama must run on port 11435)
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

## Production Deployment

**Note:** Deployment configurations (Docker, systemd, reverse proxy, monitoring) are available in the `deployment-configs` branch but have not been tested end-to-end. See [docs/TESTING_DEPLOYMENT.md](docs/TESTING_DEPLOYMENT.md) for status and testing checklist.

For production deployment, we recommend:
1. Start with the basic installation above
2. Run behind a reverse proxy (nginx/caddy) with SSL
3. Use systemd for process management
4. Monitor with the `/health` endpoint

See the `deployment-configs` branch for example configurations that need validation before production use.

## Documentation

**User Documentation:**
- [docs/PERFORMANCE.md](docs/PERFORMANCE.md) - Comprehensive performance benchmarks and analysis
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
- Model validation (Qwen 2.5, Mistral tested - see docs/MODELS.md)
- Deployment validation (Docker, systemd, monitoring configs need testing)
- Getting real user feedback

**Strategy:** Linux-only, vLLM-based, production-focused

See [ai/TODO.md](ai/TODO.md), [ai/STATUS.md](ai/STATUS.md), and [CLAUDE.md](CLAUDE.md) for development roadmap and current priorities.

Contributions welcome! Please check [ai/TODO.md](ai/TODO.md) for current priorities and open an issue to discuss major changes.

## License

This project is licensed under the Elastic License 2.0 - see the [LICENSE](LICENSE) file for details.

**Summary:**
- Free to use, modify, and distribute
- Free for commercial use and self-hosting
- Cannot be provided as a managed/hosted service without permission
- Source code is available for review and modification

For questions about licensing or commercial partnerships, please open an issue.

## Credits

Built with:
- [vLLM](https://github.com/vllm-project/vllm)
- [Axum](https://github.com/tokio-rs/axum)
- [Tokio](https://tokio.rs/)
