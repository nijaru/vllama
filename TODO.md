# HyperLlama Development TODO

## Phase 0: Week 1 - Technology Validation

### 1. Rust Workspace Setup
- [ ] Create workspace `Cargo.toml`
- [ ] Initialize crate: `hyperlama-cli` (binary)
- [ ] Initialize crate: `hyperlama-core` (library)
- [ ] Initialize crate: `hyperlama-engine` (library)
- [ ] Initialize crate: `hyperlama-models` (library)
- [ ] Initialize crate: `hyperlama-server` (library)
- [ ] Add core dependencies to each crate
- [ ] Verify `cargo build` works
- [ ] Set up `cargo clippy` and `cargo fmt`
- [ ] Create basic CI/CD workflow (GitHub Actions)

### 2. MAX Engine Integration Prototype
- [ ] Install Modular/MAX Engine locally
- [ ] Research MAX Engine API (Python? FFI? Subprocess?)
- [ ] Write minimal Rust program that calls MAX Engine
- [ ] Load smallest available model (TinyLlama or similar)
- [ ] Generate single token via MAX Engine
- [ ] Document integration approach
- [ ] Measure FFI/bridge overhead

### 3. Performance Baseline Benchmarks
- [ ] Install Ollama (if not present)
- [ ] Install llama.cpp (if not present)
- [ ] Select benchmark model (llama3-8b-q4_K_M recommended)
- [ ] Benchmark Ollama: single request tokens/sec
- [ ] Benchmark llama.cpp: single request tokens/sec
- [ ] Benchmark MAX Engine: single request tokens/sec
- [ ] Measure memory usage for each
- [ ] Document hardware specs used
- [ ] Create benchmark script for repeatability

### 4. GO/NO-GO Decision
- [ ] Analyze MAX Engine performance vs baseline
- [ ] Evaluate MAX Engine integration complexity
- [ ] Assess MAX Engine documentation quality
- [ ] Check hardware compatibility (test on available hardware)
- [ ] Document decision: Continue with MAX or pivot?
- [ ] Update technical plan based on findings

## Phase 0: Week 2 - Project Foundation

### 1. Project Structure Finalization
- [ ] Finalize database schema (SQLite)
- [ ] Create migration files
- [ ] Set up configuration file structure
- [ ] Define core data structures (models, requests, responses)

### 2. Development Environment
- [ ] Document setup instructions
- [ ] Create development scripts
- [ ] Set up logging/tracing framework
- [ ] Configure testing framework

### 3. Basic CLI Skeleton
- [ ] Implement `hyperlama --version`
- [ ] Implement `hyperlama --help`
- [ ] Stub out all major commands (run, pull, list, serve, etc.)
- [ ] Add command-line argument parsing

## Questions to Answer

### MAX Engine
- [ ] Does MAX Engine support GGUF format directly?
- [ ] What quantization methods does MAX support?
- [ ] Can we call MAX from Rust efficiently?
- [ ] What hardware does MAX actually work on today?
- [ ] Are there any dealbreaker limitations?

### Architecture
- [ ] Should we use PyO3 for MAX Engine bridge?
- [ ] Or subprocess communication?
- [ ] Or wait for native Rust MAX bindings?
- [ ] How to structure the engine abstraction layer?

### Performance
- [ ] Can we realistically achieve 2x over Ollama?
- [ ] What's the actual overhead of our architecture?
- [ ] Which optimizations should we prioritize?

## Decisions Made

(Document key decisions as they're made)

### Week 1
- Date: ___
- Decision: ___
- Rationale: ___

## Blockers & Risks

(Document anything blocking progress)

### Current Blockers
- None yet

### Identified Risks
- MAX Engine may not meet performance expectations
- FFI overhead may be significant
- Hardware compatibility issues

## Notes

(Quick notes and learnings)
