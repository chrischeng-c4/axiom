# syntax=docker/dockerfile:1
FROM python:3.11-slim

# Install uv
COPY --from=ghcr.io/astral-sh/uv:latest /uv /uvx /bin/

WORKDIR /app

# Install dependencies using uv sync
COPY pyproject.toml uv.lock ./
RUN uv sync --frozen --no-install-project --system

# Copy app code
COPY app/ app/

# Cloud Run injects PORT, default to 8080
ENV PORT=8080
EXPOSE 8080

CMD uvicorn app.main:app --host 0.0.0.0 --port $PORT
