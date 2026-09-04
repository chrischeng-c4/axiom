# syntax=docker/dockerfile:1
# From-source build for dev / CI (used by scripts/k8s-e2e.sh). For production
# prefer `Dockerfile.release`, which downloads a published binary (no Rust
# toolchain, smaller context). Rendered by `loom dockerfile render --variant
# source`.
#
# This is a cargo-workspace build, so the build context must be the repo root
# (cargo needs every workspace member's Cargo.toml). The root .dockerignore
# excludes target/ and .git to keep the context sane.
#
# One image backs every loom role (controller / worker / job-controller /
# run-task / schema-layer); it also carries relay + keep so a single image can
# stand up the whole data plane for e2e. Match the host toolchain (1.92): the
# resolved lockfile pulls deps that require the edition2024 Cargo feature.
FROM rust:1.92-slim-bookworm AS builder
# release (default; CI / GKE) or debug (build-debug — unoptimized, matches
# `cargo build` on the host). Only these two values work: the binaries are
# copied out of target/<profile>/.
ARG CARGO_PROFILE=release
WORKDIR /src
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev protobuf-compiler ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --chown=root:root . .
# BuildKit cache mounts keep the cargo registry + target dir warm across builds;
# target/ is a cache mount (not persisted into the layer), so copy the binaries
# out in the same RUN. Add `--features operator` to also bake in the k8s
# operator/CRD paths for an operator-role image. relay ships one binary,
# `relay` (its `relay-server` bin went away in #1239); copying the old name
# failed every build of this image until build-debug ran it on 2026-09-02.
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/src/target \
    cargo build $([ "$CARGO_PROFILE" = release ] && echo --release) -p loom -p relay -p keep \
 && cp "target/$CARGO_PROFILE/loom" "target/$CARGO_PROFILE/relay" "target/$CARGO_PROFILE/keep" /usr/local/bin/

# debian-slim (not distroless): the in-cluster `loom job-controller` shells
# kubectl to create Jobs (it uses the pod's ServiceAccount token automatically),
# so the runtime needs kubectl + a shell.
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl \
    && ARCH=$(dpkg --print-architecture) \
    && curl -fsSL -o /usr/local/bin/kubectl "https://dl.k8s.io/release/v1.31.3/bin/linux/${ARCH}/kubectl" \
    && chmod +x /usr/local/bin/kubectl \
    && apt-get purge -y curl && apt-get autoremove -y && rm -rf /var/lib/apt/lists/*
COPY --chown=65532:65532 --from=builder /usr/local/bin/loom /usr/local/bin/loom
COPY --chown=65532:65532 --from=builder /usr/local/bin/relay /usr/local/bin/relay
COPY --chown=65532:65532 --from=builder /usr/local/bin/keep /usr/local/bin/keep
# 7474 = controller control API; 7475 = schema-layer bidi edge (both h2c+HTTP/1).
EXPOSE 7474 7475
ENTRYPOINT ["loom"]
CMD ["controller"]
