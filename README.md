# HyperLlama

> High-performance local LLM inference server with exceptional developer experience

**Status**: ✅ Phase 0 Complete | 🚧 Phase 1 - REST API & Streaming

## What is HyperLlama?

HyperLlama is a local LLM inference server that delivers:
- **Simple developer experience**: One-command installation, auto-downloads, intuitive CLI
- **State-of-the-art performance**: Continuous batching, FlashAttention, speculative decoding, prefix caching
- **Universal hardware support**: NVIDIA, AMD, Apple Silicon, Intel, CPU with automatic optimization

### Performance Goals
- **Fast single requests**: 100+ tokens/sec on consumer GPUs
- **High throughput**: Efficient concurrent request handling
- **Memory efficient**: 50-60% reduction via advanced caching techniques

## Quick Start

### Prerequisites
- Rust 1.75+
- Python 3.12+
- MAX Engine: `pip install modular`

### Build & Run

```bash
# Build HyperLlama
cargo build --release

# Terminal 1: Start MAX Engine service
cd python && PYTHONPATH=python uvicorn max_service.server:app --host 127.0.0.1 --port 8100

# Terminal 2: Start HyperLlama server
cargo run --release --bin hyperllama -- serve --host 127.0.0.1 --port 11434

# Terminal 3: Test
curl http://localhost:11434/health
curl -X POST http://localhost:11434/api/generate \
  -H "Content-Type: application/json" \
  -d '{"model":"modularai/Llama-3.1-8B-Instruct-GGUF","prompt":"What is AI?","stream":false}'
```

### CLI Commands

```bash
hyperllama serve              # Start API server
hyperllama generate <model> "prompt"  # Generate text
hyperllama bench <model> "prompt" -i 5  # Benchmark
hyperllama info               # Hardware detection
```

## Architecture

**Core Technologies**:
- **Rust**: API server, CLI, model management, orchestration
- **Modular MAX Engine**: Primary inference engine with hardware-agnostic compilation
- **Fallback engines**: vLLM (GPU scenarios), llama.cpp (CPU scenarios)

**Key Optimizations**:
- Continuous batching (dynamic request scheduling)
- FlashAttention-2 (memory-efficient attention)
- PagedAttention (KV cache paging)
- Speculative decoding (2-3x latency reduction)
- Prefix caching (10-100x speedup for common prompts)

## Development Status

### Phase 0: Technology Validation ✅ COMPLETE
- ✅ Rust workspace with 5 crates
- ✅ MAX Engine integration via Python service
- ✅ Performance baseline: 23.71 tok/s on M3 Max CPU
- ✅ Hardware detection (Apple Silicon, NVIDIA, AMD)
- ✅ Complete CI/CD pipeline

### Phase 1: REST API & Streaming 🚧 IN PROGRESS
- ✅ Ollama-compatible REST API (POST /api/generate, GET /api/tags, GET /health)
- ✅ Streaming generation (Server-Sent Events)
- ✅ Thread-safe engine orchestration
- ⏳ GPU testing (Fedora + RTX 4090)
- ⏳ Chat completions endpoint
- ⏳ Model management

### Phase 2-5: See [docs/hyperllama_technical_plan.md](docs/hyperllama_technical_plan.md)

## Performance

**Current (M3 Max CPU):**
- Single request: 2108ms latency
- Throughput: 23.71 tokens/sec
- Model: Llama-3.1-8B-Instruct Q4_K (4.58GB)

**Expected (RTX 4090 GPU):**
- Throughput: 200-800 tokens/sec (10-50x improvement)
- Latency: 50-200ms per request

## Documentation

- **[START_HERE.md](START_HERE.md)**: Development start guide
- **[Phase 0 Complete](PHASE_0_COMPLETE.md)**: Initial validation results
- **[Phase 1 Progress](PHASE_1_PROGRESS.md)**: Streaming & REST API implementation
- **[REST API Docs](docs/PHASE_1_REST_API.md)**: API endpoint documentation
- **[Technical Plan](docs/hyperllama_technical_plan.md)**: Complete roadmap

## Contributing

Not yet accepting contributions - we're in early validation phase. Star/watch the repo to follow progress!

## License

Apache 2.0 with LLVM Exception (pending finalization)

## Roadmap

- **Week 1-2**: Technology validation
- **Week 3-6**: MVP with basic inference
- **Week 7-10**: API server + streaming
- **Week 11-14**: Advanced memory management
- **Week 15-18**: Performance optimizations
- **Week 19-20**: Production polish + v1.0 launch

---

**Current Focus**: Phase 1 - Testing REST API and preparing for GPU benchmarks on Fedora + RTX 4090
