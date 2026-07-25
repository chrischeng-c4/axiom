# Multi-stage build of the single tape binary. Build context is the repo
# root (tape is a workspace member); see Dockerfile.dockerignore for the
# trimmed context. The image also runs the checked-in operator Deployment,
# so it includes the operator feature; the same entrypoint serves single-node
# and raft-replica pods — REPLICAS_PER_SHARD > 1 flips HA.
# Keep the compiler and runtime on the same Debian ABI baseline. The floating
# `rust:1` tag moved ahead of bookworm and produced a binary the bookworm
# runtime could not load (GLIBC_2.38+).
#
# `backup` is enabled for the same reason as `operator` (#2574): the operator
# renders a CronJob that runs `tape backup` from THIS image, and a build
# without the feature answers that with the "compiled without backup support"
# bail. The release image already carried it (the published binary is built
# with `operator backup self-update issue`) — this is the from-source build
# catching up, and matches defer's `--features operator,backup`.
FROM rust:1-bookworm AS build
WORKDIR /src
COPY --chown=root:root . .
RUN cargo build --release -p tape --bin tape --features operator,backup

FROM debian:bookworm-slim
# Debian already reserves the `tape` group. Reuse it explicitly rather than
# asking useradd to create it a second time, which makes the source image fail
# before either the server or the operator can start.
RUN useradd --uid 10001 --gid tape --create-home tape
COPY --chown=tape:tape --from=build /src/target/release/tape /usr/local/bin/tape
USER tape
EXPOSE 7137
ENTRYPOINT ["/usr/local/bin/tape"]
