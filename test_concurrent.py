#!/usr/bin/env python3
"""
Quick concurrent performance test for vLLama
Tests with 5, 10, 50 concurrent requests
"""

import asyncio
import httpx
import time
from typing import List

async def make_request(client: httpx.AsyncClient, prompt: str) -> float:
    """Make a single request and return latency"""
    start = time.time()
    response = await client.post(
        "http://localhost:11434/api/generate",
        json={
            "model": "facebook/opt-125m",
            "prompt": prompt,
            "stream": False,
            "options": {"max_tokens": 50}
        },
        timeout=30.0
    )
    latency = time.time() - start
    response.raise_for_status()
    return latency

async def benchmark_concurrent(num_requests: int) -> dict:
    """Run concurrent requests and measure performance"""
    print(f"\n{'='*60}")
    print(f"Testing {num_requests} concurrent requests...")
    print(f"{'='*60}")

    prompt = "Explain quantum computing in simple terms"

    async with httpx.AsyncClient() as client:
        start_time = time.time()

        # Create all tasks
        tasks = [make_request(client, prompt) for _ in range(num_requests)]

        # Run concurrently
        latencies = await asyncio.gather(*tasks)

        total_time = time.time() - start_time

    # Calculate stats
    avg_latency = sum(latencies) / len(latencies)
    min_latency = min(latencies)
    max_latency = max(latencies)

    results = {
        "num_requests": num_requests,
        "total_time": total_time,
        "avg_latency": avg_latency,
        "min_latency": min_latency,
        "max_latency": max_latency,
        "throughput": num_requests / total_time,
    }

    print(f"✓ Completed in {total_time:.2f}s")
    print(f"  Avg latency: {avg_latency:.3f}s")
    print(f"  Min latency: {min_latency:.3f}s")
    print(f"  Max latency: {max_latency:.3f}s")
    print(f"  Throughput: {results['throughput']:.2f} req/s")

    return results

async def main():
    print("\nvLLama Concurrent Performance Test")
    print("Model: facebook/opt-125m")
    print("Max tokens: 50")
    print("Optimizations: chunked-prefill ✓, prefix-caching ✓, 16384 batched tokens")

    results = []

    # Test different concurrency levels
    for n in [5, 10, 50]:
        result = await benchmark_concurrent(n)
        results.append(result)
        await asyncio.sleep(2)  # Pause between tests

    # Summary
    print(f"\n{'='*60}")
    print("SUMMARY")
    print(f"{'='*60}")
    print(f"{'Concurrent':<12} {'Total Time':<12} {'Throughput':<15}")
    print(f"{'-'*60}")
    for r in results:
        print(f"{r['num_requests']:<12} {r['total_time']:<12.2f} {r['throughput']:<15.2f}")

    print(f"\n{'='*60}")
    print("COMPARISON TO OLD BENCHMARK (before optimizations)")
    print(f"{'='*60}")
    print(f"5 concurrent:")
    print(f"  Old: 7.57s")
    print(f"  New: {results[0]['total_time']:.2f}s")
    if results[0]['total_time'] < 7.57:
        speedup = 7.57 / results[0]['total_time']
        print(f"  ✓ {speedup:.2f}x FASTER!")
    else:
        slowdown = results[0]['total_time'] / 7.57
        print(f"  ✗ {slowdown:.2f}x slower")

    print(f"\nTarget: <3.0s for 5 concurrent (2x faster than Ollama's 6.50s)")
    if results[0]['total_time'] < 3.0:
        print("✓ TARGET ACHIEVED!")
    else:
        print(f"✗ Need {results[0]['total_time'] - 3.0:.2f}s improvement")

if __name__ == "__main__":
    asyncio.run(main())
