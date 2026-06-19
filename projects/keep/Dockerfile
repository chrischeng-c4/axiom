# syntax=docker/dockerfile:1
# From-source build for dev / CI. For production prefer `Dockerfile.release`,
# which downloads a published binary (no Rust toolchain, smaller context).
#
# This is a cargo-workspace build, so the build context must be the repo root
# (cargo needs every workspace member's Cargo.toml). The root .dockerignore
# excludes target/ and .git to keep the context sane.
#
#   docker build -f projects/keep/Dockerfile -t keep:dev .

FROM rust:1.92-slim-bookworm AS builder
WORKDIR /src
# Only ca-certificates needed — keep links no openssl.
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY . .
# BuildKit cache mounts keep the cargo registry + target dir warm across builds.
# target/ is a cache mount (not persisted into the layer), so copy the binary
# out in the same RUN.
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/src/target \
    cargo build --release -p keep --bin keep \
 && cp target/release/keep /usr/local/bin/

# distroless runtime: glibc + libgcc + CA certs + nonroot (uid 65532, matching
# the k8s securityContext). No shell, no init shim — a single tokio binary
# handles SIGTERM (graceful drain) and spawns no children.
FROM gcr.io/distroless/cc-debian12:nonroot
COPY --from=builder /usr/local/bin/keep /usr/local/bin/keep
# 7117 = HTTP API (HTTP/1.1 + HTTP/2 cleartext).
EXPOSE 7117
ENTRYPOINT ["/usr/local/bin/keep"]
