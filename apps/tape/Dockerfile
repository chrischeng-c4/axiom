# Multi-stage build of the single tape binary. Build context is the repo
# root (tape is a workspace member); see Dockerfile.dockerignore for the
# trimmed context. HA is auto-mode: the same entrypoint serves single-node
# and raft-replica pods — REPLICAS_PER_SHARD > 1 flips HA.
FROM rust:1 AS build
WORKDIR /src
COPY --chown=root:root . .
RUN cargo build --release -p tape --bin tape

FROM debian:bookworm-slim
RUN useradd -m -u 10001 tape
COPY --chown=tape:tape --from=build /src/target/release/tape /usr/local/bin/tape
USER tape
EXPOSE 7137
ENTRYPOINT ["/usr/local/bin/tape"]
