# HyperLlama: Quick Tech Stack Reference

## Core Technologies

### Languages
- **Rust (95%)**: API server, CLI, model management, all glue code
- **Mojo (5%)**: Only for custom MAX Engine kernels when needed

### Inference Engines (Priority Order)
1. **Modular MAX Engine** (Primary) - Hardware-agnostic, CUDA-free
2. **vLLM** (GPU Fallback) - Mature NVIDIA/AMD support
3. **llama.cpp** (CPU Fallback) - CPU-only systems

### Web Framework
- **Axum** - Fast, async Rust web framework
- **Tokio** - Async runtime for concurrency

### Key Libraries
| Library | Purpose |
|---------|---------|
| `clap` | CLI parsing |
| `serde` | JSON/TOML serialization |
| `sqlx` | SQLite for model metadata |
| `reqwest` | Model downloads |
| `tracing` | Structured logging |

## State-of-the-Art Optimizations

### Must-Have (Phase 1-2)
- ✅ FlashAttention-2 (2-3x faster attention)
- ✅ Continuous Batching (3-4x throughput)
- ✅ Quantization (Q4_K_M, Q8_0)

### High-Impact (Phase 3-4)
- ✅ PagedAttention (60% memory reduction)
- ✅ Prefix Caching (10-100x speedup for common prompts)
- ✅ Speculative Decoding (2-3x latency reduction)
- ✅ Tensor Parallelism (multi-GPU support)

### Nice-to-Have (Post-v1.0)
- Chunked Prefill (long contexts)
- KV Cache Quantization
- Fused Kernels (via MAX)

## Performance Targets

| Scenario | vs Ollama | How |
|----------|-----------|-----|
| Single request | 2-3x faster | MAX Engine + FlashAttention |
| 8 concurrent | 3-4x faster | Continuous batching |
| Long context | 2-3x faster | Chunked prefill |
| Memory usage | 50-60% less | PagedAttention + Q8 KV |

## Hardware Support

| Platform | Engine | Status |
|----------|--------|--------|
| NVIDIA GPU | MAX/vLLM | ✅ Excellent |
| AMD GPU | MAX | ⚠️ Good (validate) |
| Apple Silicon | MAX/Metal | ⚠️ Preliminary |
| Intel GPU | MAX | ⚠️ Experimental |
| CPU | MAX/llama.cpp | ✅ Good |

## Developer Experience Priorities

1. **Installation**: Single command, <2 minutes
2. **CLI**: 100% Ollama-compatible commands
3. **API**: OpenAI-compatible REST + streaming
4. **Models**: Auto-download, smart defaults
5. **Config**: Optional, sensible defaults
6. **Errors**: Clear, actionable messages

## Key Decisions

### Why Rust over Python?
- Zero-cost abstractions
- Memory safety
- Excellent async
- Single binary distribution
- Better for systems programming

### Why MAX Engine over pure vLLM?
- Universal hardware support
- No CUDA/ROCm dependencies
- Automatic optimization
- Future-proof architecture

### Why Hybrid Approach?
- Lower risk (fallbacks to proven engines)
- Can pivot if MAX underperforms
- Best-in-class for each hardware

### Why Ollama Compatibility?
- Zero learning curve
- Easy migration
- Familiar mental model
- Can run side-by-side

## Timeline

- **Weeks 1-2**: Validation & setup
- **Weeks 3-6**: MVP (single model inference)
- **Weeks 7-10**: API server + streaming
- **Weeks 11-14**: Advanced memory management
- **Weeks 15-18**: Performance optimizations
- **Weeks 19-20**: Production polish & launch

## Critical Risks

1. **MAX Engine limitations** → Validate in Week 1-2, fallback ready
2. **Performance targets miss** → Hybrid approach, realistic expectations
3. **Hardware compatibility** → Extensive testing matrix
4. **Mojo learning curve** → Minimize usage, hire expert

## Success Criteria

- ✅ 2x faster than Ollama (minimum)
- ✅ 100% API compatibility
- ✅ Works on 4+ hardware platforms
- ✅ 10K+ users in 6 months
- ✅ <2 minute installation

## Next Steps

1. **Week 1**: Prototype MAX Engine integration, benchmark
2. **Week 2**: Decide: proceed with MAX or pivot to pure vLLM
3. **Week 3**: Start MVP development
4. Launch v1.0 in 20 weeks

---

**Full Plan**: See `hyperlama_technical_plan.md` (1,600 lines)
