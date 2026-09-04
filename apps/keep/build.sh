#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: apps/keep/build.sh <debug|release>

debug    Build keep and install target/debug/keep to ~/.cargo/bin/keep.
release  Prepare the local release candidate: read the version from
         apps/keep/Cargo.toml, pin apps/keep/k8s/{base,operator} to
         keep:<version> and apps/keep/Dockerfile.release to keep@<version>,
         build with --release --locked, and install locally. It neither bumps
         the version nor commits.

Cross-platform release binaries and the digest-pinned candidate image are
built by keep-release-candidate.yml after `git land`; promotion to
keep@<version> is keep-release.yml (see the build-release skill). This command
never creates a tag or pushes a branch.
USAGE
}

fail_hint() {
  local mode="$1"
  echo ""
  echo "Build failed."
  echo "Retry with: apps/keep/build.sh ${mode}"
  echo "Verify with: ~/.cargo/bin/keep --version"
}

MODE="${1:-}"
case "$MODE" in
  debug|release) ;;
  -h|--help|help|"") usage; exit 0 ;;
  *) usage >&2; exit 2 ;;
esac
if [[ $# -gt 1 ]]; then
  usage >&2
  exit 2
fi

ROOT="$(git -c core.fsmonitor=false rev-parse --show-toplevel)"
cd "$ROOT"
. scripts/project-build-lib.sh
trap 'fail_hint "$MODE"' ERR

# Every manifest below carries exactly one bare `image: keep:<tag>` pin. The
# name stays registry-free on purpose: k8s/overlays/template and the GKE
# harness override the image by that name (`kustomize edit set image keep=…`),
# and a registry-qualified pin would silently stop matching that override.
KEEP_IMAGE_PINS=(
  apps/keep/k8s/base/statefulset.yaml
  apps/keep/k8s/operator/deployment.yaml
)

# `keep dockerfile render --variant release` substitutes CARGO_PKG_VERSION into
# exactly these three lines of the committed Dockerfile.release and must
# reproduce the file byte-for-byte (`cargo test -p keep --test deploy_cli`), so
# a Cargo.toml bump without this sync turns that test red; this sync is the fix.
KEEP_DOCKERFILE=apps/keep/Dockerfile.release

install_keep() {
  local profile="$1"
  install -d "$HOME/.cargo/bin"
  install -m 755 "target/${profile}/keep" "$HOME/.cargo/bin/keep"
  if command -v codesign >/dev/null 2>&1; then
    codesign -s - -f "$HOME/.cargo/bin/keep" >/dev/null 2>&1 || true
  fi
  echo "Installed: $HOME/.cargo/bin/keep"
  echo "Verify with: ~/.cargo/bin/keep --version"
}

sync_keep_release_image_pins() {
  local version="$1"; shift
  local manifest matches
  for manifest in "$@"; do
    matches="$(awk '/^[[:space:]]*image:[[:space:]]*keep:[^[:space:]]+$/ { count++ } END { print count + 0 }' "$manifest")"
    if [[ "$matches" -ne 1 ]]; then
      echo "error: expected exactly one keep image pin in ${manifest}; found ${matches}" >&2
      return 1
    fi
    KEEP_RELEASE_IMAGE_VERSION="$version" perl -0pi -e 's#(^[[:space:]]*image:[[:space:]]*keep:)\S+$#$1$ENV{KEEP_RELEASE_IMAGE_VERSION}#m' "$manifest"
    if ! grep -Eq "^[[:space:]]*image: keep:${version//./\\.}$" "$manifest"; then
      echo "error: failed to pin ${manifest} to keep ${version}" >&2
      return 1
    fi
  done
}

sync_keep_release_dockerfile() {
  local version="$1" dockerfile="$2"
  KEEP_RELEASE_VERSION="$version" perl -pi -e '
    s#^ARG KEEP_VERSION=.*$#ARG KEEP_VERSION=keep\@$ENV{KEEP_RELEASE_VERSION}#;
    s#^\#   docker build -f apps/keep/Dockerfile.release -t keep:.*$#\#   docker build -f apps/keep/Dockerfile.release -t keep:$ENV{KEEP_RELEASE_VERSION} \\#;
    s#^\#     --build-arg KEEP_VERSION=.*$#\#     --build-arg KEEP_VERSION=keep\@$ENV{KEEP_RELEASE_VERSION} .#;
  ' "$dockerfile"
  if ! grep -Fxq "ARG KEEP_VERSION=keep@${version}" "$dockerfile" \
    || ! grep -Fxq "#   docker build -f apps/keep/Dockerfile.release -t keep:${version} \\" "$dockerfile" \
    || ! grep -Fxq "#     --build-arg KEEP_VERSION=keep@${version} ." "$dockerfile"; then
    echo "error: failed to pin ${dockerfile} to keep@${version}" >&2
    return 1
  fi
}

case "$MODE" in
  debug)
    cargo build -p keep --bin keep
    install_keep debug
    echo "next: done"
    ;;
  release)
    CURRENT_VERSION="$(project_build_read_version apps/keep/Cargo.toml)"
    sync_keep_release_image_pins "$CURRENT_VERSION" "${KEEP_IMAGE_PINS[@]}"
    sync_keep_release_dockerfile "$CURRENT_VERSION" "$KEEP_DOCKERFILE"
    cargo build --release --locked -p keep --bin keep
    install_keep release
    echo "Local release preparation complete for keep@${CURRENT_VERSION}."
    echo "Next: git land main, then run build-release keep."
    ;;
esac
