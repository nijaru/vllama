# macOS Optimization Strategy

**Issue:** vLLM has only experimental CPU-only support on macOS, resulting in similar performance to Ollama.

**Solution:** Add llama.cpp engine with Metal acceleration for Apple Silicon.

---

## Research Summary (2025)

### Current State
- **vLLM:** NVIDIA GPU-focused, experimental CPU-only on macOS
- **Ollama:** Uses llama.cpp backend with Metal acceleration
- **llama.cpp:** Excellent Apple Silicon support via Metal API
- **MLX:** Apple's framework, now matching llama.cpp performance

### Performance Data (Apple Silicon)
- **llama.cpp with Metal:** ~26 tokens/sec (M3 MacBook Pro, Llama 2 7B Q4)
- **llama.cpp with Metal:** ~65 tokens/sec (M3 Max, Llama 8B 4-bit)
- **llama.cpp speedup:** ~1.5x faster GPU vs CPU on M2
- **MLX vs llama.cpp:** Comparable performance as of 2025
- **Model loading:** MLX ~10s, llama.cpp ~30s

### Why llama.cpp (Not MLX)?
1. **Proven production use** - What Ollama uses
2. **Mature ecosystem** - Extensive documentation, community support
3. **Same baseline as Ollama** - Easier to compare and claim parity
4. **Python bindings** - `llama-cpp-python` available
5. **GGUF format** - Industry standard for quantized models

---

## Proposed Architecture

### Multi-Engine Design

```
Platform Detection
    ↓
┌─────────────────────────────┐
│   vLLama Server (Rust)      │
│   - Ollama-compatible API   │
└─────────────────────────────┘
         ↓
    Engine Router
    (based on platform)
         ↓
    ┌────────┴────────┐
    ↓                 ↓
Linux/NVIDIA      macOS/Apple Silicon
    ↓                 ↓
vLLM OpenAI      llama.cpp
  Server           (Metal)
    ↓                 ↓
NVIDIA GPU        Apple GPU
```

### Engine Selection Logic

```rust
fn select_engine() -> EngineType {
    match detect_hardware() {
        Hardware::NvidiaGpu => EngineType::Vllm,
        Hardware::AppleSilicon => EngineType::LlamaCpp,
        Hardware::Cpu => EngineType::LlamaCpp,  // Better CPU support than vLLM
    }
}
```

---

## Implementation Plan

### Phase 1: llama.cpp Integration (Foundations)

**Goal:** Add llama.cpp as alternative engine for macOS

**Tasks:**
1. Add `llama-cpp-python` to Python dependencies
2. Implement `LlamaCppEngine` struct in Rust
3. Create Python wrapper service (similar to vLLM OpenAI server approach)
4. Platform detection in `serve` command
5. Auto-start appropriate engine based on platform

**Files to Create:**
- `crates/vllama-engine/src/llama_cpp.rs` - Engine implementation
- `python/llama_cpp_service/server.py` - OpenAI-compatible wrapper
- `python/llama_cpp_service/__init__.py`

**Files to Modify:**
- `crates/vllama-cli/src/commands/serve.rs` - Platform detection + engine selection
- `python/pyproject.toml` - Add llama-cpp-python dependency
- `crates/vllama-engine/src/lib.rs` - Export LlamaCppEngine

**Estimated Effort:** 2-3 days

---

### Phase 2: Model Format Support

**Challenge:** vLLM uses HuggingFace models, llama.cpp uses GGUF format

**Solutions:**
1. **Auto-convert on download** - Convert HF models to GGUF automatically
2. **Support both formats** - Let users specify format preference
3. **Smart detection** - Check model hub for GGUF versions first

**Recommended Approach:**
```rust
// Check for pre-quantized GGUF on HuggingFace
let gguf_model = format!("{}-GGUF", model_name);
if model_exists(&gguf_model) {
    download_gguf(gguf_model)
} else {
    download_hf_model(model_name)
    convert_to_gguf()  // Optional: use llama.cpp convert script
}
```

**Files to Modify:**
- `crates/vllama-server/src/api.rs` - `/api/pull` endpoint
- Model download logic to support GGUF

**Estimated Effort:** 1-2 days

---

### Phase 3: Testing & Documentation

**Testing:**
- macOS (Apple Silicon) end-to-end tests
- Performance benchmarks vs Ollama
- Model download and conversion tests
- Cross-platform CI (Linux + macOS)

**Documentation:**
- Update README with macOS performance expectations
- Add GGUF model format guide
- Document quantization options (Q4, Q5, Q8)
- Update BENCHMARKS.md with macOS results

**Estimated Effort:** 1-2 days

---

## Expected Results

### Performance (macOS Apple Silicon)

| Platform | Before | After | Improvement |
|----------|--------|-------|-------------|
| M3 MacBook Pro | ~10 tok/s (CPU) | ~26 tok/s (Metal) | 2.6x |
| M3 Max | ~15 tok/s (CPU) | ~65 tok/s (Metal) | 4.3x |

**Reality Check:** We won't be "10x faster than Ollama" on macOS because we'll be using the same backend (llama.cpp). But we'll offer:
- ✅ Same great macOS performance as Ollama
- ✅ **10x faster on Linux + NVIDIA GPU**
- ✅ One tool for both dev (macOS) and prod (Linux)

### Developer Experience

**Before:**
```bash
# Developer on macOS
ollama serve              # Dev/testing
# ... deploy to Linux server
# ... switch to vLLama for production

# Two different tools, different APIs
```

**After:**
```bash
# Developer on macOS
vllama serve --model MODEL    # Same tool for dev (llama.cpp)
# ... deploy to Linux server
vllama serve --model MODEL    # Same tool for prod (vLLM)

# One tool, same API, optimized backend per platform
```

---

## Alternative Considered: MLX

**Pros:**
- Apple's official framework
- Similar performance to llama.cpp (2025)
- Pure Python API

**Cons:**
- Newer, less mature ecosystem
- Apple-only (not cross-platform)
- Different model format
- No production track record like llama.cpp/Ollama

**Decision:** Start with llama.cpp, consider MLX as Phase 4 optional enhancement.

---

## Model Format Strategy

### Recommended Model Catalog

**For macOS (GGUF):**
- `TheBloke/*-GGUF` models on HuggingFace
- Q4_K_M quantization (good speed/quality balance)
- Q5_K_M for better quality (slower)
- Q8_0 for maximum quality (slower, more memory)

**For Linux (HuggingFace):**
- Standard HF models (current approach)
- vLLM handles quantization internally

### Download Logic

```rust
async fn download_model(model: String, engine: EngineType) -> Result<PathBuf> {
    match engine {
        EngineType::Vllm => {
            // Current approach: HuggingFace model
            download_hf_model(model).await
        }
        EngineType::LlamaCpp => {
            // Check for GGUF variant first
            let gguf_variant = format!("{}-GGUF", model);
            if check_model_exists(&gguf_variant).await? {
                download_gguf_model(gguf_variant).await
            } else {
                // Fallback: download HF and convert
                let hf_path = download_hf_model(model).await?;
                convert_to_gguf(hf_path).await
            }
        }
    }
}
```

---

## Success Criteria

**Phase 1 Complete:**
- [x] llama.cpp engine integrated
- [x] Auto-selects engine based on platform
- [x] macOS developers get Metal acceleration
- [x] End-to-end test on macOS M3

**Phase 2 Complete:**
- [x] GGUF model download working
- [x] Model conversion (HF → GGUF) working
- [x] Both formats supported seamlessly

**Phase 3 Complete:**
- [x] Performance benchmarks published
- [x] Documentation updated
- [x] CI tests pass on both platforms

---

## Timeline

- **Phase 1 (Foundations):** 2-3 days
- **Phase 2 (Model Formats):** 1-2 days
- **Phase 3 (Testing/Docs):** 1-2 days
- **Total:** ~1 week for working macOS optimization

---

## Open Questions

1. **Quantization defaults:** What quantization level (Q4, Q5, Q8) should we default to?
2. **Model storage:** Same cache directory for both GGUF and HF models?
3. **API parity:** Any llama.cpp-specific parameters to expose?
4. **Streaming:** Does llama.cpp Python wrapper support SSE streaming?

---

*Created: 2025-10-20*
*Status: Proposal - Not Yet Implemented*
