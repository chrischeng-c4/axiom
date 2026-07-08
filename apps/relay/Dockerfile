# Multi-stage build of the single relay binary. Build context is the repo
# root (relay is a workspace member); see Dockerfile.dockerignore for the
# trimmed context. HA is auto-mode (#544): the same entrypoint serves
# single-node and raft-replica pods — REPLICAS_PER_SHARD > 1 flips HA.
FROM rust:1 AS build
WORKDIR /src
COPY --chown=root:root . .
RUN cargo build --release -p relay --bin relay

FROM debian:bookworm-slim
RUN useradd -m -u 10001 relay
COPY --chown=relay:relay --from=build /src/target/release/relay /usr/local/bin/relay
USER relay
EXPOSE 7000
ENTRYPOINT ["/usr/local/bin/relay"]
