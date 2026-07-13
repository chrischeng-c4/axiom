FROM rust:1.96-bookworm AS build
WORKDIR /workspace
COPY . .
RUN cargo build --locked --release -p sift

FROM gcr.io/distroless/cc-debian12:nonroot
COPY --from=build /workspace/target/release/sift /usr/local/bin/sift
ENV SIFT_HOST=0.0.0.0 \
    SIFT_PORT=7380 \
    SIFT_DATA_DIR=/var/lib/sift \
    SIFT_LOG_FORMAT=json
VOLUME ["/var/lib/sift"]
EXPOSE 7380
USER nonroot:nonroot
ENTRYPOINT ["/usr/local/bin/sift"]
CMD ["serve"]

<!-- marker: sift-source-image-artifact path: projects/sift/Dockerfile reason: Provide the source-build Sift image contract. -->
