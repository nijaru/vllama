#!/bin/bash

# Check GPU VRAM availability before running tests

set -e

MIN_FREE_GB=${1:-2}  # Minimum free VRAM in GB (default: 2GB)

echo "Checking GPU VRAM availability..."

if ! command -v nvidia-smi &> /dev/null; then
    echo "⚠️  nvidia-smi not found - assuming CPU mode"
    exit 0
fi

# Get free VRAM in MiB
FREE_MIB=$(nvidia-smi --query-gpu=memory.free --format=csv,noheader,nounits | head -1)
FREE_GB=$(echo "scale=2; $FREE_MIB / 1024" | bc)

echo "Free VRAM: ${FREE_GB}GB"

# Check if enough VRAM is available
if (( $(echo "$FREE_GB < $MIN_FREE_GB" | bc -l) )); then
    echo "❌ Insufficient VRAM: ${FREE_GB}GB free, need ${MIN_FREE_GB}GB"
    echo ""
    echo "GPU processes using VRAM:"
    nvidia-smi --query-compute-apps=pid,process_name,used_memory --format=csv
    echo ""
    echo "Suggestions:"
    echo "  - Stop running models: curl -X POST http://localhost:8100/models/unload -d '{\"model_id\":\"MODEL_ID\"}'"
    echo "  - Kill GPU processes: kill -9 <PID>"
    echo "  - Lower gpu_memory_utilization in server.py (currently 0.9)"
    exit 1
fi

echo "✅ Sufficient VRAM available: ${FREE_GB}GB"
exit 0
