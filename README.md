# HyperLlama

> High-performance local LLM inference server - Ollama's simplicity, vLLM's speed

**Status**: 🚧 Phase 0 (Week 1) - Technology Validation

## What is HyperLlama?

HyperLlama is a next-generation local LLM inference server that combines:
- **Ollama's exceptional developer experience** (simple CLI, auto-downloads, one command setup)
- **State-of-the-art performance optimizations** (continuous batching, FlashAttention, speculative decoding)
- **Universal hardware support** (NVIDIA, AMD, Apple Silicon, Intel, CPU)

### Target Performance
- **2-3x faster** than Ollama on single requests
- **3-4x faster** on concurrent workloads
- **50-60% less memory** usage via advanced caching

## Quick Start (Coming Soon)

```bash
# Installation (not yet available)
curl -fsSL https://hyperlama.dev/install.sh | sh

# Run a model
hyperlama run llama3

# Start API server
hyperlama serve
```

## Architecture

**Core Technologies**:
- **Rust (95%)**: API server, CLI, orchestration
- **Modular MAX Engine**: Hardware-agnostic inference compilation
- **Fallbacks**: vLLM (GPU), llama.cpp (CPU)

**Key Optimizations**:
- Continuous batching (dynamic request scheduling)
- FlashAttention-2 (memory-efficient attention)
- PagedAttention (KV cache paging)
- Speculative decoding (2-3x latency reduction)
- Prefix caching (10-100x speedup for common prompts)

## Development Status

### Phase 0: Technology Validation (Weeks 1-2)
- [ ] Rust workspace initialization
- [ ] MAX Engine integration prototype
- [ ] Performance baseline benchmarks
- [ ] Go/No-Go decision on MAX Engine

### Phase 1: MVP (Weeks 3-6)
- [ ] Model loading and inference
- [ ] Basic CLI commands
- [ ] Quantization support

### Phase 2-5: See `docs/hyperlama_technical_plan.md`

## Documentation

- **[START_HERE.md](START_HERE.md)**: Development start guide
- **[Technical Plan](hyperlama_technical_plan.md)**: Complete 20-week roadmap (1,600 lines)
- **[Tech Stack Summary](hyperlama_tech_stack_summary.md)**: Quick reference

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

**Current Focus**: Phase 0, Week 1 - Validating MAX Engine performance claims
