# Model Compatibility

This document lists tested models and their compatibility with vLLama.

## Tested Models

All tests performed on RTX 4090 (24GB VRAM) with vLLM V1 engine.

### ✅ Fully Compatible

| Model | Size | GPU Util | Load Time | VRAM | KV Cache | Max Concurrency | Notes |
|-------|------|----------|-----------|------|----------|-----------------|-------|
| **Qwen/Qwen2.5-0.5B-Instruct** | 0.5B | 50% | ~1s | 0.9 GiB | 819K tokens | 25.01x | Smallest, fastest |
| **Qwen/Qwen2.5-1.5B-Instruct** | 1.5B | 50% | ~2s | 2.9 GiB | 277K tokens | 8.46x | Good balance |
| **Qwen/Qwen2.5-7B-Instruct** | 7B | **90%** | ~5s | 14.2 GiB | 88K tokens | 2.68x | Requires high GPU util |
| **mistralai/Mistral-7B-Instruct-v0.3** | 7B | **90%** | ~5s | 13.5 GiB | 47K tokens | 1.43x | Requires high GPU util |

### ⚠️ Requires Authentication

| Model | Status | How to Enable |
|-------|--------|---------------|
| **meta-llama/Llama-3.2-1B-Instruct** | Gated | Accept license + HF token |
| **meta-llama/Llama-3.2-3B-Instruct** | Gated | Accept license + HF token |
| **meta-llama/Llama-3.1-8B-Instruct** | Gated | Accept license + HF token |

To use Llama models:
1. Go to https://huggingface.co/meta-llama/Llama-3.2-1B-Instruct
2. Click "Agree and access repository"
3. Create a read token at https://huggingface.co/settings/tokens
4. Set environment variable: `export HF_TOKEN=hf_...`

## GPU Memory Requirements

### Small Models (0.5-1.5B)
- **GPU Utilization:** 50% (12 GB on RTX 4090)
- **Recommended:** Any GPU with 8GB+ VRAM
- **Use case:** Development, testing, low-latency inference

### Large Models (7B)
- **GPU Utilization:** 90% (22 GB on RTX 4090)
- **Recommended:** GPU with 24GB+ VRAM
- **Use case:** Production, best quality responses

> **Note:** 7B models fail with 50% GPU utilization due to insufficient KV cache memory. Always use 90% for 7B models.

## Model Selection Guide

### By Use Case

**Development & Testing:**
- Use Qwen 2.5 0.5B or 1.5B
- Fast iteration, low resource usage
- Good for prototyping

**Production API:**
- Use Qwen 2.5 7B or Mistral 7B
- Better quality responses
- Requires 24GB GPU

**High Throughput:**
- Smaller models (0.5B, 1.5B) support higher concurrency
- 0.5B: Up to 25x concurrent requests
- 7B: Up to 2-3x concurrent requests

### By Vendor

**Alibaba Qwen:**
- Open access (no authentication required)
- Multiple sizes: 0.5B, 1.5B, 7B
- Good instruction following
- Default sampling: `temperature=0.7, top_p=0.8, top_k=20, repetition_penalty=1.1`

**Mistral AI:**
- Open access (no authentication required)
- Well-optimized for inference
- Good for coding and chat
- Lower KV cache usage than Qwen 7B

**Meta Llama:**
- **Requires HuggingFace authentication**
- Industry standard
- High quality responses
- Authentication setup needed before use

## Performance Characteristics

### Context Length
All tested models support **32,768 token context** (32K tokens).

### Optimization Features
All models benefit from:
- ✅ Chunked prefill (16,384 batched tokens)
- ✅ Prefix caching (KV cache reuse)
- ✅ Flash Attention
- ✅ CUDA graph optimization

### Known Limitations

1. **7B models need 90% GPU utilization**
   - 50% utilization → Negative KV cache memory (fails)
   - 90% utilization → Sufficient KV cache (works)

2. **First request may be slow**
   - Model download time (if not cached)
   - torch.compile optimization (~2-10s)
   - Subsequent requests are fast

3. **Gated models require setup**
   - Llama models need HuggingFace token
   - License acceptance required
   - One-time setup per machine

## Testing Methodology

All models were tested with:
- **Hardware:** RTX 4090 (24GB), Fedora Linux
- **Engine:** vLLM V1 (v0.11.0)
- **Test prompt:** "What is 2+2?"
- **Validation:** Correct arithmetic response

Memory and concurrency metrics from vLLM engine logs.

## Troubleshooting

### "No available memory for the cache blocks"

**Problem:** 7B model fails to load with 50% GPU utilization.

**Solution:** Use `--gpu-memory-utilization 0.9` for 7B models:
```bash
vllama serve --model Qwen/Qwen2.5-7B-Instruct --gpu-memory-utilization 0.9
```

### "401 Client Error" or "GatedRepoError"

**Problem:** Trying to load a Llama model without authentication.

**Solution:** Set up HuggingFace token (see "Requires Authentication" section above).

### "vLLM server failed to start within 60 seconds"

**Problem:** Large models take time to download on first use.

**Explanation:** This is expected behavior:
- Qwen 7B: ~70s download + 5s load
- Mistral 7B: ~131s download + 5s load

The model will finish loading in the background. The vLLM server will be available on port 8100 even if vLLama wrapper times out.

## Future Model Support

vLLama supports any model compatible with vLLM. Untested but likely to work:
- Other Qwen 2.5 sizes (3B, 14B, 32B)
- Other Mistral variants (Mixtral, Mistral Nemo)
- Llama 3.1 (70B with multi-GPU)
- Phi-3 models
- Yi models

To test a new model:
```bash
vllama serve --model <huggingface-model-id> --gpu-memory-utilization 0.9
```

For models >7B, you may need to adjust `--gpu-memory-utilization` based on available VRAM.
