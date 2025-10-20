"""
vLLM Engine wrapper service for vLLama.

This provides a simple HTTP interface to vLLM's Python API,
allowing the Rust code to communicate with vLLM via HTTP.
"""

import asyncio
import logging
from contextlib import asynccontextmanager
from typing import Dict, List, Optional

from fastapi import FastAPI, HTTPException
from fastapi.responses import StreamingResponse
from pydantic import BaseModel

try:
    from vllm import SamplingParams
    from vllm.engine.async_llm_engine import AsyncLLMEngine
    from vllm.engine.arg_utils import AsyncEngineArgs
    from vllm.utils import random_uuid
    VLLM_AVAILABLE = True
except ImportError:
    VLLM_AVAILABLE = False
    logging.warning("vLLM not available - install with: uv pip install vllm")


logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)


class LoadModelRequest(BaseModel):
    model_path: str
    max_length: int = 32768
    device: str = "auto"


class GenerateRequest(BaseModel):
    model_id: str
    prompt: str
    max_tokens: Optional[int] = None
    temperature: float = 0.7
    top_p: float = 0.9
    stream: bool = False


class GenerateResponse(BaseModel):
    text: str
    tokens_generated: int
    prompt_tokens: int


class ModelInfo(BaseModel):
    model_id: str
    model_path: str
    loaded: bool
    size_vram: Optional[int] = None


class GPUStats(BaseModel):
    total_vram: int
    used_vram: int
    free_vram: int


def get_gpu_stats() -> Optional[GPUStats]:
    """Get GPU memory statistics if available."""
    try:
        import torch
        if torch.cuda.is_available():
            device = torch.device("cuda:0")
            total = torch.cuda.get_device_properties(device).total_memory
            reserved = torch.cuda.memory_reserved(device)
            allocated = torch.cuda.memory_allocated(device)
            free = total - reserved
            return GPUStats(
                total_vram=total,
                used_vram=allocated,
                free_vram=free
            )
    except Exception as e:
        logger.warning(f"Failed to get GPU stats: {e}")
    return None


class EngineState:
    def __init__(self):
        self.models: Dict[str, AsyncLLMEngine] = {}
        self.model_configs: Dict[str, Dict] = {}


@asynccontextmanager
async def lifespan(app: FastAPI):
    app.state.engine = EngineState()
    logger.info("vLLM Engine service started")
    yield
    logger.info("vLLM Engine service shutting down")


app = FastAPI(title="vLLM Engine Service", lifespan=lifespan)


@app.get("/health")
async def health_check():
    return {
        "status": "healthy",
        "vllm_available": VLLM_AVAILABLE,
    }


@app.post("/models/load")
async def load_model(request: LoadModelRequest):
    if not VLLM_AVAILABLE:
        raise HTTPException(status_code=503, detail="vLLM not installed")

    engine: EngineState = app.state.engine
    model_id = request.model_path.replace("/", "_")

    if model_id in engine.models:
        return {"model_id": model_id, "status": "already_loaded"}

    try:
        logger.info(f"Loading model: {request.model_path}")

        engine_args = AsyncEngineArgs(
            model=request.model_path,
            max_model_len=request.max_length,
            tensor_parallel_size=1,
            gpu_memory_utilization=0.9,
            max_num_seqs=256,  # Enable batching up to 256 concurrent sequences
            max_num_batched_tokens=8192,  # Batch up to 8K tokens
        )

        llm = AsyncLLMEngine.from_engine_args(engine_args)

        engine.models[model_id] = llm
        engine.model_configs[model_id] = {
            "model_path": request.model_path,
            "max_length": request.max_length,
        }

        logger.info(f"Model loaded successfully: {model_id}")
        return {"model_id": model_id, "status": "loaded"}

    except Exception as e:
        logger.error(f"Failed to load model: {e}")
        raise HTTPException(status_code=500, detail=f"Failed to load model: {str(e)}")


@app.post("/models/unload")
async def unload_model(model_id: str):
    engine: EngineState = app.state.engine

    if model_id not in engine.models:
        raise HTTPException(status_code=404, detail="Model not found")

    del engine.models[model_id]
    del engine.model_configs[model_id]

    logger.info(f"Model unloaded: {model_id}")
    return {"model_id": model_id, "status": "unloaded"}


async def generate_stream(request: GenerateRequest, engine: EngineState):
    """Stream generation tokens as Server-Sent Events."""
    llm = engine.models[request.model_id]
    max_tokens = request.max_tokens or 512

    try:
        import json

        sampling_params = SamplingParams(
            temperature=request.temperature,
            top_p=request.top_p,
            max_tokens=max_tokens,
        )

        request_id = random_uuid()
        results_generator = llm.generate(request.prompt, sampling_params, request_id)

        accumulated_text = ""
        async for output in results_generator:
            if output.outputs:
                new_text = output.outputs[0].text
                delta = new_text[len(accumulated_text):]
                accumulated_text = new_text

                if delta:
                    chunk = {
                        "text": delta,
                        "done": output.finished,
                    }
                    yield f"data: {json.dumps(chunk)}\n\n"

    except Exception as e:
        logger.error(f"Streaming generation failed: {e}")
        error_chunk = {"error": str(e), "done": True}
        yield f"data: {json.dumps(error_chunk)}\n\n"


@app.get("/models")
async def list_models():
    engine: EngineState = app.state.engine
    gpu_stats = get_gpu_stats()

    return {
        "models": [
            ModelInfo(
                model_id=model_id,
                model_path=config["model_path"],
                loaded=True,
            )
            for model_id, config in engine.model_configs.items()
        ],
        "gpu_stats": gpu_stats.dict() if gpu_stats else None,
    }


@app.post("/generate", response_model=GenerateResponse)
async def generate(request: GenerateRequest):
    if not VLLM_AVAILABLE:
        raise HTTPException(status_code=503, detail="vLLM not installed")

    engine: EngineState = app.state.engine

    if request.model_id not in engine.models:
        raise HTTPException(status_code=404, detail="Model not loaded")

    if request.stream:
        return StreamingResponse(
            generate_stream(request, engine),
            media_type="text/event-stream",
        )

    llm = engine.models[request.model_id]

    try:
        max_tokens = request.max_tokens or 512

        sampling_params = SamplingParams(
            temperature=request.temperature,
            top_p=request.top_p,
            max_tokens=max_tokens,
        )

        request_id = random_uuid()
        results_generator = llm.generate(request.prompt, sampling_params, request_id)

        final_output = None
        async for output in results_generator:
            final_output = output

        if not final_output or not final_output.outputs:
            raise HTTPException(status_code=500, detail="No output generated")

        response_text = final_output.outputs[0].text
        prompt_tokens = len(final_output.prompt_token_ids)
        tokens_generated = len(final_output.outputs[0].token_ids)

        return GenerateResponse(
            text=response_text,
            tokens_generated=tokens_generated,
            prompt_tokens=prompt_tokens,
        )

    except Exception as e:
        logger.error(f"Generation failed: {e}")
        raise HTTPException(status_code=500, detail=f"Generation failed: {str(e)}")


if __name__ == "__main__":
    import uvicorn

    uvicorn.run(
        "server:app",
        host="127.0.0.1",
        port=8100,
        log_level="info",
    )
