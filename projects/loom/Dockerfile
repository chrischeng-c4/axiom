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
# kubectl: the in-cluster job-controller shells it to create Jobs (uses the pod's
# ServiceAccount token automatically). Arch-matched to the build.
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl \
    && ARCH=$(dpkg --print-architecture) \
    && curl -fsSL -o /usr/local/bin/kubectl "https://dl.k8s.io/release/v1.31.3/bin/linux/${ARCH}/kubectl" \
    && chmod +x /usr/local/bin/kubectl \
    && apt-get purge -y curl && apt-get autoremove -y && rm -rf /var/lib/apt/lists/*
COPY --from=builder /src/target/release/loom /usr/local/bin/loom
COPY --from=builder /src/target/release/relay-server /usr/local/bin/relay-server
COPY --from=builder /src/target/release/keep /usr/local/bin/keep
ENTRYPOINT ["loom"]
