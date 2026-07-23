# Source candidate only. Published Tape releases use apps/tape/Dockerfile.release.
FROM rust:1.96-bookworm AS builder
WORKDIR /workspace
COPY . .
RUN cargo build --locked --release -p tape --bin tape --features "operator backup" \
 && install -D -m 0755 target/release/tape /out/tape

FROM gcr.io/distroless/cc-debian12:nonroot
COPY --from=builder /out/tape /usr/local/bin/tape
EXPOSE 7137
ENTRYPOINT ["/usr/local/bin/tape"]
CMD ["serve"]
