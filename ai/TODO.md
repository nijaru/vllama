# TODO

_Last Updated: 2025-10-22_

## Completed 0.0.x Development ✅

- [x] vLLM optimization (29.95x faster than Ollama on concurrent)
- [x] All core Ollama endpoints (/api/generate, /api/chat, /api/ps, /api/show, /api/version)
- [x] Comprehensive testing (19 tests: 8 integration + 3 performance + 8 unit)
- [x] Documentation (TESTING.md, COMPETITIVE_STRATEGY.md)

---

## 0.1.0 - Linux Production Readiness (Target: 4 weeks)

### Week 1: Model Support & Validation
- [ ] **Test popular models**
  - [ ] Llama 3.1 8B (most popular)
  - [ ] Llama 3.2 1B, 3B
  - [ ] Qwen 2.5 1.5B, 7B
  - [ ] Mistral 7B v0.3
  - [ ] DeepSeek Coder 6.7B
- [ ] **Document compatibility matrix**
  - [ ] Which models work
  - [ ] Performance benchmarks
  - [ ] Memory requirements
  - [ ] Known issues

### Week 2: Missing Features
- [ ] **/api/delete endpoint**
  - [ ] Delete model from disk
  - [ ] Update loaded_models state
  - [ ] Tests
- [ ] **/api/copy endpoint**
  - [ ] Copy model with new name
  - [ ] Update model registry
  - [ ] Tests
- [ ] **Better error messages**
  - [ ] User-friendly error responses
  - [ ] Helpful suggestions (e.g., "Try vllama pull <model>")
  - [ ] Error codes

### Week 3: Performance Validation
- [ ] **Benchmark all supported models**
  - [ ] Sequential performance
  - [ ] Concurrent performance (5, 10, 50 requests)
  - [ ] Memory usage
  - [ ] GPU utilization
- [ ] **Document vs Ollama**
  - [ ] Performance comparison table
  - [ ] When to use vLLama vs Ollama
  - [ ] Create docs/PERFORMANCE.md

### Week 4: User Experience
- [ ] **CLI improvements**
  - [ ] Better help messages
  - [ ] Model preloading flag (--preload)
  - [ ] Startup progress indicators
  - [ ] Colored output
- [ ] **Health monitoring**
  - [ ] /health includes model status
  - [ ] /health includes GPU status
  - [ ] /metrics endpoint (Prometheus format)

---

## 0.2.0 - Advanced Features (Target: 4 weeks)

### Embeddings (RAG Support)
- [ ] /api/embeddings endpoint
- [ ] vLLM embeddings model support
- [ ] Test with sentence-transformers models
- [ ] Documentation

### Multi-Modal (Vision)
- [ ] Vision model support (LLaVA, Qwen-VL)
- [ ] Image input handling (/api/generate with images)
- [ ] Test with popular vision models
- [ ] Documentation

### Observability
- [ ] Prometheus metrics endpoint
- [ ] Request tracing (OpenTelemetry)
- [ ] GPU utilization metrics
- [ ] Dashboard examples (Grafana)

---

## 0.3.0 - macOS Parity (Target: 5 weeks)

### llama.cpp Integration
- [ ] **Research Rust bindings**
  - [ ] Evaluate llama-cpp-rs
  - [ ] Test Metal backend
  - [ ] Benchmark raw performance
- [ ] **Implement llama.cpp engine**
  - [ ] LlamaCppEngine struct
  - [ ] GGUF model loading
  - [ ] Metal acceleration
  - [ ] Unified InferenceEngine trait
- [ ] **Platform detection**
  - [ ] Auto-detect macOS + Apple Silicon
  - [ ] Switch engine at runtime
  - [ ] No user configuration needed

### GGUF Model Management
- [ ] Download GGUF models from HuggingFace
- [ ] Model quantization selection (Q4_K_M, Q5_K_M, etc.)
- [ ] Convert safetensors → GGUF (optional)

### Testing & Benchmarking
- [ ] Test on M1/M2/M3 hardware
- [ ] Benchmark vs Ollama
- [ ] Target: match or beat Ollama
- [ ] Document results

---

## 0.4.0 - macOS Dominance (Target: 6 weeks)

### Performance Optimizations
- [ ] **Model Loading (Target: 2x faster)**
  - [ ] Memory-mapped file loading
  - [ ] Parallel GGUF shard loading
  - [ ] Lazy weight initialization
- [ ] **Prompt Processing (Target: 5-10x faster)**
  - [ ] Zero-copy prompt encoding
  - [ ] Batch prompt processing
  - [ ] Pre-allocated buffers
- [ ] **Token Generation (Target: 15% faster)**
  - [ ] Metal kernel optimization
  - [ ] KV cache optimization
  - [ ] Speculative decoding (if supported)
- [ ] **Concurrent Requests (Target: 50x+ faster)**
  - [ ] Continuous batching (like vLLM)
  - [ ] Metal multi-stream support
  - [ ] Optimized scheduling

---

## Backlog / Future

### Quantization
- [ ] Research vLLM quantization support
- [ ] GPTQ/AWQ if feasible
- [ ] Document trade-offs

### Multi-GPU
- [ ] Tensor parallelism
- [ ] Pipeline parallelism
- [ ] Test on 2x, 4x GPU setups

### Streaming Tests
- [ ] Integration tests for streaming endpoints
- [ ] SSE parsing validation
- [ ] Error handling in streams

### CI/CD
- [ ] GitHub Actions workflow
- [ ] Automated testing on PR
- [ ] Performance regression detection
- [ ] Release automation
