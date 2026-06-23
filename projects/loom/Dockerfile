# loom container image — carries loom + relay + keep so one image backs every
# role in a k8s deploy (controller / worker / job-controller / run-task, plus
# relay + keep). Built for the cluster arch (linux). Used by scripts/k8s-e2e.sh.
FROM rust:1-slim AS builder
WORKDIR /src
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev protobuf-compiler && rm -rf /var/lib/apt/lists/*
COPY . .
RUN cargo build --release -p loom -p relay -p keep

FROM debian:stable-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /src/target/release/loom /usr/local/bin/loom
COPY --from=builder /src/target/release/relay-server /usr/local/bin/relay-server
COPY --from=builder /src/target/release/keep /usr/local/bin/keep
ENTRYPOINT ["loom"]
