# syntax=docker/dockerfile:1
# ── Stage 1: build ────────────────────────────────────────────────────────────
FROM rust:1.87-slim AS builder

# Build deps for OpenSSL / native-tls (reqwest uses rustls, but keep for flexibility)
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev musl-tools ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build

# Cache Cargo registry + deps separately from source
COPY Cargo.toml Cargo.lock ./
COPY apps/ apps/
COPY libs/ libs/
COPY crates/ crates/

# Strip projects/ workspace members — those crates aren't in this build context
RUN sed -i '/\"projects\//d' Cargo.toml

# Build only the courier binary in release mode
RUN cargo build --release -p courier

# ── Stage 2: runtime ──────────────────────────────────────────────────────────
FROM gcr.io/distroless/cc-debian12:nonroot

COPY --from=builder /build/target/release/courier /usr/local/bin/courier
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

# Cloud Run injects PORT; courier reads COURIER_BIND
ENV COURIER_BIND=0.0.0.0:8080
EXPOSE 8080

ENTRYPOINT ["/usr/local/bin/courier"]
