# Multi-stage build of the relay binaries. Build context is the repo root
# (relay is a workspace member); see Dockerfile.dockerignore for the trimmed
# context. relay has no axiom-project dependencies, only third-party crates.
FROM rust:1 AS build
WORKDIR /src
COPY --chown=root:root . .
RUN cargo build --release -p relay --bin relay-raft --bin relay

FROM debian:bookworm-slim
RUN useradd -m -u 10001 relay
COPY --chown=relay:relay --from=build /src/target/release/relay-raft /usr/local/bin/relay-raft
COPY --chown=relay:relay --from=build /src/target/release/relay /usr/local/bin/relay
USER relay
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/relay-raft"]
