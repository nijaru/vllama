# vLLama Project Instructions

## What is vLLama?

**Ollama's DX with vLLM's performance**

vLLama is an Ollama-compatible LLM inference server optimized for Linux + NVIDIA GPUs. We use vLLM for GPU inference and provide an Ollama-compatible API with better developer experience than raw vLLM.

**Target users:** Production deployments on Linux with NVIDIA GPUs

**NOT targeting:** macOS/hobbyists (Ollama already great there), researchers (use raw vLLM)

## Project Status

**Current:** 0.0.3 (development)
- ✅ Core Ollama API endpoints working
- ✅ 29.95x faster than Ollama on concurrent requests
- ✅ Comprehensive testing (19 tests)
- ⚠️ Only tested with tiny models (opt-125m, Qwen 1.5B)
- ❌ No production users yet

**Next:** Stay in 0.0.x until production-ready
- 0.0.4: Model validation (Llama 3.x, Qwen, Mistral)
- 0.0.5: Production polish (errors, CLI, monitoring)
- 0.0.6: Performance docs (benchmarks)
- 0.0.7: First production user

## Architecture

```
┌─────────┐      ┌────────────┐      ┌──────────────┐      ┌─────┐
│ Client  │ ───> │ vLLama     │ ───> │ vLLM OpenAI  │ ───> │ GPU │
│         │      │ (Rust)     │      │ Server       │      │     │
└─────────┘      └────────────┘      └──────────────┘      └─────┘
                  Ollama API          (Python/uv)
                  Translation
```

**Stack:**
- Rust CLI + server (Axum)
- vLLM for GPU inference (via uv)
- Ollama-compatible API
- OpenAI API passthrough

## Key Decisions (ai/DECISIONS.md)

1. **Linux-only focus** - vLLM beats llama.cpp on NVIDIA GPUs
2. **vLLM over MAX** - More mature, better concurrency (512 vs 248)
3. **Skip macOS for now** - Would need llama.cpp, modest gains, opportunity cost too high
4. **Ollama API compatibility** - Drop-in replacement for Ollama users
5. **Stay in 0.0.x** - Don't rush to 0.1.0 until production-ready

## Development Guidelines

### Testing
- All API changes need integration tests
- Performance changes need benchmark validation
- Run tests before committing: `cargo test`
- Integration tests: `cargo test --test api_tests -- --ignored`

### Performance Claims
- Never claim "Nx faster" without benchmarks
- Always specify: model, hardware, workload type
- Document in ai/research/ with methodology
- Update ai/STATUS.md with results

### Versioning (Stay in 0.0.x)
- 0.0.x = development, breaking changes OK
- Tag each milestone (0.0.4, 0.0.5, etc.)
- Don't jump to 0.1.0 until:
  - 5+ popular models validated
  - 1+ production user
  - Performance fully documented
  - No critical bugs

### Code Style
- Follow existing patterns in codebase
- Rust: clippy warnings = errors
- Error messages: user-friendly, not technical
- Logging: structured (use tracing)
- Comments: explain WHY, not WHAT

## What NOT to Do

**Don't:**
- ❌ Add macOS support yet (see ai/REALISTIC_NEXT_STEPS.md)
- ❌ Add complex features (embeddings, multi-modal, quantization)
- ❌ Switch to MAX Engine (vLLM is the right choice)
- ❌ Optimize for every use case (focus on production Linux)
- ❌ Jump to 0.1.0 prematurely
- ❌ Make performance claims without evidence

**Do:**
- ✅ Focus on Linux + NVIDIA production deployments
- ✅ Validate popular models (Llama 3.x, Qwen, Mistral)
- ✅ Make DX great (errors, CLI, monitoring)
- ✅ Document everything (benchmarks, compatibility)
- ✅ Get real users and feedback
- ✅ Stay in 0.0.x until proven

## Project Structure

```
vllama/
├── crates/
│   ├── vllama-cli/       # CLI binary (serve, pull, etc.)
│   ├── vllama-server/    # Axum server, Ollama API
│   ├── vllama-engine/    # vLLM wrapper
│   ├── vllama-core/      # Shared types, OpenAI client
│   └── vllama-models/    # Model registry
├── ai/                   # AI agent context
│   ├── STATUS.md         # Current state (read FIRST)
│   ├── TODO.md           # Tasks and roadmap
│   ├── DECISIONS.md      # Architectural decisions
│   ├── REALISTIC_NEXT_STEPS.md  # Focused strategy
│   └── research/         # Research docs
├── docs/                 # User documentation
│   ├── BENCHMARKS.md
│   ├── TESTING.md
│   └── FEDORA_SETUP.md
└── CLAUDE.md            # This file
```

## Common Tasks

### Testing New Model
```bash
# Start server
cargo run --release -- serve --model <model-name>

# Test generation
curl -X POST localhost:11434/api/generate \
  -d '{"model":"<model>","prompt":"Hello","stream":false}'

# Document in docs/MODELS.md
```

### Adding Endpoint
1. Add handler to `crates/vllama-server/src/api.rs`
2. Add route to `crates/vllama-server/src/server.rs`
3. Add integration test to `crates/vllama-server/tests/api_tests.rs`
4. Update docs/TESTING.md

### Benchmarking
```bash
# Use existing script
./test_concurrent.sh

# Document in ai/research/
# Update ai/STATUS.md with results
```

## Quick Reference

**Key files to check:**
- `ai/STATUS.md` - Current state, blockers
- `ai/TODO.md` - Next tasks
- `ai/REALISTIC_NEXT_STEPS.md` - Strategy

**Performance baseline:**
- 29.95x faster than Ollama (concurrent, facebook/opt-125m)
- 4.4x faster (sequential, Qwen 1.5B)
- RTX 4090, i9-13900KF, 32GB DDR5

**Target positioning:**
- "Ollama's DX with vLLM's performance"
- "The fastest LLM server for Linux production"
- NOT: "Works everywhere" or "Fastest on all platforms"

## Questions?

- Check ai/STATUS.md first
- Check ai/DECISIONS.md for rationale
- See ai/REALISTIC_NEXT_STEPS.md for strategy
- Existing research in ai/research/

## External References

- vLLM docs: https://docs.vllm.ai/
- Ollama API: https://github.com/ollama/ollama/blob/main/docs/api.md
- Agent contexts standard: github.com/nijaru/agent-contexts
