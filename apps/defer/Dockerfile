# HANDWRITE-BEGIN gap="missing-generator:deployment:defer-image" tracker="#766" reason="Source-build image fixture rendered by defer dockerfile render."
FROM rust:1-bookworm AS build
WORKDIR /src
COPY --chown=root:root . .
RUN cargo build --release -p defer --bin defer --features operator,backup

FROM debian:bookworm-slim
RUN groupadd --gid 10001 defer \
    && useradd --uid 10001 --gid 10001 --create-home defer
COPY --chown=defer:defer --from=build /src/target/release/defer /usr/local/bin/defer
USER 10001:10001
EXPOSE 7141
ENTRYPOINT ["/usr/local/bin/defer"]
# HANDWRITE-END
