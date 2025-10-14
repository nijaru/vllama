# HyperLlama Python Services

This directory contains Python microservices that bridge HyperLlama (Rust) with Python-based inference engines.

## MAX Engine Service

Thin wrapper around MAX Engine's Python API.

### Installation

```bash
# Install Python dependencies
pip install -r requirements.txt

# Install MAX Engine (nightly)
pip install modular --index-url https://dl.modular.com/public/nightly/python/simple/
```

### Running

```bash
# Start the service
python -m max_service.server

# Or with uvicorn directly
uvicorn max_service.server:app --host 127.0.0.1 --port 8100
```

### API

#### Health Check
```bash
curl http://localhost:8100/health
```

#### Load Model
```bash
curl -X POST http://localhost:8100/models/load \
  -H "Content-Type: application/json" \
  -d '{"model_path": "modularai/Llama-3.1-8B-Instruct-GGUF"}'
```

#### Generate
```bash
curl -X POST http://localhost:8100/generate \
  -H "Content-Type: application/json" \
  -d '{
    "model_id": "modularai_Llama-3.1-8B-Instruct-GGUF",
    "prompt": "Hello, world!",
    "max_tokens": 100
  }'
```

#### List Models
```bash
curl http://localhost:8100/models
```

#### Unload Model
```bash
curl -X POST "http://localhost:8100/models/unload?model_id=modularai_Llama-3.1-8B-Instruct-GGUF"
```
