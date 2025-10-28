# TODO

_Last Updated: 2025-10-22_

## Completed So Far ✅

- [x] vLLM optimization (29.95x faster than Ollama on concurrent)
- [x] All core Ollama endpoints (/api/generate, /api/chat, /api/ps, /api/show, /api/version)
- [x] Comprehensive testing (19 tests: 8 integration + 3 performance + 8 unit)
- [x] Documentation (TESTING.md, COMPETITIVE_STRATEGY.md, REALISTIC_NEXT_STEPS.md)

**Current version:** 0.0.3 (development)

---

## 0.0.4 - Model Validation (This Week)

**Goal:** Verify vLLama works with popular models (not just opt-125m)

### Test Popular Models
- [ ] **Llama 3.1 8B** - most popular open model
- [ ] **Llama 3.2 1B** - newer, smaller
- [ ] **Llama 3.2 3B** - newer, medium
- [ ] **Qwen 2.5 7B** - already tested 1.5B
- [ ] **Mistral 7B v0.3** - popular for coding/chat

### Documentation
- [ ] Create docs/MODELS.md
  - Compatibility matrix (which models work)
  - Memory requirements per model
  - Performance data (tokens/sec)
  - Known issues/limitations
- [ ] Update README with model support section

**Tag:** v0.0.4 when done

---

## 0.0.5 - Production Polish (Next Week)

**Goal:** Make it production-ready

### Error Handling
- [ ] User-friendly error messages
- [ ] Helpful suggestions (e.g., "Try: vllama pull <model>")
- [ ] Don't leak internal errors
- [ ] Consistent error format

### CLI Improvements
- [ ] Better help text
- [ ] Colored output (errors in red, success in green)
- [ ] Progress indicators for downloads
- [ ] Model preload flag (--preload)

### Monitoring
- [ ] /health endpoint improvements
  - Show loaded models
  - Show GPU status
  - Show memory usage
- [ ] /metrics endpoint (Prometheus format)
  - Request counts
  - Latencies
  - GPU utilization
- [ ] Structured logging (JSON)
  - Request IDs
  - Performance metrics

**Tag:** v0.0.5 when done

---

## 0.0.6 - Performance Documentation (Week 3)

**Goal:** Document the performance advantage

### Benchmarking
- [ ] Benchmark all tested models
  - Sequential performance
  - Concurrent (5, 10, 50 requests)
  - Memory usage per concurrency level
  - GPU utilization
- [ ] Create docs/PERFORMANCE.md
  - Performance vs Ollama comparison table
  - When to use vLLama (production, high throughput)
  - When to use Ollama (hobbyist, macOS)
  - Hardware recommendations

### Update README
- [ ] Performance claims with evidence
- [ ] Link to benchmarks
- [ ] Clear positioning: "Linux + NVIDIA production deployments"

**Tag:** v0.0.6 when done

---

## 0.0.7 - First Production User (Week 4)

**Goal:** Get someone using this in production

### Deployment Guide
- [ ] Docker setup
- [ ] Systemd service file
- [ ] Reverse proxy (nginx/caddy examples)
- [ ] Monitoring setup (Prometheus + Grafana)

### Security
- [ ] Input validation review
- [ ] Rate limiting example
- [ ] Document auth (how to add it)

### Promotion
- [ ] Share on r/LocalLLaMA
- [ ] Share on r/rust
- [ ] Share on Hacker News
- [ ] Position: "vLLM for Ollama users on Linux"

**Success:** 1+ production deployment, real user feedback

**Tag:** v0.0.7 when done

---

## Future (After 0.0.7)

### When to Consider 0.1.0
- ✅ 5+ models tested and working
- ✅ Production deployment guide
- ✅ 1+ real production user
- ✅ Performance fully documented
- ✅ No critical bugs

### What NOT to Do Yet
- ❌ macOS support (see REALISTIC_NEXT_STEPS.md for rationale)
- ❌ Multi-modal (vision)
- ❌ Embeddings (RAG)
- ❌ Quantization
- ❌ Multi-GPU

### Maybe Later (User-Driven)
- [ ] /api/delete endpoint (if users request it)
- [ ] /api/copy endpoint (if users request it)
- [ ] Embeddings (if RAG users appear)
- [ ] Streaming tests (if streaming breaks)

**Strategy:** Stay focused on Linux + NVIDIA production deployments. Let user feedback drive features.
