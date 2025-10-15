"""
MAX Engine wrapper service for HyperLlama.

This provides a simple HTTP interface to MAX Engine's Python API,
allowing the Rust code to communicate with MAX via HTTP.
"""

import asyncio
import logging
from contextlib import asynccontextmanager
from typing import Dict, List, Optional

from fastapi import FastAPI, HTTPException
from fastapi.responses import StreamingResponse
from pydantic import BaseModel

try:
    from max.entrypoints.llm import LLM
    from max.pipelines import PipelineConfig
    MAX_AVAILABLE = True
except ImportError:
    MAX_AVAILABLE = False
    logging.warning("MAX Engine not available - install with: pip install modular")


logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)


class LoadModelRequest(BaseModel):
    model_path: str
    max_length: int = 32768
    device: str = "auto"  # "auto", "cpu", "cuda"


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


class EngineState:
    def __init__(self):
        self.models: Dict[str, LLM] = {}
        self.model_configs: Dict[str, Dict] = {}


@asynccontextmanager
async def lifespan(app: FastAPI):
    app.state.engine = EngineState()
    logger.info("MAX Engine service started")
    yield
    logger.info("MAX Engine service shutting down")


app = FastAPI(title="MAX Engine Service", lifespan=lifespan)


@app.get("/health")
async def health_check():
    return {
        "status": "healthy",
        "max_available": MAX_AVAILABLE,
    }


@app.post("/models/load")
async def load_model(request: LoadModelRequest):
    if not MAX_AVAILABLE:
        raise HTTPException(status_code=503, detail="MAX Engine not installed")

    engine: EngineState = app.state.engine
    model_id = request.model_path.replace("/", "_")

    if model_id in engine.models:
        return {"model_id": model_id, "status": "already_loaded"}

    try:
        logger.info(f"Loading model: {request.model_path} on device: {request.device}")
        config_kwargs = {
            "model_path": request.model_path,
            "max_length": request.max_length,
        }
        if request.device != "auto":
            config_kwargs["device"] = request.device

        pipeline_config = PipelineConfig(**config_kwargs)
        llm = LLM(pipeline_config)

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

        responses = llm.generate(
            [request.prompt],
            max_new_tokens=max_tokens,
        )

        full_text = responses[0]
        words = full_text.split()

        for i, word in enumerate(words):
            chunk = {
                "text": word + " " if i < len(words) - 1 else word,
                "done": i == len(words) - 1,
            }
            yield f"data: {json.dumps(chunk)}\n\n"

    except Exception as e:
        logger.error(f"Streaming generation failed: {e}")
        error_chunk = {"error": str(e), "done": True}
        yield f"data: {json.dumps(error_chunk)}\n\n"


@app.get("/models")
async def list_models():
    engine: EngineState = app.state.engine
    return {
        "models": [
            ModelInfo(
                model_id=model_id,
                model_path=config["model_path"],
                loaded=True,
            )
            for model_id, config in engine.model_configs.items()
        ]
    }


@app.post("/generate", response_model=GenerateResponse)
async def generate(request: GenerateRequest):
    if not MAX_AVAILABLE:
        raise HTTPException(status_code=503, detail="MAX Engine not installed")

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

        responses = llm.generate(
            [request.prompt],
            max_new_tokens=max_tokens,
        )

        response_text = responses[0]

        prompt_tokens = len(request.prompt.split())
        tokens_generated = len(response_text.split())

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
