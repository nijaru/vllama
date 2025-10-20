#!/bin/bash

REQUESTS=${1:-10}
MODEL="Qwen/Qwen2.5-1.5B-Instruct"
OLLAMA_MODEL="qwen2.5:1.5b"

echo "=== Testing $REQUESTS Concurrent Requests ==="
echo ""

# Test vLLama
echo "Testing vLLama..."
start_time=$(date +%s.%N)

for i in $(seq 1 $REQUESTS); do
  (curl -s -X POST http://localhost:11434/api/generate \
    -H "Content-Type: application/json" \
    -d "{\"model\":\"$MODEL\",\"prompt\":\"Test $i\",\"stream\":false,\"max_tokens\":50}" \
    > /tmp/vllama_scale_${i}.json 2>&1) &
done

wait
end_time=$(date +%s.%N)
vllama_duration=$(echo "$end_time - $start_time" | bc)

echo "vLLama: ${vllama_duration}s total"
echo "vLLama: $(echo "$vllama_duration / $REQUESTS" | bc -l)s average"
echo ""

# Clean up
rm -f /tmp/vllama_scale_*.json

# Test Ollama
echo "Testing Ollama..."
start_time=$(date +%s.%N)

for i in $(seq 1 $REQUESTS); do
  (curl -s -X POST http://127.0.0.1:11435/api/generate \
    -H "Content-Type: application/json" \
    -d "{\"model\":\"$OLLAMA_MODEL\",\"prompt\":\"Test $i\",\"stream\":false,\"options\":{\"num_predict\":50}}" \
    > /tmp/ollama_scale_${i}.json 2>&1) &
done

wait
end_time=$(date +%s.%N)
ollama_duration=$(echo "$end_time - $start_time" | bc)

echo "Ollama: ${ollama_duration}s total"
echo "Ollama: $(echo "$ollama_duration / $REQUESTS" | bc -l)s average"
echo ""

# Clean up
rm -f /tmp/ollama_scale_*.json

# Calculate speedup
speedup=$(echo "$ollama_duration / $vllama_duration" | bc -l)
if (( $(echo "$speedup > 1" | bc -l) )); then
    echo "Result: vLLama ${speedup}x faster"
elif (( $(echo "$speedup < 1" | bc -l) )); then
    inverse=$(echo "1 / $speedup" | bc -l)
    echo "Result: Ollama ${inverse}x faster"
else
    echo "Result: Tied"
fi
