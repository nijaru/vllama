# vLLama Setup on Fedora + RTX 4090

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

### 3. Install Python Dependencies
```bash
cd ~/github/nijaru/vllama/python
uv pip install -r requirements.txt
```

### 4. Build vLLama
```bash
cd ~/github/nijaru/vllama
cargo build --release
```

### 5. Test Everything Works

**Terminal 1: Start vLLM Service**
```bash
cd ~/github/nijaru/vllama/python
uvicorn llm_service.server:app --host 127.0.0.1 --port 8100
```

**Terminal 2: Run Benchmark**
```bash
cd ~/github/nijaru/vllama
./target/release/vllama bench "meta-llama/Llama-3.1-8B-Instruct" "Test prompt" -i 10
```

## Expected GPU Performance

**Mac M3 Max CPU Baseline:**
- Throughput: 23.71 tokens/sec
- Latency: 2108ms per request

**Fedora RTX 4090 GPU Target:**
- Throughput: 200-800 tokens/sec
- Latency: 50-200ms per request
- Speedup: **10-50x improvement**

## Continue Development

Once setup is complete:

1. Open the `vllama` directory in your editor
2. Reference `PROJECT_STATUS.md` for current status and roadmap
3. See `README.md` for API usage examples

## Key Files to Reference

- `PROJECT_STATUS.md` - Current status and roadmap (source of truth)
- `README.md` - User-facing getting started guide
- `docs/` - Additional documentation (mostly archived)

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
```bash
# Kill processes on ports
lsof -ti:8100 | xargs kill -9
lsof -ti:11434 | xargs kill -9
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
1. Start the vLLM service
2. Start the vLLama server
3. Make API requests (see README.md for examples)
4. Test performance

See `PROJECT_STATUS.md` for current features and roadmap!
