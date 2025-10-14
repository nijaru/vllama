# HyperLlama

> High-performance local LLM inference server with exceptional developer experience

**Status**: 🚧 Phase 0 (Week 1) - Technology Validation

## What is HyperLlama?

HyperLlama is a local LLM inference server that delivers:
- **Simple developer experience**: One-command installation, auto-downloads, intuitive CLI
- **State-of-the-art performance**: Continuous batching, FlashAttention, speculative decoding, prefix caching
- **Universal hardware support**: NVIDIA, AMD, Apple Silicon, Intel, CPU with automatic optimization

### Performance Goals
- **Fast single requests**: 100+ tokens/sec on consumer GPUs
- **High throughput**: Efficient concurrent request handling
- **Memory efficient**: 50-60% reduction via advanced caching techniques

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
