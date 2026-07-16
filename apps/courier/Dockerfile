# syntax=docker/dockerfile:1
FROM python:3.11-slim

# Install uv
COPY --from=ghcr.io/astral-sh/uv:latest /uv /uvx /bin/

WORKDIR /app

# Install dependencies using uv sync (creates .venv)
COPY pyproject.toml uv.lock ./
RUN uv sync --frozen --no-install-project

# Copy app code
COPY app/ app/

# Add virtualenv to PATH
ENV PATH="/app/.venv/bin:$PATH"

# Cloud Run injects PORT, default to 8080
ENV PORT=8080
EXPOSE 8080

CMD uvicorn app.main:app --host 0.0.0.0 --port $PORT
