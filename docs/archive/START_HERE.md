# HyperLlama - Development Start Guide

## Current Status: Phase 0, Week 1 - Technology Validation

We're at the beginning of a 20-week journey to build HyperLlama, a high-performance local LLM inference server that delivers Ollama's simplicity with vLLM-class performance.

## Quick Context

**What we're building**: Local LLM inference server (like Ollama) with 2-5x better performance through state-of-the-art optimizations (continuous batching, FlashAttention, speculative decoding, PagedAttention).

**Tech Stack**:
- **Rust (95%)**: API server, CLI, model management, orchestration
- **Mojo (5%)**: Custom MAX Engine kernels only when needed
- **MAX Engine (Primary)**: Hardware-agnostic inference compilation
- **Fallbacks**: vLLM (GPU), llama.cpp (CPU)

**Target Performance**: 2-3x faster than Ollama on single requests, 3-4x on concurrent workloads.

## Phase 0: Week 1 Tasks (THIS WEEK)

### Critical Path - Must Complete

1. **✅ Set up Rust workspace**
   - Initialize Cargo workspace
   - Set up proper project structure (see Architecture below)
   - Add core dependencies (axum, tokio, clap, serde, sqlx, etc.)
   - Configure CI/CD basics (GitHub Actions)

2. **🎯 MAX Engine Integration Prototype**
   - Set up MAX Engine / Modular development environment
   - Prove MAX can load GGUF models (or convert)
   - Run simple inference (load llama3-8b, generate 1 token)
   - Document the FFI/integration approach (Python bridge? Direct?)

3. **📊 Performance Validation**
   - Benchmark MAX Engine vs llama.cpp on same hardware
   - Single request latency (tokens/sec)
   - Memory usage comparison
   - Document actual numbers (need baseline for decisions)

4. **🚦 GO/NO-GO Decision**
   - If MAX >= 1.5x llama.cpp → Continue with MAX as primary
   - If MAX < 1.5x OR major issues → Pivot to vLLM + llama.cpp hybrid
   - Document decision rationale

## Project Structure (To Create)

```
hyperllama/
├── Cargo.toml              # Workspace root
├── crates/
│   ├── hyperlama-cli/      # CLI binary (clap commands)
│   ├── hyperlama-server/   # API server (axum)
│   ├── hyperlama-core/     # Core inference logic
│   ├── hyperlama-engine/   # Engine abstraction (MAX/vLLM/llama.cpp)
│   └── hyperlama-models/   # Model management, registry, downloads
├── docs/
│   ├── hyperlama_technical_plan.md      # Full 1600-line plan
│   ├── hyperlama_tech_stack_summary.md  # Quick reference
│   └── START_HERE.md                     # This file
├── examples/               # Example code for users
├── tests/                  # Integration tests
├── benches/                # Performance benchmarks
└── .github/
    └── workflows/          # CI/CD
```

## Key Technical Constraints

### Language Requirements
- **Rust nightly**: Needed for some async features
- **Rust 2021 edition**: Modern idioms
- **MSRV**: 1.75+ (for latest async features)

### Core Dependencies (Add These)

```toml
# Web framework
axum = "0.7"
tokio = { version = "1.35", features = ["full"] }
tower = "0.4"

# CLI
clap = { version = "4.4", features = ["derive", "env"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"

# Database
sqlx = { version = "0.7", features = ["runtime-tokio", "sqlite"] }

# HTTP client
reqwest = { version = "0.11", features = ["stream", "json"] }

# Observability
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }

# Error handling
anyhow = "1.0"
thiserror = "1.0"

# Async utilities
futures = "0.3"
async-stream = "0.3"

# Concurrent data structures
dashmap = "5.5"
parking_lot = "0.12"
```

### Hardware Detection Libraries
```toml
sysinfo = "0.30"          # System info, GPU detection
nvml-wrapper = "0.9"      # NVIDIA GPU info (optional)
```

## Critical Questions to Answer This Week

1. **MAX Engine Reality**
   - Does it actually work for GGUF models?
   - Can we call it from Rust efficiently?
   - What's the performance actually like?
   - Are there deal-breaker limitations?

2. **Integration Approach**
   - PyO3 bridge to Python MAX?
   - Direct FFI to MAX Engine?
   - Or run MAX as separate process?

3. **Model Format**
   - Can MAX load GGUF directly?
   - Need conversion to SafeTensors?
   - What about quantized models?

## Architecture Summary

### Engine Abstraction Layer
```rust
// Core trait all engines must implement
pub trait InferenceEngine {
    async fn load_model(&mut self, path: &Path) -> Result<ModelHandle>;
    async fn generate(&self, request: GenerateRequest) -> Result<GenerateResponse>;
    fn supports_hardware(&self, hw: &Hardware) -> bool;
    fn capabilities(&self) -> EngineCapabilities;
}

// Engines to implement
pub struct MaxEngine { /* MAX Engine via FFI */ }
pub struct VllmEngine { /* vLLM via Python */ }
pub struct LlamaCppEngine { /* llama.cpp via FFI */ }

// Orchestrator picks best engine
pub struct EngineOrchestrator {
    engines: Vec<Box<dyn InferenceEngine>>,
    hardware: Hardware,
}
```

### Request Flow
```
User Request
    ↓
CLI/API Layer (clap/axum)
    ↓
Request Queue
    ↓
Continuous Batcher (schedules batches)
    ↓
Engine Orchestrator (picks best engine)
    ↓
MAX Engine | vLLM | llama.cpp
    ↓
Hardware (GPU/CPU)
    ↓
Streaming Response
```

## Success Criteria for Week 1

- [ ] Cargo workspace initialized with proper structure
- [ ] Core dependencies added and compiling
- [ ] MAX Engine successfully loads a small model (e.g., tinyllama)
- [ ] Can generate at least 1 token via MAX Engine
- [ ] Benchmark numbers documented (tokens/sec vs llama.cpp)
- [ ] Decision documented: Continue with MAX or pivot?
- [ ] Basic CI/CD running (cargo test, cargo clippy)

## Reference Documents

- **Full Technical Plan**: `hyperlama_technical_plan.md` (1,600 lines, all details)
- **Tech Stack Summary**: `hyperlama_tech_stack_summary.md` (quick reference)
- **This File**: Context for starting Phase 0

## Development Commands (Once Set Up)

```bash
# Build workspace
cargo build

# Run tests
cargo test

# Run clippy
cargo clippy

# Format code
cargo fmt

# Run CLI (once built)
cargo run --bin hyperlama -- --help

# Run benchmarks
cargo bench

# Watch mode (install cargo-watch)
cargo watch -x test -x clippy
```

## Getting Started Right Now

### Step 1: Initialize Cargo Workspace
```bash
# Create workspace Cargo.toml
# Create crates/ directory structure
# Set up each crate with proper dependencies
```

### Step 2: Validate MAX Engine
```bash
# Install Modular/MAX Engine
# Write minimal Rust program that calls MAX
# Load tinyllama or smallest available model
# Time single token generation
```

### Step 3: Baseline Benchmark
```bash
# Install Ollama (if not present)
# Run same model through Ollama
# Compare tokens/sec and memory usage
# Document findings
```

## Key Decisions Pending

### This Week (Week 1)
- [ ] MAX Engine: Go or No-Go?
- [ ] FFI approach: PyO3, direct FFI, or subprocess?
- [ ] Model format: GGUF support or conversion needed?

### Next Week (Week 2)
- [ ] Database schema for model registry
- [ ] CLI command structure (finalize)
- [ ] Configuration file format (finalize)

## Testing Hardware Available

Document what hardware you have access to:
- [ ] NVIDIA GPU (model: _____, VRAM: _____)
- [ ] AMD GPU (model: _____, VRAM: _____)
- [ ] Apple Silicon (model: _____, RAM: _____)
- [ ] CPU-only (model: _____, RAM: _____)

## Notes & Learnings

(Document discoveries as you go)

### MAX Engine Notes
- Installation:
- Performance:
- Limitations found:

### Integration Notes
- FFI approach decided:
- Build complexity:
- Runtime overhead:

### Benchmark Results
- llama.cpp baseline:
- MAX Engine:
- Decision:

---

## Quick Start Command

```bash
# Start here - initialize the Rust workspace
cargo init --name hyperlama-cli crates/hyperlama-cli
cargo init --lib crates/hyperlama-core
cargo init --lib crates/hyperlama-engine
cargo init --lib crates/hyperlama-models
cargo init --lib crates/hyperlama-server

# Then create workspace Cargo.toml to tie them together
```

**Next Action**: Ask Claude Code to initialize the Cargo workspace with proper structure based on this document.
