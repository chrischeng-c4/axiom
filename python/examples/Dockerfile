FROM python:3.12-slim

# Install system dependencies
RUN apt-get update && apt-get install -y \
    gcc \
    g++ \
    libpq-dev \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy project files
COPY . /app

# Install Rust (required for building data-bridge)
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Install Python dependencies
RUN pip install --no-cache-dir \
    maturin \
    fastapi \
    uvicorn[standard] \
    pydantic[email] \
    opentelemetry-api \
    opentelemetry-sdk \
    opentelemetry-instrumentation-fastapi \
    opentelemetry-exporter-otlp-proto-grpc

# Build and install data-bridge
RUN maturin develop --release

# Expose port
EXPOSE 8000

# Run the application
CMD ["python", "examples/fastapi_otel_example.py"]
