# vllama Setup on Fedora + RTX 4090

## Prerequisites

### Port Conflict Warning

**NOTE:** vllama uses port 11435 by default, which allows it to run alongside Ollama (port 11434).

**If using vllama as drop-in replacement (`--port 11434`):** Stop Ollama first:

```bash
# Stop Ollama service
sudo systemctl stop ollama

# Or disable it from auto-starting
sudo systemctl disable ollama

# Verify port 11435 is free
lsof -i:11435  # Should return nothing
```

**Running alongside Ollama (default):**
```bash
# vllama on default port 11435
vllama serve --model <model>

# Ollama on its default port 11434
ollama serve
```

**Using vllama as Ollama drop-in replacement:**
```bash
# Stop Ollama first
sudo systemctl stop ollama

# Run vllama on Ollama's port
vllama serve --model <model> --port 11434
```

## Quick Setup

Run these commands on your Fedora machine to get started:

### 1. Clone Repository
```bash
cd ~/github/nijaru
git clone https://github.com/nijaru/vllama
# Or if already cloned: git pull origin main
cd vllama
```

### 2. Install Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
rustc --version  # Verify installation
```

### 3. Install uv (Python Package Manager)
```bash
curl -LsSf https://astral.sh/uv/install.sh | sh
source "$HOME/.cargo/env"  # Reload PATH
uv --version  # Verify installation
```

### 4. Install Python Dependencies
```bash
cd ~/github/nijaru/vllama/python
uv sync  # Installs vLLM and all dependencies
```

### 5. Build vllama
```bash
cd ~/github/nijaru/vllama
cargo build --release
```

### 6. Test Everything Works

**Start server (auto-starts vLLM):**
```bash
./target/release/vllama serve --model Qwen/Qwen2.5-0.5B-Instruct
```

**In another terminal, test generation:**
```bash
curl -X POST http://localhost:11435/api/generate \
  -H "Content-Type: application/json" \
  -d '{
    "model": "Qwen/Qwen2.5-0.5B-Instruct",
    "prompt": "What is 2+2?",
    "stream": false
  }'
```

**Run benchmarks:**
```bash
./target/release/vllama bench Qwen/Qwen2.5-0.5B-Instruct "Test prompt" -i 10
```

## Expected GPU Performance

**RTX 4090 Performance:**
- **Sequential:** 232ms per request (4.4x faster than Ollama)
- **Concurrent (5 requests):** 0.217s total (29.95x faster than Ollama)
- **High concurrency (50 requests):** 23.6 req/s sustained throughput

See [docs/PERFORMANCE.md](PERFORMANCE.md) for comprehensive benchmarks.

## Continue Development

Once setup is complete:

1. See [ai/STATUS.md](../ai/STATUS.md) for current status and roadmap
2. See [ai/TODO.md](../ai/TODO.md) for active tasks and priorities
3. See [README.md](../README.md) for API usage examples
4. See [CLAUDE.md](../CLAUDE.md) for development guidelines

## Key Files to Reference

- [ai/STATUS.md](../ai/STATUS.md) - Current status (read first)
- [ai/TODO.md](../ai/TODO.md) - Active tasks and priorities
- [CLAUDE.md](../CLAUDE.md) - Development guidelines
- [README.md](../README.md) - User-facing getting started guide
- [TESTING_STATUS.md](../TESTING_STATUS.md) - What's tested vs not

## Troubleshooting

### CUDA Not Detected
```bash
# Check NVIDIA driver
nvidia-smi

# Check CUDA toolkit
nvcc --version

# Verify vLLM can see GPU
python -c "import torch; print(f'CUDA available: {torch.cuda.is_available()}')"
```

### Port Already in Use

**Port 11435 conflict (usually Ollama):**
```bash
# Check what's using port 11435
lsof -i:11435

# Stop Ollama if it's running
sudo systemctl stop ollama

# Or kill any process on port 11435
lsof -ti:11435 | xargs kill -9
```

**Port 8100 conflict (vLLM):**
```bash
# Kill any process on port 8100
lsof -ti:8100 | xargs kill -9
```

### Compilation Errors
```bash
# Update Rust
rustup update

# Clean and rebuild
cargo clean
cargo build --release
```

## Ready to Use!

Once setup is complete, you're ready to:
1. Start vllama server (auto-starts vLLM)
2. Make API requests (see [README.md](../README.md) for examples)
3. Run benchmarks and tests
4. Monitor with `/health` endpoint

**Remember:** Stop Ollama first to avoid port conflicts!

See [ai/STATUS.md](../ai/STATUS.md) for current features and roadmap.
