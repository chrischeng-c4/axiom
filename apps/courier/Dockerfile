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

# Keep only libs/, crates/, and apps/courier workspace members
RUN python3 -c "
import re, sys
text = open('Cargo.toml').read()
def keep(m):
    p = m.group(1)
    if p.startswith('libs/') or p.startswith('crates/') or p == 'apps/courier':
        return m.group(0)
    return ''
text = re.sub(r'^\s*\"([^\"]+)\",?\n', keep, text, flags=re.MULTILINE)
open('Cargo.toml', 'w').write(text)
"

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
