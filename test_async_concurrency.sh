#!/bin/bash

echo "=== Testing AsyncLLMEngine Concurrent Performance ===" 
echo "5 concurrent requests to vLLama"

start_time=$(date +%s.%N)

for i in {1..5}; do
  (curl -s -X POST http://localhost:11434/api/generate \
    -H "Content-Type: application/json" \
    -d "{\"model\":\"Qwen/Qwen2.5-1.5B-Instruct\",\"prompt\":\"Explain quantum computing $i\",\"stream\":false,\"max_tokens\":50}" \
    > /tmp/async_concurrent_${i}.json 2>&1) &
done

wait

end_time=$(date +%s.%N)
duration=$(echo "$end_time - $start_time" | bc)

echo "Total time: ${duration}s for 5 concurrent requests"
echo "Average: $(echo "$duration / 5" | bc -l)s per request"

rm -f /tmp/async_concurrent_*.json
