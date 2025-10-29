# vllama Deployment Guide

**Target:** Production Linux deployments with NVIDIA GPUs

This guide covers deploying vllama in production environments with Docker, systemd, reverse proxies, and monitoring.

## Table of Contents

- [Quick Start](#quick-start)
- [Docker Deployment](#docker-deployment)
- [Systemd Service](#systemd-service)
- [Reverse Proxy](#reverse-proxy)
- [Monitoring](#monitoring)
- [Security](#security)
- [Performance Tuning](#performance-tuning)
- [Troubleshooting](#troubleshooting)

## Quick Start

**Prerequisites:**
- Linux server (Ubuntu 22.04+ or Fedora 38+ recommended)
- NVIDIA GPU with CUDA 12.1+
- Docker + NVIDIA Container Runtime OR bare metal with Rust/Python
- 16GB+ RAM
- 50GB+ disk space (for models)

**Fastest deployment (Docker):**
```bash
git clone https://github.com/nijaru/vllama.git
cd vllama
docker compose up -d
```

## Docker Deployment

### Option 1: Docker Compose (Recommended)

**1. Clone repository:**
```bash
git clone https://github.com/nijaru/vllama.git
cd vllama
```

**2. Configure environment:**
Edit `docker-compose.yml` to set:
- `MODEL_NAME`: Your HuggingFace model (e.g., `Qwen/Qwen2.5-7B-Instruct`)
- `GPU_MEMORY_UTILIZATION`: 0.5-0.9 (use 0.9 for 7B models)
- `MAX_NUM_SEQS`: 256 for high concurrency

**3. Start services:**
```bash
# Production (vllama only)
docker compose up -d

# With monitoring (Prometheus + Grafana)
docker compose --profile monitoring up -d
```

**4. Verify health:**
```bash
curl http://localhost:11434/health
```

**5. Test inference:**
```bash
curl -X POST http://localhost:11434/api/generate \
  -H "Content-Type: application/json" \
  -d '{
    "model": "Qwen/Qwen2.5-7B-Instruct",
    "prompt": "What is vllama?",
    "stream": false
  }'
```

### Option 2: Docker Build & Run

**1. Build image:**
```bash
docker build -t vllama:latest .
```

**2. Run container:**
```bash
docker run -d \
  --name vllama \
  --runtime=nvidia \
  --gpus all \
  -p 11434:11434 \
  -v huggingface-cache:/root/.cache/huggingface \
  -e MODEL_NAME=Qwen/Qwen2.5-7B-Instruct \
  -e GPU_MEMORY_UTILIZATION=0.9 \
  -e RUST_LOG=info \
  -e VLLAMA_LOG_FORMAT=json \
  --restart unless-stopped \
  vllama:latest
```

**3. View logs:**
```bash
docker logs -f vllama
```

### Docker Configuration

**Environment variables:**
```bash
# Model settings
MODEL_NAME=Qwen/Qwen2.5-7B-Instruct
GPU_MEMORY_UTILIZATION=0.9
MAX_NUM_SEQS=256

# Server settings
VLLAMA_HOST=0.0.0.0
VLLAMA_PORT=11434
VLLM_PORT=8100

# Logging
RUST_LOG=info              # debug for verbose logs
VLLAMA_LOG_FORMAT=json     # json for structured logs
```

**Volume mounts:**
```bash
# Cache models (avoids re-downloading)
-v /data/huggingface:/root/.cache/huggingface

# Persist logs
-v /var/log/vllama:/var/log/vllama

# Custom config
-v /etc/vllama/config.toml:/root/.config/vllama/config.toml
```

## Systemd Service

For bare-metal deployments without Docker.

### Installation

**1. Build vllama:**
```bash
cd vllama
cargo build --release
sudo cp target/release/vllama /usr/local/bin/
```

**2. Create user:**
```bash
sudo useradd -r -s /bin/false -d /opt/vllama vllama
sudo mkdir -p /opt/vllama /var/log/vllama
sudo chown vllama:vllama /opt/vllama /var/log/vllama
```

**3. Install systemd service:**
```bash
sudo cp deployment/vllama.service /etc/systemd/system/
sudo systemctl daemon-reload
```

**4. Edit configuration:**
```bash
sudo systemctl edit vllama
```

Add your model and settings:
```ini
[Service]
Environment="MODEL_NAME=Qwen/Qwen2.5-7B-Instruct"
Environment="GPU_MEMORY_UTILIZATION=0.9"
```

**5. Enable and start:**
```bash
sudo systemctl enable vllama
sudo systemctl start vllama
```

**6. Check status:**
```bash
sudo systemctl status vllama
sudo journalctl -u vllama -f
```

### Service Management

```bash
# Start/stop/restart
sudo systemctl start vllama
sudo systemctl stop vllama
sudo systemctl restart vllama

# Enable/disable autostart
sudo systemctl enable vllama
sudo systemctl disable vllama

# View logs
sudo journalctl -u vllama -n 100 --no-pager
sudo journalctl -u vllama -f  # Follow

# Reload after config changes
sudo systemctl daemon-reload
sudo systemctl restart vllama
```

## Reverse Proxy

Add HTTPS, rate limiting, and authentication with a reverse proxy.

### Nginx

**1. Install nginx:**
```bash
sudo apt install nginx  # Ubuntu/Debian
sudo dnf install nginx  # Fedora
```

**2. Copy configuration:**
```bash
sudo cp deployment/nginx.conf /etc/nginx/sites-available/vllama
sudo ln -s /etc/nginx/sites-available/vllama /etc/nginx/sites-enabled/
```

**3. Edit configuration:**
```bash
sudo nano /etc/nginx/sites-available/vllama
```

Replace `llm.example.com` with your domain.

**4. Get SSL certificate (Let's Encrypt):**
```bash
sudo apt install certbot python3-certbot-nginx
sudo certbot --nginx -d llm.example.com
```

**5. Test and reload:**
```bash
sudo nginx -t
sudo systemctl reload nginx
```

**6. Test HTTPS:**
```bash
curl https://llm.example.com/health
```

### Caddy (Easier alternative)

**1. Install Caddy:**
```bash
# Ubuntu/Debian
sudo apt install -y debian-keyring debian-archive-keyring apt-transport-https
curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/gpg.key' | sudo gpg --dearmor -o /usr/share/keyrings/caddy-stable-archive-keyring.gpg
curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/debian.deb.txt' | sudo tee /etc/apt/sources.list.d/caddy-stable.list
sudo apt update && sudo apt install caddy

# Fedora
sudo dnf install caddy
```

**2. Copy configuration:**
```bash
sudo cp deployment/caddy.conf /etc/caddy/Caddyfile
```

**3. Edit configuration:**
```bash
sudo nano /etc/caddy/Caddyfile
```

Replace `llm.example.com` with your domain.

**4. Start Caddy:**
```bash
sudo systemctl enable caddy
sudo systemctl start caddy
```

Caddy automatically obtains SSL certificates from Let's Encrypt!

## Monitoring

Track performance, resource usage, and errors.

### Built-in Health Endpoint

```bash
curl http://localhost:11434/health | jq
```

Returns:
```json
{
  "status": "ok",
  "vllm_status": "healthy",
  "models": ["Qwen/Qwen2.5-7B-Instruct"],
  "gpu": {
    "name": "NVIDIA GeForce RTX 4090",
    "memory_total_mb": "24564",
    "memory_used_mb": "14231",
    "memory_free_mb": "10333",
    "utilization_percent": "87"
  },
  "memory": {
    "total_mb": 32768,
    "used_mb": 8192,
    "available_mb": 24576
  },
  "uptime_seconds": 3600
}
```

### JSON Logging

Enable structured logs for aggregation:

```bash
export VLLAMA_LOG_FORMAT=json
```

Example log entry:
```json
{
  "timestamp": "2025-10-29T10:30:00Z",
  "level": "INFO",
  "request_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "method": "POST",
  "uri": "/api/generate",
  "status": 200,
  "latency_ms": 234,
  "message": "request completed"
}
```

### Prometheus + Grafana

See [docs/MONITORING.md](MONITORING.md) for complete Prometheus/Grafana setup.

**Quick start with Docker Compose:**
```bash
docker compose --profile monitoring up -d
```

Access:
- Prometheus: http://localhost:9090
- Grafana: http://localhost:3000 (admin/admin)

## Security

### Authentication

**Option 1: API Key (Custom Middleware)**

Not built-in. Add via reverse proxy or custom middleware.

**Option 2: Reverse Proxy Basic Auth**

**Nginx:**
```bash
# Create password file
sudo apt install apache2-utils
sudo htpasswd -c /etc/nginx/.htpasswd username

# Uncomment in nginx.conf:
# auth_basic "vllama API";
# auth_basic_user_file /etc/nginx/.htpasswd;

sudo systemctl reload nginx
```

**Caddy:**
```bash
# Generate bcrypt hash
caddy hash-password

# Add to Caddyfile:
# basicauth {
#     username $2a$14$...hash...
# }

sudo systemctl reload caddy
```

**Option 3: mTLS (Client Certificates)**

For enterprise deployments. Configure at reverse proxy level.

### Rate Limiting

**Nginx:** Already configured (10 req/s per IP, burst 20)

Adjust in `nginx.conf`:
```nginx
limit_req_zone $binary_remote_addr zone=vllama_limit:10m rate=10r/s;
```

**Caddy:** Already configured (10 req/s)

Adjust in `Caddyfile`:
```caddy
rate_limit {
    zone vllama {
        key {remote_host}
        events 10
        window 1s
    }
}
```

### Input Validation

vllama validates:
- Model names (must be valid HuggingFace format)
- Token limits (max 4096 tokens)
- Request size (max 10MB via reverse proxy)

For additional validation, use reverse proxy or API gateway.

### Network Security

**Firewall (ufw):**
```bash
# Allow only HTTPS and SSH
sudo ufw default deny incoming
sudo ufw default allow outgoing
sudo ufw allow ssh
sudo ufw allow 443/tcp
sudo ufw enable
```

**Internal-only vllama:**
```bash
# Bind vllama to localhost only
vllama serve --host 127.0.0.1 --port 11434
```

Then use reverse proxy for public access.

## Performance Tuning

### GPU Memory

**For 7B models:**
```bash
--gpu-memory-utilization 0.9
```

**For smaller models (<2B):**
```bash
--gpu-memory-utilization 0.5
```

**Symptom of too low:** "No available memory for cache blocks"

### Concurrency

**High traffic (100+ concurrent):**
```bash
--max-num-seqs 512
```

**Medium traffic (10-50 concurrent):**
```bash
--max-num-seqs 256
```

**Low traffic (<10 concurrent):**
```bash
--max-num-seqs 64
```

### Config File

Avoid repetitive flags:

`~/.config/vllama/config.toml`:
```toml
[server]
host = "0.0.0.0"
port = 11434

[model]
default_model = "Qwen/Qwen2.5-7B-Instruct"
gpu_memory_utilization = 0.9
max_num_seqs = 256

[logging]
level = "info"
json = true
```

### Kernel Tuning

For high concurrency:

```bash
# Increase file descriptors
echo "* soft nofile 65536" | sudo tee -a /etc/security/limits.conf
echo "* hard nofile 65536" | sudo tee -a /etc/security/limits.conf

# Increase connection backlog
echo "net.core.somaxconn = 1024" | sudo tee -a /etc/sysctl.conf
sudo sysctl -p
```

## Troubleshooting

### vllama won't start

**Check logs:**
```bash
# Docker
docker logs vllama

# Systemd
sudo journalctl -u vllama -n 100
```

**Common issues:**
- **"CUDA not available":** Install NVIDIA drivers and CUDA toolkit
- **"Model not found":** Check model name spelling, try `vllama pull <model>`
- **"Port already in use":** Another process using port 11434 (`lsof -i:11434`)
- **"Permission denied":** Check file permissions, run as correct user

### Slow inference

**Check GPU utilization:**
```bash
nvidia-smi
```

Should show 80-90% utilization during inference.

**If low (<50%):**
- Increase `--gpu-memory-utilization` to 0.9 for 7B models
- Increase `--max-num-seqs` for better batching
- Check CPU bottlenecks (`htop`)

### Out of memory

**GPU OOM:**
```
RuntimeError: CUDA out of memory
```

Solutions:
- Use smaller model (7B → 1.5B)
- Increase GPU memory: `--gpu-memory-utilization 0.95`
- Reduce concurrency: `--max-num-seqs 128`

**System OOM:**
```
Killed by signal 9 (SIGKILL)
```

Solutions:
- Increase system RAM (16GB minimum for 7B models)
- Reduce model size
- Add swap (not recommended for performance)

### Connection refused

**Check if running:**
```bash
curl http://localhost:11434/health
```

**If failed:**
- Check service status: `sudo systemctl status vllama`
- Check port binding: `ss -tulpn | grep 11434`
- Check firewall: `sudo ufw status`

### SSL certificate errors

**Let's Encrypt failure:**
```bash
# Check DNS
dig llm.example.com

# Test certificate manually
sudo certbot certonly --standalone -d llm.example.com

# Check nginx config
sudo nginx -t
```

## Production Checklist

Before going live:

- [ ] SSL/TLS configured (Let's Encrypt)
- [ ] Authentication enabled (basic auth or API keys)
- [ ] Rate limiting configured (10-20 req/s per IP)
- [ ] Monitoring setup (health checks, logs)
- [ ] Backups configured (config files, not models)
- [ ] Firewall rules configured (only HTTPS + SSH)
- [ ] Resource limits tested (concurrent load)
- [ ] Error handling tested (OOM, network failures)
- [ ] Documentation written (for team)
- [ ] Rollback plan documented

## Next Steps

- [docs/MONITORING.md](MONITORING.md) - Prometheus + Grafana setup
- [docs/PERFORMANCE.md](PERFORMANCE.md) - Benchmarking and optimization
- [docs/MODELS.md](MODELS.md) - Model compatibility guide
- [ai/STATUS.md](../ai/STATUS.md) - Project status and roadmap

## Support

- **Issues:** https://github.com/nijaru/vllama/issues
- **Discussions:** https://github.com/nijaru/vllama/discussions
- **Documentation:** https://github.com/nijaru/vllama/tree/main/docs
