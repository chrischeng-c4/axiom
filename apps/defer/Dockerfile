# HANDWRITE-BEGIN gap="missing-generator:deployment:defer-image" tracker="#766" reason="Source-build image fixture rendered by defer dockerfile render."
FROM rust:1-bookworm AS build
# release (default; CI / GKE) or debug (build-debug). Only these two values
# work: the binary is copied out of target/<profile>/.
ARG CARGO_PROFILE=release
WORKDIR /src
COPY --chown=root:root . .
RUN cargo build $([ "$CARGO_PROFILE" = release ] && echo --release) -p defer --bin defer --features operator,backup \
 && cp "target/$CARGO_PROFILE/defer" /usr/local/bin/

FROM debian:bookworm-slim
RUN groupadd --gid 10001 defer \
    && useradd --uid 10001 --gid 10001 --create-home defer
COPY --chown=defer:defer --from=build /usr/local/bin/defer /usr/local/bin/defer
USER 10001:10001
EXPOSE 7141
ENTRYPOINT ["/usr/local/bin/defer"]
# HANDWRITE-END
