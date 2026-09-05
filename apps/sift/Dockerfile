# HANDWRITE-BEGIN gap="sift-source-image-artifact" tracker="1606" reason="Provide the source-build Sift image contract."
FROM rust:1.96-bookworm AS build
ARG TARGETPLATFORM
WORKDIR /workspace
COPY --chown=65532:65532 . .
RUN --mount=type=cache,id=sift-cargo-registry,target=/usr/local/cargo/registry \
    --mount=type=cache,id=sift-cargo-git,target=/usr/local/cargo/git \
    --mount=type=cache,id=sift-cargo-target-${TARGETPLATFORM},target=/workspace/target \
    cargo build --locked --release -p sift \
    && install -D -m 0755 /workspace/target/release/sift /image-root/usr/local/bin/sift
RUN install -d -m 0700 -o 65532 -g 65532 /image-root/var/lib/sift

FROM gcr.io/distroless/cc-debian12:nonroot
COPY --chown=65532:65532 --from=build /image-root/usr/local/bin/sift /usr/local/bin/sift
COPY --from=build --chown=65532:65532 --chmod=0700 /image-root/var/lib/sift /var/lib/sift
ENV SIFT_HOST=0.0.0.0 \
    SIFT_PORT=7380 \
    SIFT_DATA_DIR=/var/lib/sift \
    SIFT_LOG_FORMAT=json
VOLUME ["/var/lib/sift"]
EXPOSE 4317 7380
USER nonroot:nonroot
ENTRYPOINT ["/usr/local/bin/sift"]
CMD ["serve"]
# HANDWRITE-END
