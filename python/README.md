# vLLama Python Environment

This directory contains Python dependencies for vLLama's inference backend.

## Architecture

vLLama uses vLLM's official OpenAI-compatible server for inference. The Python environment is managed by `uv` and spawned automatically by the Rust binary.

**No manual Python service needed** - the `vllama serve` command handles everything.

## Installation

```bash
# Install uv (if not already installed)
curl -LsSf https://astral.sh/uv/install.sh | sh

# Install Python dependencies
cd python
uv sync --extra vllm
```

This installs:
- vLLM (GPU-accelerated inference engine)
- FastAPI, Uvicorn, Pydantic (used by vLLM's OpenAI server)

## Usage

The Rust binary automatically manages the Python environment:

```bash
# vLLama handles Python environment via uv
cargo run --release -- serve --model meta-llama/Llama-3.2-1B-Instruct

# Under the hood:
# 1. Rust spawns: uv run --directory python python -m vllm.entrypoints.openai.api_server
# 2. vLLM server starts on port 8100
# 3. vLLama serves Ollama-compatible API on port 11434
```

## Manual Testing (Optional)

If you need to test vLLM directly:

```bash
cd python

# Start vLLM OpenAI server
uv run python -m vllm.entrypoints.openai.api_server \
  --model facebook/opt-125m \
  --port 8100

# Test
curl http://localhost:8100/v1/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "facebook/opt-125m",
    "prompt": "Hello, world!",
    "max_tokens": 50
  }'
```

## Dependencies

Managed via `pyproject.toml`:
- **Required:** fastapi, uvicorn, pydantic
- **Optional (vllm):** vLLM inference engine

```toml
[project.optional-dependencies]
vllm = [
    "vllm>=0.5.0",
]
```

## Troubleshooting

**vLLM not found:**
```bash
cd python
uv sync --extra vllm
```

**GPU memory errors:**
```bash
# Reduce GPU memory utilization
vllama serve --model MODEL --gpu-memory-utilization 0.5
```

**Python version issues:**
```bash
# vLLM requires Python 3.10-3.12
mise use python@3.12
uv sync --extra vllm
```
