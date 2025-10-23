# TODO

_Last Updated: 2025-10-22_

## High Priority

- [x] **Test vLLM optimizations** (DONE ✅)
  - [x] Add optimization flags to serve.rs
  - [x] Start server with optimized config
  - [x] Test with concurrent requests (5, 10, 50)
  - [x] Compare performance vs Ollama
  - Result: **29.95x faster than Ollama!** (crushed the 2x target)

## In Progress

- [ ] **Fix missing endpoints** (Day 2)
  - [ ] Fix /api/ps - return actual model data (not empty array)
  - [ ] Improve /api/show - return useful metadata
  - [ ] Add /api/version - basic version info

## Backlog

- [ ] **Comprehensive benchmarking** (Day 3)
  - [ ] Test with 1.5B, 3B, 8B models
  - [ ] Document performance improvements
  - [ ] Update README and docs

- [ ] **macOS support** (Phase 2)
  - [ ] Add llama.cpp for Apple Silicon
  - [ ] Platform detection
  - [ ] GGUF model handling

- [ ] **Future enhancements**
  - [ ] Embeddings endpoint (needs separate model/RAG features)
  - [ ] Model management (/api/copy, /api/delete)
  - [ ] Multi-GPU tensor parallelism
