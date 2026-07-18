# syntax=docker/dockerfile:1
# From-source build for dev / CI. For production prefer `Dockerfile.release`.
# Multi-stage: the distroless runtime carries only the binaries + a non-root user.

FROM rust:1.92-slim-bookworm AS builder
WORKDIR /src
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/src/target \
    cargo build --release -p beam --bin beam \
 && cp target/release/beam /usr/local/bin/

FROM gcr.io/distroless/cc-debian12:nonroot
COPY --from=builder /usr/local/bin/beam /usr/local/bin/beam
# 7373 = client API.
EXPOSE 7373
ENTRYPOINT ["/usr/local/bin/beam"]
CMD ["serve"]
