# vllama Project Instructions

## What is vllama?

**Ollama's DX with vLLM's performance**

vllama is an Ollama-compatible LLM inference server optimized for Linux + NVIDIA GPUs. We use vLLM for GPU inference and provide an Ollama-compatible API with better developer experience than raw vLLM.

**Target users:** Production deployments on Linux with NVIDIA GPUs

**NOT targeting:** macOS/hobbyists (Ollama already great there), researchers (use raw vLLM)

## Project Status

**Current:** 0.0.5 (core tested and working!)
- ✅ Core Ollama API endpoints working
- ✅ 29.95x faster than Ollama on concurrent requests (validated benchmarks)
- ✅ Comprehensive testing (22 tests passing: 14 unit + 8 integration)
- ✅ Critical bugs found and FIXED (timeout, orphaned subprocess)
- ✅ Model validation (Qwen 2.5: 0.5B, 1.5B, 7B; Mistral 7B)
- ✅ Modern CLI UX, error handling, monitoring
- ✅ Performance documentation (docs/PERFORMANCE.md)
- ✅ Server cleanup verified (processes, GPU memory, ports)
- ❌ Deployment configs UNTESTED (moved to `deployment-configs` branch)
- ❌ NO real production users yet
- ❌ NOT validated in real use

**Reality:** Core server works, deployment needs validation. Stay in 0.0.x until production-proven.

**Testing Status (Latest):**
- Tested server locally with Qwen/Qwen2.5-0.5B-Instruct
- Found 2 critical bugs through testing (not code review!)
  - Bug 1: Startup timeout too short (60s → 120s fixed)
  - Bug 2: Orphaned vLLM subprocess on error (process group kill fixed)
- All 22 tests passing (14 unit + 8 integration)
- Cleanup verified: processes killed, GPU memory released, ports freed
- See TESTING_STATUS.md for full details

**Deployment Status:**
- Deployment configs moved to `deployment-configs` branch (untested)
- Main branch contains only tested code
- Docker, systemd, nginx, monitoring configs need validation before merge

**Next:** Deployment validation & real user feedback
- ✅ 0.0.5: Core tested, critical bugs fixed
- 0.0.6: Test deployment configs (Docker, systemd, etc.)
- 0.0.x: Stay here until proven in real use
- Don't jump to 0.1.0 until production-proven with real users

## Architecture

```
┌─────────┐      ┌────────────┐      ┌──────────────┐      ┌─────┐
│ Client  │ ───> │ vllama     │ ───> │ vLLM OpenAI  │ ───> │ GPU │
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

### Versioning (Stay in 0.0.x for a LONG time)
- 0.0.x = experimental, breaking changes expected
- Tag each small milestone (0.0.5, 0.0.6, 0.0.7...)
- Incremental improvements, not big jumps
- **Don't jump to 0.1.0 until:**
  - Multiple real users (not just me)
  - Weeks/months of real usage
  - Bugs found and fixed through actual use
  - Clear evidence of value (not just benchmarks)
  - Community validation
- **Expect to stay in 0.0.x for months**

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

## Model Guidelines

**Tested & Working:**
- Qwen 2.5 (0.5B, 1.5B, 7B) - Best for testing, open access
- Mistral 7B v0.3 - Great for coding/chat
- See docs/MODELS.md for full compatibility matrix

**Critical GPU Memory Rules:**
- **7B models:** MUST use `--gpu-memory-utilization 0.9` (not 0.5)
- **Small models (0.5-1.5B):** Can use 0.5 GPU utilization
- **24GB GPU (RTX 4090):** Can run any 7B model at 90% util
- **Failure symptom:** "No available memory for cache blocks" = need higher GPU util

**Model Recommendations:**
- Quick testing: Qwen 2.5 0.5B or 1.5B (fast, small)
- Production: Qwen 2.5 7B or Mistral 7B (quality)
- Auth required: Llama models (need HuggingFace token)

## Common Tasks

### Testing New Model
```bash
# Start server (7B models need 0.9 GPU utilization!)
cargo run --release -- serve --model <model-name> --gpu-memory-utilization 0.9

# Test generation
curl -X POST localhost:11434/api/generate \
  -d '{"model":"<model>","prompt":"What is 2+2?","stream":false}'

# Document in docs/MODELS.md with:
# - Model size, GPU util, load time, VRAM usage
# - KV cache size and max concurrency
# - Any special requirements or issues
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
