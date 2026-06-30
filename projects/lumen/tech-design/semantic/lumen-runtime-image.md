---
id: semantic-lumen-runtime-image
summary: Semantic coverage for "projects/lumen/runtime-image"
capability_refs:
  - id: "competitor-feature-parity"
    role: primary
    claim: "query-planner-boolean-eval-roaring-postings"
    coverage: partial
    rationale: "Semantic takeover coverage for existing source group `projects/lumen/runtime-image`."
fill_sections: [runtime-image, changes]
---

# Semantic TD: lumen/runtime-image

## Runtime Image
<!-- type: runtime-image lang: yaml -->

```yaml
runtime_image:
  format: dockerfile
  semantic_domain:
    key: "lumen/runtime-image"
    source_group: "projects/lumen/runtime-image"
    coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/lumen/Dockerfile"
        language: "dockerfile"
        ownership_state: "codegen"
        generator_primitives: ["runtime_image"]
        source_evidence_node:
          layer: "operations"
          ecosystem: "dockerfile"
          role: "dockerfile"
          section_type: "runtime-image"
          domain: "projects/lumen/runtime-image"
      - path: "projects/lumen/Dockerfile.release"
        language: "dockerfile"
        ownership_state: "codegen"
        generator_primitives: ["runtime_image"]
        source_evidence_node:
          layer: "operations"
          ecosystem: "dockerfile"
          role: "release-dockerfile"
          section_type: "runtime-image"
          domain: "projects/lumen/runtime-image"
  artifacts:
    - path: "projects/lumen/Dockerfile"
      kind: "dockerfile"
      content: |
        # SPEC-MANAGED: projects/lumen/tech-design/semantic/lumen-runtime-image.md#runtime-image
        # CODEGEN-BEGIN
        # syntax=docker/dockerfile:1
        # From-source build for dev / CI. For production prefer `Dockerfile.release`,
        # which downloads a published binary (far faster, no Rust toolchain, no big build
        # context). Multi-stage: the distroless runtime carries only the binaries + a
        # non-root user, not the toolchain.
        #
        # Note: this is a cargo-workspace build, so the build context must be the repo
        # root (cargo needs every workspace member's Cargo.toml). A .dockerignore that
        # excludes target/ and .git keeps that context sane.

        # Match the host toolchain (1.92): the resolved lockfile pulls deps that require
        # the edition2024 Cargo feature (stabilized in 1.85), so an older builder fails.
        FROM rust:1.92-slim-bookworm AS builder
        WORKDIR /src
        # Only ca-certificates needed — lumen links no openssl (reqwest is dev-only), so
        # no pkg-config / libssl-dev.
        RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
            && rm -rf /var/lib/apt/lists/*
        COPY . .
        # BuildKit cache mounts keep the cargo registry + target dir warm across builds,
        # so a source edit doesn't rebuild every dependency. target/ is a cache mount
        # (not persisted into the image layer), so copy the binaries out in the same RUN.
        RUN --mount=type=cache,target=/usr/local/cargo/registry \
            --mount=type=cache,target=/usr/local/cargo/git \
            --mount=type=cache,target=/src/target \
            cargo build --release -p lumen --bin lumen --features "otel operator relay-wal jieba" \
         && cp target/release/lumen /usr/local/bin/

        # distroless runtime: glibc + libgcc + CA certs + nonroot (uid 65532, matching
        # the k8s securityContext). No openssl, no shell, no init shim — a single tokio
        # binary handles SIGTERM (graceful drain) and spawns no children.
        FROM gcr.io/distroless/cc-debian12:nonroot
        COPY --from=builder /usr/local/bin/lumen /usr/local/bin/lumen
        # 7373 = client API. The write log lives in the broker, not in this container.
        EXPOSE 7373
        ENTRYPOINT ["/usr/local/bin/lumen"]
        CMD ["serve"]
        # CODEGEN-END
    - path: "projects/lumen/Dockerfile.release"
      kind: "dockerfile"
      content: |
        # SPEC-MANAGED: projects/lumen/tech-design/semantic/lumen-runtime-image.md#runtime-image
        # CODEGEN-BEGIN
        # syntax=docker/dockerfile:1
        # Production image for lumen — downloads a PUBLISHED release binary (no source
        # tree, no Rust toolchain) into a distroless runtime. Tiny + minimal attack
        # surface (no shell/apt/curl in the final image). The sibling `Dockerfile` is
        # the from-source build for dev / CI.
        #
        #   docker build -f projects/lumen/Dockerfile.release -t lumen:0.4.5 \
        #     --build-arg LUMEN_VERSION=lumen@0.4.5 .
        #
        # The image arch (BuildKit TARGETARCH) selects the matching linux tarball:
        #   --platform linux/amd64 → x86_64-unknown-linux-gnu
        #   --platform linux/arm64 → aarch64-unknown-linux-gnu

        # ---- stage 1: fetch + verify the release binary (needs shell + curl) --------
        FROM debian:bookworm-slim AS fetch
        ARG LUMEN_VERSION=lumen@0.4.5
        ARG LUMEN_REPO=chrischeng-c4/axiom
        ARG TARGETARCH
        RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl \
            && rm -rf /var/lib/apt/lists/*
        RUN set -eux; \
            case "${TARGETARCH}" in \
              amd64) t=x86_64-unknown-linux-gnu ;; \
              arm64) t=aarch64-unknown-linux-gnu ;; \
              *) echo "unsupported TARGETARCH=${TARGETARCH}" >&2; exit 1 ;; \
            esac; \
            base="https://github.com/${LUMEN_REPO}/releases/download/${LUMEN_VERSION}"; \
            curl -fsSL "${base}/lumen-${t}.tar.gz"        -o /tmp/lumen.tgz; \
            curl -fsSL "${base}/lumen-${t}.tar.gz.sha256" -o /tmp/lumen.sha256; \
            echo "$(cat /tmp/lumen.sha256)  /tmp/lumen.tgz" | sha256sum -c -; \
            tar -xzf /tmp/lumen.tgz -C /tmp --strip-components=1 "lumen-${t}/lumen"; \
            /tmp/lumen --version

        # ---- stage 2: distroless runtime — only the binary + CA certs, nonroot -------
        # cc-debian12 carries glibc + libgcc (lumen is a dynamically-linked glibc Rust
        # binary; openssl was removed so no libssl is needed). :nonroot = uid 65532.
        # A single tokio binary handles SIGTERM itself (graceful drain) and spawns no
        # children, so no tini/init shim is required.
        FROM gcr.io/distroless/cc-debian12:nonroot
        COPY --from=fetch /tmp/lumen /usr/local/bin/lumen
        # 7373 = client API. The write log lives in the broker, not in this container.
        EXPOSE 7373
        ENTRYPOINT ["/usr/local/bin/lumen"]
        CMD ["serve"]
        # CODEGEN-END

```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/lumen/Dockerfile"
    action: modify
    section: runtime-image
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: codegen
  - path: "projects/lumen/Dockerfile.release"
    action: modify
    section: runtime-image
    description: |
      Production release-binary image variant is covered by this runtime-image TD.
    impl_mode: codegen
```
