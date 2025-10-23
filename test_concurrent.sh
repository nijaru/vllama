#!/bin/bash
# Quick concurrent performance test for vLLama

echo ""
echo "vLLama Concurrent Performance Test"
echo "Model: facebook/opt-125m"
echo "Max tokens: 50"
echo "Optimizations: chunked-prefill ✓, prefix-caching ✓, 16384 batched tokens"
echo ""

# Function to make a single request
make_request() {
    curl -s -X POST http://localhost:11434/api/generate \
        -H "Content-Type: application/json" \
        -d '{
            "model": "facebook/opt-125m",
            "prompt": "Explain quantum computing in simple terms",
            "stream": false,
            "options": {"max_tokens": 50}
        }' > /dev/null
}

# Export function so it's available to subshells
export -f make_request

# Test concurrent requests
test_concurrent() {
    local N=$1
    echo "========================================================"
    echo "Testing $N concurrent requests..."
    echo "========================================================"

    local start=$(date +%s.%N)

    # Run N requests in parallel
    seq 1 $N | xargs -P $N -I {} bash -c 'make_request'

    local end=$(date +%s.%N)
    local total=$(echo "$end - $start" | bc)

    echo "✓ Completed in ${total}s"
    echo "  Throughput: $(echo "scale=2; $N / $total" | bc) req/s"
    echo ""

    # Store result for comparison
    eval "TIME_${N}=$total"
}

# Run tests
test_concurrent 5
sleep 2
test_concurrent 10
sleep 2
test_concurrent 50

echo "========================================================"
echo "SUMMARY"
echo "========================================================"
printf "%-12s %-12s\n" "Concurrent" "Total Time"
echo "--------------------------------------------------------"
printf "%-12s %-12s\n" "5" "${TIME_5}s"
printf "%-12s %-12s\n" "10" "${TIME_10}s"
printf "%-12s %-12s\n" "50" "${TIME_50}s"

echo ""
echo "========================================================"
echo "COMPARISON TO OLD BENCHMARK (before optimizations)"
echo "========================================================"
echo "5 concurrent:"
echo "  Old: 7.57s"
echo "  New: ${TIME_5}s"

# Check if faster
if (( $(echo "$TIME_5 < 7.57" | bc -l) )); then
    speedup=$(echo "scale=2; 7.57 / $TIME_5" | bc)
    echo "  ✓ ${speedup}x FASTER!"
else
    slowdown=$(echo "scale=2; $TIME_5 / 7.57" | bc)
    echo "  ✗ ${slowdown}x slower"
fi

echo ""
echo "Target: <3.0s for 5 concurrent (2x faster than Ollama's 6.50s)"
if (( $(echo "$TIME_5 < 3.0" | bc -l) )); then
    echo "✓ TARGET ACHIEVED!"
else
    diff=$(echo "scale=2; $TIME_5 - 3.0" | bc)
    echo "✗ Need ${diff}s improvement"
fi
