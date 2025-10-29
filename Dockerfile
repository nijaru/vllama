# vllama Docker Image
# Optimized for Linux + NVIDIA GPU production deployments

FROM nvidia/cuda:12.1.0-runtime-ubuntu22.04

# Install system dependencies
RUN apt-get update && apt-get install -y \
    curl \
    build-essential \
    pkg-config \
    libssl-dev \
    python3.12 \
    python3.12-venv \
    python3-pip \
    && rm -rf /var/lib/apt/lists/*

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Install uv for Python package management
RUN curl -LsSf https://astral.sh/uv/install.sh | sh
ENV PATH="/root/.local/bin:${PATH}"

# Set working directory
WORKDIR /app

# Copy Python dependencies first (better caching)
COPY python/pyproject.toml python/uv.lock ./python/
RUN cd python && uv sync --extra vllm

# Copy Rust project files
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

# Build vllama in release mode
RUN cargo build --release

# Copy built binary to /usr/local/bin
RUN cp target/release/vllama /usr/local/bin/vllama

# Create directory for logs
RUN mkdir -p /var/log/vllama

# Expose ports
# 11434: vllama server (Ollama-compatible API)
# 8100: vLLM OpenAI server (internal)
EXPOSE 11434 8100

# Set environment variables
ENV RUST_LOG=info
ENV VLLAMA_LOG_FORMAT=json

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=60s --retries=3 \
    CMD curl -f http://localhost:11434/health || exit 1

# Default command
CMD ["vllama", "serve", "--host", "0.0.0.0", "--port", "11434"]
