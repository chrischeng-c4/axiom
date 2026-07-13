# Multi-stage build of the single tape binary. Build context is the repo
# root (tape is a workspace member); see Dockerfile.dockerignore for the
# trimmed context. The image also runs the checked-in operator Deployment,
# so it includes the operator feature; the same entrypoint serves single-node
# and raft-replica pods — REPLICAS_PER_SHARD > 1 flips HA.
# Keep the compiler and runtime on the same Debian ABI baseline. The floating
# `rust:1` tag moved ahead of bookworm and produced a binary the bookworm
# runtime could not load (GLIBC_2.38+).
FROM rust:1-bookworm AS build
WORKDIR /src
COPY --chown=root:root . .
RUN cargo build --release -p tape --bin tape --features operator

FROM debian:bookworm-slim
# Debian already reserves the `tape` group. Reuse it explicitly rather than
# asking useradd to create it a second time, which makes the source image fail
# before either the server or the operator can start.
RUN useradd --uid 10001 --gid tape --create-home tape
COPY --chown=tape:tape --from=build /src/target/release/tape /usr/local/bin/tape
USER tape
EXPOSE 7137
ENTRYPOINT ["/usr/local/bin/tape"]
