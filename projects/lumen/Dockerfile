# Minimal production-style image for lumen. Multi-stage so the runtime
# image carries only the binary + a non-root user, not the toolchain.

# Match the host toolchain (1.92): the resolved lockfile pulls deps that
# require the edition2024 Cargo feature (stabilized in 1.85), so an older
# builder fails to parse their manifests.
FROM rust:1.92-slim-bookworm AS builder
WORKDIR /src
RUN apt-get update && apt-get install -y --no-install-recommends \
      pkg-config libssl-dev ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY . .
RUN cargo build --release -p lumen --bin lumen
# The K8s Operator ships in the same image but is a separate build so the
# serving `lumen` binary stays free of the kube-rs tree (feature-gated).
RUN cargo build --release -p lumen --features operator --bin lumen-operator

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
      ca-certificates tini \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --system --gid 1000 lumen \
    && useradd  --system --uid 1000 --gid 1000 --home-dir /var/lib/lumen lumen \
    && mkdir -p /var/lib/lumen/data \
    && chown -R lumen:lumen /var/lib/lumen
COPY --from=builder /src/target/release/lumen /usr/local/bin/lumen
COPY --from=builder /src/target/release/lumen-operator /usr/local/bin/lumen-operator
USER lumen
WORKDIR /var/lib/lumen
# 7373 = client API. The write log lives in NATS, not in this container.
EXPOSE 7373
ENTRYPOINT ["/usr/bin/tini", "--", "/usr/local/bin/lumen"]
CMD ["serve"]
