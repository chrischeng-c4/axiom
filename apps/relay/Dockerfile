# Multi-stage build of the single relay binary. Build context is the repo
# root (relay is a workspace member); see Dockerfile.dockerignore for the
# trimmed context. HA is auto-mode (#544): the same entrypoint serves
# single-node and raft-replica pods — REPLICAS_PER_SHARD > 1 flips HA.
# Builder and runtime must share a Debian release: the binary links the
# builder's glibc, and the runtime below is bookworm (glibc 2.36). Bare
# `rust:1` moved to trixie (glibc 2.41) and the image crashed on start with
# "GLIBC_2.38 not found" — caught by /build:debug on 2026-09-02.
FROM rust:1-bookworm AS build
# release (default; CI / GKE) or debug (/build:debug). Only these two values
# work: the binary is copied out of target/<profile>/.
ARG CARGO_PROFILE=release
WORKDIR /src
COPY --chown=root:root . .
RUN cargo build $([ "$CARGO_PROFILE" = release ] && echo --release) -p relay --bin relay \
 && cp "target/$CARGO_PROFILE/relay" /usr/local/bin/

FROM debian:bookworm-slim
RUN useradd -m -u 10001 relay
COPY --chown=relay:relay --from=build /usr/local/bin/relay /usr/local/bin/relay
USER relay
EXPOSE 7000
ENTRYPOINT ["/usr/local/bin/relay"]
