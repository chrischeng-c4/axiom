# SPEC-MANAGED: projects/lumen/tech-design/semantic/lumen-runtime-image.md#runtime-image
# CODEGEN-BEGIN
# syntax=docker/dockerfile:1
# From-source build for dev / CI. For production prefer `Dockerfile.release`,
# which downloads a published binary (far faster, no Rust toolchain, no big build
# context). Multi-stage: the distroless runtime carries only the binaries + a
# non-root user, not the toolchain.
#
# Note: this is a cargo-workspace build, so the build context must be the repo
# root (cargo needs every workspace member's Cargo.toml). A .dockerignore that
# excludes target/ and .git keeps that context sane.

# Match the host toolchain (1.92): the resolved lockfile pulls deps that require
# the edition2024 Cargo feature (stabilized in 1.85), so an older builder fails.
FROM rust:1.92-slim-bookworm AS builder
WORKDIR /src
# Only ca-certificates needed — lumen links no openssl (reqwest is dev-only), so
# no pkg-config / libssl-dev.
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY . .
# BuildKit cache mounts keep the cargo registry + target dir warm across builds,
# so a source edit doesn't rebuild every dependency. target/ is a cache mount
# (not persisted into the image layer), so copy the binaries out in the same RUN.
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/src/target \
    cargo build --release -p lumen --bin lumen --features otel \
 && cargo build --release -p lumen --features operator --bin lumen-operator \
 && cp target/release/lumen target/release/lumen-operator /usr/local/bin/

# distroless runtime: glibc + libgcc + CA certs + nonroot (uid 65532, matching
# the k8s securityContext). No openssl, no shell, no init shim — a single tokio
# binary handles SIGTERM (graceful drain) and spawns no children.
FROM gcr.io/distroless/cc-debian12:nonroot
COPY --from=builder /usr/local/bin/lumen          /usr/local/bin/lumen
COPY --from=builder /usr/local/bin/lumen-operator /usr/local/bin/lumen-operator
# 7373 = client API. The write log lives in NATS, not in this container.
EXPOSE 7373
ENTRYPOINT ["/usr/local/bin/lumen"]
CMD ["serve"]
# CODEGEN-END
