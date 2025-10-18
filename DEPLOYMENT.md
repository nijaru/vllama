# vLLama Production Deployment Guide

This guide covers deploying vLLama in production environments.

## Architecture Overview

vLLama consists of two services:
1. **vLLM Service** (Python) - Handles GPU inference on port 8100
2. **vLLama Server** (Rust) - Ollama-compatible API on port 11434

Both services must run for the system to function.

## Prerequisites

### Hardware
- **GPU**: NVIDIA GPU with CUDA support (minimum 8GB VRAM for 7B models)
- **RAM**: 16GB+ system RAM recommended
- **Storage**: 50GB+ for model cache

### Software
- **OS**: Linux (Ubuntu 22.04+, Fedora 39+, or similar)
- **GPU**: NVIDIA drivers + CUDA 12.1+
- **Python**: 3.10+ (via mise or system)
- **Rust**: 1.75+ for building from source

### Installation

**Install mise (Python version management):**
```bash
curl https://mise.jdx.dev/install.sh | sh
```

**Install uv (Python package manager):**
```bash
mise use python@3.12
pip install uv
```

**Install vLLM:**
```bash
cd vllama/python
uv sync
```

**Build vLLama:**
```bash
cargo build --release
```

Binary will be at `target/release/vllama`

## Deployment Methods

### Method 1: systemd Services (Recommended for Linux)

Create two systemd service files:

**/etc/systemd/system/vllama-vllm.service:**
```ini
[Unit]
Description=vLLM Inference Service for vLLama
After=network.target

[Service]
Type=simple
User=vllama
Group=vllama
WorkingDirectory=/opt/vllama/python
Environment="PATH=/home/vllama/.local/share/mise/installs/python/3.12/bin:/usr/local/bin:/usr/bin"
Environment="CUDA_VISIBLE_DEVICES=0"
ExecStart=/home/vllama/.local/share/mise/installs/python/3.12/bin/uvicorn llm_service.server:app --host 127.0.0.1 --port 8100
Restart=always
RestartSec=10

# Resource limits
LimitNOFILE=65536
# Allow GPU access
DeviceAllow=/dev/nvidia0 rw
DeviceAllow=/dev/nvidiactl rw
DeviceAllow=/dev/nvidia-uvm rw

[Install]
WantedBy=multi-user.target
```

**/etc/systemd/system/vllama.service:**
```ini
[Unit]
Description=vLLama Ollama-Compatible Server
After=network.target vllama-vllm.service
Requires=vllama-vllm.service

[Service]
Type=simple
User=vllama
Group=vllama
WorkingDirectory=/opt/vllama
ExecStart=/opt/vllama/target/release/vllama serve --host 0.0.0.0 --port 11434
Restart=always
RestartSec=10

# Resource limits
LimitNOFILE=65536

[Install]
WantedBy=multi-user.target
```

**Enable and start services:**
```bash
sudo systemctl daemon-reload
sudo systemctl enable vllama-vllm vllama
sudo systemctl start vllama-vllm vllama
```

**Check status:**
```bash
sudo systemctl status vllama-vllm
sudo systemctl status vllama
```

**View logs:**
```bash
sudo journalctl -u vllama-vllm -f
sudo journalctl -u vllama -f
```

### Method 2: Docker Compose (Cross-Platform)

**docker-compose.yml:**
```yaml
version: '3.8'

services:
  vllm:
    image: vllm/vllm-openai:latest
    container_name: vllama-vllm
    ports:
      - "127.0.0.1:8100:8100"
    environment:
      - CUDA_VISIBLE_DEVICES=0
    deploy:
      resources:
        reservations:
          devices:
            - driver: nvidia
              count: 1
              capabilities: [gpu]
    volumes:
      - ./python:/app
      - model-cache:/root/.cache/huggingface
    command: >
      uvicorn llm_service.server:app
      --host 0.0.0.0
      --port 8100
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8100/health"]
      interval: 30s
      timeout: 10s
      retries: 3

  vllama:
    build:
      context: .
      dockerfile: Dockerfile
    container_name: vllama-server
    ports:
      - "11434:11434"
    depends_on:
      vllm:
        condition: service_healthy
    environment:
      - VLLM_URL=http://vllm:8100
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:11434/health"]
      interval: 30s
      timeout: 10s
      retries: 3

volumes:
  model-cache:
```

**Dockerfile:**
```dockerfile
FROM rust:1.75 as builder

WORKDIR /usr/src/vllama
COPY . .

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y ca-certificates curl && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/vllama/target/release/vllama /usr/local/bin/vllama

EXPOSE 11434

CMD ["vllama", "serve", "--host", "0.0.0.0", "--port", "11434"]
```

**Start services:**
```bash
docker-compose up -d
```

**View logs:**
```bash
docker-compose logs -f vllm
docker-compose logs -f vllama
```

### Method 3: Manual Process Management (Development)

**Terminal 1 - vLLM Service:**
```bash
cd python
uv run uvicorn llm_service.server:app --host 127.0.0.1 --port 8100
```

**Terminal 2 - vLLama Server:**
```bash
cargo run --release --bin vllama -- serve --host 127.0.0.1 --port 11434
```

## Configuration

### Environment Variables

**vLLM Service:**
- `CUDA_VISIBLE_DEVICES`: GPU selection (default: "0")
- `VLLM_CACHE_DIR`: Model cache location (default: ~/.cache/vllama/)
- `VLLM_TENSOR_PARALLEL_SIZE`: Multi-GPU tensor parallelism (default: 1)

**vLLama Server:**
- `VLLM_URL`: vLLM service URL (default: http://127.0.0.1:8100)
- `RUST_LOG`: Logging level (info, debug, trace)

### Model Cache Location

Models download to `~/.cache/vllama/` by default. For production:

```bash
# Create dedicated cache directory
sudo mkdir -p /var/cache/vllama
sudo chown vllama:vllama /var/cache/vllama

# Symlink to user cache
ln -s /var/cache/vllama ~/.cache/vllama
```

### Memory Management

**Disable GUI on GPU Server (Fedora/RHEL):**
```bash
sudo systemctl stop gdm
sudo systemctl disable gdm
```

This frees 1-2GB VRAM for models.

**Check VRAM usage:**
```bash
nvidia-smi
```

## Security Best Practices

### Network Security

**Firewall (firewalld):**
```bash
# Only allow localhost by default
sudo firewall-cmd --permanent --add-service=vllama
sudo firewall-cmd --reload
```

**Nginx Reverse Proxy with TLS:**
```nginx
server {
    listen 443 ssl http2;
    server_name llm.example.com;

    ssl_certificate /etc/letsencrypt/live/llm.example.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/llm.example.com/privkey.pem;

    location / {
        proxy_pass http://127.0.0.1:11434;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # Streaming support
        proxy_buffering off;
        proxy_cache off;
        chunked_transfer_encoding on;
    }
}
```

### User Isolation

**Create dedicated user:**
```bash
sudo useradd -r -s /bin/false vllama
sudo mkdir -p /opt/vllama
sudo chown -R vllama:vllama /opt/vllama
```

**Run as non-root:**
Never run vLLama services as root. Use the dedicated `vllama` user.

### Rate Limiting

Use nginx or reverse proxy for rate limiting:
```nginx
limit_req_zone $binary_remote_addr zone=llm_limit:10m rate=10r/s;

server {
    location / {
        limit_req zone=llm_limit burst=20 nodelay;
        proxy_pass http://127.0.0.1:11434;
    }
}
```

## Monitoring

### Health Checks

**Check service health:**
```bash
curl http://localhost:11434/health
# Expected: {"status":"ok"}
```

**Check running models:**
```bash
curl http://localhost:11434/api/ps
```

### Prometheus Metrics (Future)

vLLama will support Prometheus metrics in a future release:
- `/metrics` endpoint
- Request counts, latencies, token throughput
- GPU utilization, VRAM usage

### Log Management

**Configure log rotation:**
```bash
# /etc/logrotate.d/vllama
/var/log/vllama/*.log {
    daily
    missingok
    rotate 14
    compress
    delaycompress
    notifempty
    create 0644 vllama vllama
}
```

## Scaling & Performance

### Multi-GPU Setup

**Tensor Parallelism (single model across GPUs):**
```bash
# vLLM service with 2 GPUs
CUDA_VISIBLE_DEVICES=0,1 \
VLLM_TENSOR_PARALLEL_SIZE=2 \
uvicorn llm_service.server:app --host 127.0.0.1 --port 8100
```

**Pipeline Parallelism (future):**
Not yet supported. Coming in Phase 4.

### Load Balancing

For horizontal scaling, run multiple vLLama instances behind a load balancer:

```nginx
upstream vllama_backend {
    least_conn;
    server 127.0.0.1:11434;
    server 127.0.0.1:11435;
    server 127.0.0.1:11436;
}

server {
    location / {
        proxy_pass http://vllama_backend;
    }
}
```

Each instance needs its own vLLM service.

### Model Preloading

**Preload models at startup:**
```bash
# After services start
curl -X POST http://localhost:11434/api/pull \
  -d '{"name":"meta-llama/Llama-3.1-8B-Instruct"}'
```

Add to systemd `ExecStartPost` or Docker healthcheck.

## Troubleshooting

### vLLM Service Won't Start

**Check CUDA:**
```bash
nvidia-smi
python3 -c "import torch; print(torch.cuda.is_available())"
```

**Check port:**
```bash
lsof -i :8100
```

### vLLama Server Connection Refused

**Verify vLLM service is running:**
```bash
curl http://localhost:8100/health
```

**Check logs:**
```bash
journalctl -u vllama -n 50
```

### Out of Memory (OOM)

**Check VRAM usage:**
```bash
nvidia-smi
```

**Solutions:**
- Use smaller model (8B instead of 70B)
- Stop GUI (GDM/X11) to free VRAM
- Increase system swap (not recommended for GPU)
- Use quantized models

### Slow Inference

**Check GPU utilization:**
```bash
nvidia-smi dmon
```

**Benchmark:**
```bash
cargo run --release --bin vllama -- bench \
  "meta-llama/Llama-3.1-8B-Instruct" \
  "Test prompt" \
  -i 10
```

See [BENCHMARKS.md](BENCHMARKS.md) for detailed performance testing.

## Maintenance

### Updating vLLama

```bash
cd /opt/vllama
git pull
cargo build --release
sudo systemctl restart vllama
```

### Updating vLLM

```bash
cd /opt/vllama/python
uv sync --upgrade
sudo systemctl restart vllama-vllm
```

### Clearing Model Cache

```bash
rm -rf ~/.cache/vllama/*
# Models will re-download on next request
```

### Backup

**Important paths:**
- `/opt/vllama` - Application code
- `~/.cache/vllama/` - Model cache (50GB+, optional backup)
- `/etc/systemd/system/vllama*.service` - Service configs

## Production Checklist

- [ ] Services run as non-root user
- [ ] Firewall configured (only necessary ports open)
- [ ] TLS/HTTPS enabled (nginx reverse proxy)
- [ ] Rate limiting configured
- [ ] Health checks enabled
- [ ] Log rotation configured
- [ ] Monitoring/alerts set up
- [ ] Backup strategy defined
- [ ] Model cache on fast storage
- [ ] GPU drivers updated
- [ ] systemd services enabled for auto-start
- [ ] Documentation for team

## Support

- **Issues**: https://github.com/nijaru/vllama/issues
- **Benchmarks**: [BENCHMARKS.md](BENCHMARKS.md)
- **Project Status**: [PROJECT_STATUS.md](PROJECT_STATUS.md)
