# HyperLlama - Next Steps

## Phase 1 Complete - Ready to Commit ✅

All Phase 1 code has been written. Here's what to do next:

## 1. Test Compilation

```bash
cd /Users/nick/github/nijaru/hyperllama

# Build the project
cargo build --release

# If there are compilation errors, fix them
# Most likely issues:
# - Missing use statements
# - Type mismatches in api.rs
# - Async/await issues
```

## 2. Commit Phase 1 Changes

```bash
# Check what's changed
git status

# Add all new and modified files
git add crates/hyperllama-server/src/
git add crates/hyperllama-cli/src/commands/serve.rs
git add crates/hyperllama-engine/src/max.rs
git add python/max_service/server.py
git add README.md
git add docs/
git add PHASE_0_COMPLETE.md
git add PHASE_1_PROGRESS.md
git add SESSION_SUMMARY.md
git add COMMIT_MESSAGE.txt

# Commit using the prepared message
git commit -F COMMIT_MESSAGE.txt

# Push to remote
git push
```

## 3. Test the REST API Server

### Terminal 1: Start Python MAX Engine Service
```bash
cd /Users/nick/github/nijaru/hyperllama/python
PYTHONPATH=python uvicorn max_service.server:app --host 127.0.0.1 --port 8100
```

### Terminal 2: Start HyperLlama Server
```bash
cd /Users/nick/github/nijaru/hyperllama
cargo run --release --bin hyperllama -- serve --host 127.0.0.1 --port 11434
```

### Terminal 3: Test Endpoints

**Health Check:**
```bash
curl http://localhost:11434/health
# Expected: OK
```

**Non-Streaming Generation:**
```bash
curl -X POST http://localhost:11434/api/generate \
  -H "Content-Type: application/json" \
  -d '{
    "model": "modularai/Llama-3.1-8B-Instruct-GGUF",
    "prompt": "What is 2+2?",
    "stream": false
  }'

# Expected: JSON response with "response" field containing "4"
```

**Streaming Generation:**
```bash
curl -X POST http://localhost:11434/api/generate \
  -H "Content-Type: application/json" \
  -d '{
    "model": "modularai/Llama-3.1-8B-Instruct-GGUF",
    "prompt": "Tell me a short story",
    "stream": true
  }'

# Expected: Server-Sent Events stream
# data: {"model":"...","response":"Once","done":false}
# data: {"model":"...","response":" upon","done":false}
# ...
```

**List Models:**
```bash
curl http://localhost:11434/api/tags
# Expected: {"models": []}
```

## 4. Deploy to Fedora + RTX 4090

### Sync Project to Fedora
```bash
# From Mac
rsync -avz --exclude target --exclude .git \
  /Users/nick/github/nijaru/hyperllama/ \
  nick@fedora:/home/nick/hyperllama/
```

### On Fedora: Install Dependencies
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Python dependencies
cd ~/hyperllama/python
pip install -r requirements.txt

# Install MAX Engine with CUDA support
pip install modular --index-url https://dl.modular.com/public/nightly/python/simple/
```

### On Fedora: Build and Test
```bash
cd ~/hyperllama

# Build
cargo build --release

# Terminal 1: Start Python service
cd python && PYTHONPATH=python uvicorn max_service.server:app --host 127.0.0.1 --port 8100

# Terminal 2: Start HyperLlama server
./target/release/hyperllama serve --host 127.0.0.1 --port 11434

# Terminal 3: Run benchmark
./target/release/hyperllama bench "modularai/Llama-3.1-8B-Instruct-GGUF" "Test prompt" -i 10
```

### Compare CPU vs GPU Performance

**Expected Results on RTX 4090:**
- Throughput: 200-800 tokens/sec (vs 23.71 on M3 Max CPU)
- Latency: 50-200ms per request (vs 2108ms on CPU)
- Speedup: 10-50x improvement

## 5. Document GPU Performance

Update these files with actual GPU benchmarks:
- `README.md` - Add GPU performance section
- `PHASE_1_PROGRESS.md` - Add GPU test results
- `docs/PHASE_1_REST_API.md` - Update performance section

## 6. Phase 2 Planning

Once GPU testing is complete, start Phase 2:

### Chat Completions Endpoint
```rust
// In crates/hyperllama-server/src/api.rs
#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub stream: bool,
}

pub async fn chat(
    State(state): State<ServerState>,
    Json(req): Json<ChatRequest>,
) -> Response {
    // Implementation
}
```

### Model Management
- `POST /api/pull` - Download models from HuggingFace
- `POST /api/show` - Show model information
- `GET /api/tags` - List all loaded models (implement properly)
- `DELETE /api/delete` - Delete models

### vLLM Backend
Create `python/vllm_service/server.py` following same pattern as MAX service:
```python
from vllm import LLM, SamplingParams

app = FastAPI()

@app.post("/generate")
async def generate(request: GenerateRequest):
    llm = engine.models[request.model_id]
    outputs = llm.generate([request.prompt], sampling_params)
    return {"text": outputs[0].outputs[0].text}
```

## Common Issues

### Compilation Errors

**Issue**: `cannot find type MaxEngine in this scope`
**Fix**: Add `use hyperllama_engine::MaxEngine;` to api.rs

**Issue**: `trait InferenceEngine is not in scope`
**Fix**: Add `use hyperllama_engine::InferenceEngine;` to api.rs

**Issue**: Mutex lock across await boundary
**Fix**: Already handled with proper scoping in api.rs

### Runtime Errors

**Issue**: `MAX Engine service not available`
**Fix**: Make sure Python service is running on port 8100

**Issue**: `Model not found`
**Fix**: Use correct HuggingFace model ID format:
- ✅ `modularai/Llama-3.1-8B-Instruct-GGUF`
- ❌ `modularai/llama-3.1-8b-instruct`

**Issue**: Port already in use
**Fix**:
```bash
# Kill existing processes
lsof -ti:11434 | xargs kill
lsof -ti:8100 | xargs kill
```

## Testing Checklist

- [ ] Compilation succeeds without errors
- [ ] Health endpoint returns "OK"
- [ ] Non-streaming generation works
- [ ] Streaming generation works with SSE
- [ ] Model auto-loads on first request
- [ ] Concurrent requests work
- [ ] Error handling works (invalid model, etc.)

## Performance Testing Checklist

- [ ] CPU baseline established (M3 Max: 23.71 tok/s)
- [ ] GPU performance measured on RTX 4090
- [ ] Speedup calculated and documented
- [ ] Concurrent request performance tested
- [ ] Memory usage monitored
- [ ] Latency percentiles (p50, p95, p99) measured

## Files to Review

Before deploying, review these files for any TODOs or issues:

1. `crates/hyperllama-server/src/api.rs` - Main API logic
2. `crates/hyperllama-engine/src/max.rs` - Streaming implementation
3. `python/max_service/server.py` - Python service
4. `README.md` - User-facing documentation

## Success Criteria

Phase 1 is successful when:
- ✅ Code compiles without errors
- ✅ REST API endpoints work correctly
- ✅ Streaming generation works end-to-end
- ✅ GPU performance is 10x+ faster than CPU
- ✅ All documentation is up to date

## What's Next After Phase 1

### Phase 2 (Weeks 3-4)
- Chat completions endpoint
- Model management (pull, show, delete)
- Authentication and rate limiting
- Metrics and observability

### Phase 3 (Weeks 5-6)
- vLLM backend integration
- Performance comparison: MAX vs vLLM
- Automatic engine selection based on hardware
- Connection pooling

### Phase 4 (Weeks 7-8)
- Request batching
- Multi-engine load balancing
- Docker deployment
- Production hardening

---

**Current Status**: ✅ Phase 0 Complete | ✅ Phase 1 Code Complete | ⏳ Ready for Testing

**Next Action**: Run `cargo build --release` and test!
