# HyperLlama Setup on Fedora + RTX 4090

## Quick Setup

Run these commands on your Fedora machine to get started:

### 1. Clone Repository
```bash
cd ~/github/nijaru
git clone https://github.com/nijaru/hyperllama
# Or if already cloned: git pull origin main
cd hyperllama
```

### 2. Install Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
rustc --version  # Verify installation
```

### 3. Install Python Dependencies
```bash
cd ~/github/nijaru/hyperllama/python
pip install -r requirements.txt
```

### 4. Install MAX Engine with CUDA Support
```bash
# Install Modular CLI
curl -s https://get.modular.com | sh -

# Install MAX Engine (nightly for GPU support)
modular install max

# Verify GPU is detected
python -c "import torch; print(f'CUDA available: {torch.cuda.is_available()}')"
```

### 5. Build HyperLlama
```bash
cd ~/github/nijaru/hyperllama
cargo build --release
```

### 6. Test Everything Works

**Terminal 1: Start Python MAX Engine Service**
```bash
cd ~/github/nijaru/hyperllama/python
PYTHONPATH=python uvicorn max_service.server:app --host 127.0.0.1 --port 8100
```

**Terminal 2: Run Benchmark**
```bash
cd ~/github/nijaru/hyperllama
./target/release/hyperllama bench "modularai/Llama-3.1-8B-Instruct-GGUF" "Test prompt" -i 10
```

## Expected GPU Performance

**Mac M3 Max CPU Baseline:**
- Throughput: 23.71 tokens/sec
- Latency: 2108ms per request

**Fedora RTX 4090 GPU Target:**
- Throughput: 200-800 tokens/sec
- Latency: 50-200ms per request
- Speedup: **10-50x improvement**

## Continue Development with Claude

Once setup is complete, you can continue with Claude on Fedora:

1. Open Claude Code on Fedora
2. Open the `hyperllama` directory
3. Reference `NEXT_STEPS.md` for what to do next
4. Claude will have all the context from SESSION_SUMMARY.md

## Key Files to Reference

- `NEXT_STEPS.md` - Step-by-step instructions
- `SESSION_SUMMARY.md` - Complete development history
- `PHASE_1_PROGRESS.md` - What was just built
- `docs/PHASE_1_REST_API.md` - API documentation

## Troubleshooting

### CUDA Not Detected
```bash
# Check NVIDIA driver
nvidia-smi

# Check CUDA toolkit
nvcc --version

# Reinstall MAX Engine
modular uninstall max
modular install max
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

## Ready for GPU Benchmarks!

Once setup is complete, you're ready to:
1. Test compilation
2. Run GPU benchmarks
3. Compare CPU vs GPU performance
4. Document results
5. Continue to Phase 2

See `NEXT_STEPS.md` for detailed instructions!
