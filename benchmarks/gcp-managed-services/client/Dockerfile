FROM rust:1-bookworm AS build
WORKDIR /src
COPY Cargo.toml Cargo.lock* ./
COPY src ./src
RUN cargo build --release --locked

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --uid 65532 --create-home bench
COPY --chown=bench:bench --from=build /src/target/release/axiom-gcp-managed-bench /usr/local/bin/axiom-gcp-managed-bench
USER 65532:65532
ENTRYPOINT ["/usr/local/bin/axiom-gcp-managed-bench"]
