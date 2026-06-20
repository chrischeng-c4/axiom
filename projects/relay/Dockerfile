# Multi-stage build of the relay-raft node. Build context is the repo root
# (relay is a workspace member); see Dockerfile.dockerignore for the trimmed
# context. relay has no axiom-project dependencies, only third-party crates.
FROM rust:1 AS build
WORKDIR /src
COPY . .
RUN cargo build --release -p relay --bin relay-raft

FROM debian:bookworm-slim
RUN useradd -m -u 10001 relay
COPY --from=build /src/target/release/relay-raft /usr/local/bin/relay-raft
USER relay
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/relay-raft"]
