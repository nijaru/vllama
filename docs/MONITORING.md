# vllama Monitoring Guide

**Comprehensive monitoring setup for production deployments**

This guide covers observability: metrics, logs, alerts, and dashboards.

## Table of Contents

- [Quick Start](#quick-start)
- [Health Endpoint](#health-endpoint)
- [JSON Logging](#json-logging)
- [Prometheus + Grafana](#prometheus--grafana)
- [GPU Monitoring](#gpu-monitoring)
- [Alerting](#alerting)
- [Log Aggregation](#log-aggregation)

## Quick Start

**With Docker Compose:**
```bash
docker compose --profile monitoring up -d
```

Access:
- **Grafana:** http://localhost:3000 (admin/admin)
- **Prometheus:** http://localhost:9090
- **vllama health:** http://localhost:11434/health

## Health Endpoint

vllama exposes `/health` for monitoring systems.

**Request:**
```bash
curl http://localhost:11434/health | jq
```

**Response:**
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

**Status codes:**
- `200 OK`: System healthy
- `503 Service Unavailable`: vLLM not responding

**Use with monitoring tools:**
```yaml
# Uptime Kuma
type: http
url: http://localhost:11434/health
interval: 60

# Prometheus blackbox_exporter
- job_name: 'vllama-health'
  metrics_path: /probe
  params:
    module: [http_2xx]
  static_configs:
    - targets:
      - http://localhost:11434/health
```

## JSON Logging

Enable structured logs for aggregation.

**Enable JSON logging:**
```bash
export VLLAMA_LOG_FORMAT=json
```

**Example log entry:**
```json
{
  "timestamp": "2025-10-29T10:30:00.123Z",
  "level": "INFO",
  "target": "vllama::server",
  "request_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "method": "POST",
  "uri": "/api/generate",
  "status": 200,
  "latency_ms": 234,
  "message": "request completed"
}
```

**Query with jq:**
```bash
# Filter by status code
docker logs vllama 2>&1 | grep -v "^[^{]" | jq 'select(.status >= 400)'

# Calculate average latency
docker logs vllama 2>&1 | grep -v "^[^{]" | jq -s 'map(.latency_ms) | add / length'

# Count requests by endpoint
docker logs vllama 2>&1 | grep -v "^[^{]" | jq -r '.uri' | sort | uniq -c
```

## Prometheus + Grafana

Complete monitoring stack for metrics and dashboards.

### Setup with Docker Compose

**1. Start monitoring stack:**
```bash
docker compose --profile monitoring up -d
```

**2. Access Grafana:**
- URL: http://localhost:3000
- Username: `admin`
- Password: `admin` (change on first login)

**3. Add Prometheus data source:**
Already configured in `monitoring/grafana/datasources/prometheus.yml`:
```yaml
apiVersion: 1
datasources:
  - name: Prometheus
    type: prometheus
    access: proxy
    url: http://prometheus:9090
    isDefault: true
```

**4. Import vllama dashboard:**
- Go to Dashboards → Import
- Upload `monitoring/grafana/dashboards/vllama-dashboard.json`
- Select Prometheus data source

### Manual Setup (Without Docker)

**1. Install Prometheus:**
```bash
# Ubuntu/Debian
sudo apt install prometheus

# Fedora
sudo dnf install prometheus
```

**2. Configure Prometheus:**
Edit `/etc/prometheus/prometheus.yml`:
```yaml
scrape_configs:
  - job_name: 'vllama'
    static_configs:
      - targets: ['localhost:11434']
    metrics_path: '/health'
    scrape_interval: 30s
```

**3. Restart Prometheus:**
```bash
sudo systemctl restart prometheus
```

**4. Install Grafana:**
```bash
# Ubuntu/Debian
sudo apt-get install -y software-properties-common
sudo add-apt-repository "deb https://packages.grafana.com/oss/deb stable main"
wget -q -O - https://packages.grafana.com/gpg.key | sudo apt-key add -
sudo apt-get update && sudo apt-get install grafana

# Fedora
sudo dnf install grafana
```

**5. Start Grafana:**
```bash
sudo systemctl enable grafana-server
sudo systemctl start grafana-server
```

## GPU Monitoring

Track GPU utilization, memory, temperature.

### NVIDIA DCGM Exporter (Recommended)

**1. Add to docker-compose.yml:**
```yaml
services:
  dcgm-exporter:
    image: nvcr.io/nvidia/k8s/dcgm-exporter:3.1.7-3.1.4-ubuntu20.04
    runtime: nvidia
    environment:
      - NVIDIA_VISIBLE_DEVICES=all
    ports:
      - "9400:9400"
    restart: unless-stopped
```

**2. Update Prometheus config:**
```yaml
scrape_configs:
  - job_name: 'nvidia-gpu'
    static_configs:
      - targets: ['dcgm-exporter:9400']
```

**3. GPU metrics available:**
- `DCGM_FI_DEV_GPU_UTIL`: GPU utilization (%)
- `DCGM_FI_DEV_MEM_COPY_UTIL`: Memory bandwidth utilization (%)
- `DCGM_FI_DEV_GPU_TEMP`: GPU temperature (°C)
- `DCGM_FI_DEV_POWER_USAGE`: Power usage (W)
- `DCGM_FI_DEV_FB_USED`: Frame buffer used (MB)
- `DCGM_FI_DEV_FB_FREE`: Frame buffer free (MB)

### nvidia-smi (Basic)

**Check GPU status:**
```bash
nvidia-smi
```

**Watch GPU in real-time:**
```bash
watch -n 1 nvidia-smi
```

**Query specific metrics:**
```bash
# GPU utilization
nvidia-smi --query-gpu=utilization.gpu --format=csv,noheader,nounits

# Memory usage
nvidia-smi --query-gpu=memory.used,memory.total --format=csv,noheader

# Temperature
nvidia-smi --query-gpu=temperature.gpu --format=csv,noheader
```

## Alerting

Get notified of problems before users complain.

### Prometheus Alerting Rules

**1. Create alert rules:**

`monitoring/rules/vllama-alerts.yml`:
```yaml
groups:
  - name: vllama
    interval: 30s
    rules:
      # Service down
      - alert: VllamaDown
        expr: up{job="vllama"} == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "vllama is down"
          description: "vllama has been down for more than 1 minute"

      # High latency
      - alert: HighLatency
        expr: histogram_quantile(0.99, sum(rate(http_request_duration_seconds_bucket[5m])) by (le)) > 5
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High request latency"
          description: "P99 latency is {{ $value }}s (threshold: 5s)"

      # GPU OOM risk
      - alert: GPUMemoryHigh
        expr: (DCGM_FI_DEV_FB_USED / DCGM_FI_DEV_FB_FREE) > 0.95
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "GPU memory usage high"
          description: "GPU memory is at {{ $value | humanizePercentage }} (threshold: 95%)"

      # System memory high
      - alert: SystemMemoryHigh
        expr: (node_memory_MemTotal_bytes - node_memory_MemAvailable_bytes) / node_memory_MemTotal_bytes > 0.9
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "System memory usage high"
          description: "System memory is at {{ $value | humanizePercentage }} (threshold: 90%)"
```

**2. Configure in Prometheus:**

Add to `prometheus.yml`:
```yaml
rule_files:
  - 'rules/*.yml'

alerting:
  alertmanagers:
    - static_configs:
        - targets: ['localhost:9093']
```

**3. Set up Alertmanager:**

`alertmanager.yml`:
```yaml
global:
  slack_api_url: 'https://hooks.slack.com/services/YOUR/WEBHOOK/URL'

route:
  group_by: ['alertname']
  group_wait: 10s
  group_interval: 10s
  repeat_interval: 1h
  receiver: 'slack'

receivers:
  - name: 'slack'
    slack_configs:
      - channel: '#alerts'
        title: 'vllama Alert'
        text: '{{ range .Alerts }}{{ .Annotations.description }}{{ end }}'
```

### Simple Health Check Monitoring

**Uptime monitoring (healthchecks.io):**
```bash
# Ping healthchecks.io on success
curl -m 10 https://hc-ping.com/YOUR-UUID-HERE \
  --data "$(curl -sf http://localhost:11434/health)"
```

**Add to cron:**
```bash
*/5 * * * * /usr/local/bin/vllama-health-check.sh
```

## Log Aggregation

Centralize logs from multiple instances.

### Option 1: Loki (Grafana Stack)

**1. Add Loki to docker-compose.yml:**
```yaml
services:
  loki:
    image: grafana/loki:latest
    ports:
      - "3100:3100"
    volumes:
      - loki-data:/loki
    command: -config.file=/etc/loki/local-config.yaml

  promtail:
    image: grafana/promtail:latest
    volumes:
      - /var/log:/var/log
      - /var/lib/docker/containers:/var/lib/docker/containers:ro
      - ./monitoring/promtail-config.yml:/etc/promtail/config.yml
    command: -config.file=/etc/promtail/config.yml
```

**2. Configure Grafana:**
Add Loki data source in Grafana UI:
- URL: `http://loki:3100`

**3. Query logs:**
```logql
{job="vllama"} | json | status >= 400
{job="vllama"} | json | latency_ms > 1000
```

### Option 2: ELK Stack (Elasticsearch, Logstash, Kibana)

For large-scale deployments.

**1. Configure Filebeat:**
```yaml
filebeat.inputs:
  - type: log
    enabled: true
    paths:
      - /var/log/vllama/*.log
    json.keys_under_root: true
    json.add_error_key: true

output.elasticsearch:
  hosts: ["localhost:9200"]
  index: "vllama-%{+yyyy.MM.dd}"
```

**2. Query in Kibana:**
```
status:>= 400 AND latency_ms:>1000
```

## Metrics Reference

### Health Endpoint Metrics

| Metric | Type | Description |
|--------|------|-------------|
| `status` | string | Overall status ("ok" or "error") |
| `vllm_status` | string | vLLM server status ("healthy" or "unhealthy") |
| `models[]` | array | List of loaded models |
| `gpu.utilization_percent` | number | GPU utilization (0-100) |
| `gpu.memory_used_mb` | number | GPU memory used (MB) |
| `gpu.memory_free_mb` | number | GPU memory free (MB) |
| `memory.used_mb` | number | System memory used (MB) |
| `memory.available_mb` | number | System memory available (MB) |
| `uptime_seconds` | number | Server uptime (seconds) |

### Request Logs (JSON)

| Field | Type | Description |
|-------|------|-------------|
| `request_id` | string | Unique request ID (UUID) |
| `method` | string | HTTP method (GET, POST) |
| `uri` | string | Request URI |
| `status` | number | HTTP status code |
| `latency_ms` | number | Request latency (milliseconds) |
| `timestamp` | string | ISO 8601 timestamp |
| `level` | string | Log level (INFO, WARN, ERROR) |

## Troubleshooting

### Prometheus not scraping

**Check targets:**
- Go to http://localhost:9090/targets
- Look for "DOWN" targets

**Common issues:**
- Firewall blocking port 11434
- vllama not exposing /health
- Network connectivity (Docker networks)

**Fix:**
```bash
# Test health endpoint manually
curl http://localhost:11434/health

# Check Docker network
docker network inspect vllama_default
```

### Grafana dashboard shows no data

**Check:**
1. Prometheus data source configured correctly
2. Prometheus scraping successfully (check /targets)
3. Time range in dashboard (top right)
4. Query syntax correct

**Test query in Prometheus:**
- Go to http://localhost:9090/graph
- Try: `up{job="vllama"}`

### GPU metrics missing

**Install NVIDIA DCGM:**
```bash
# Check if nvidia-smi works
nvidia-smi

# Add DCGM exporter to docker-compose.yml
# Restart: docker compose up -d
```

## Best Practices

1. **Health checks:** Monitor every 30-60s
2. **Retention:** Keep metrics for 15-30 days
3. **Alerts:** Set thresholds based on your SLA
4. **Logs:** Rotate daily, keep 7-30 days
5. **Dashboards:** One overview, multiple detailed
6. **Automation:** Alert on anomalies, not noise

## Next Steps

- [docs/DEPLOYMENT.md](DEPLOYMENT.md) - Production deployment guide
- [docs/PERFORMANCE.md](PERFORMANCE.md) - Performance optimization
- [docs/MODELS.md](MODELS.md) - Model compatibility
