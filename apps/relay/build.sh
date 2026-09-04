#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: apps/relay/build.sh <debug|release>

debug    Build relay and install target/debug/relay to ~/.cargo/bin/relay.
release  Prepare the local release candidate: read the version from
         apps/relay/Cargo.toml, pin apps/relay/k8s/{base,operator} to
         relay:<version> and apps/relay/Dockerfile.release to relay@<version>,
         build with --release --locked, and install locally. It neither bumps
         the version nor commits.

Cross-platform release binaries and the digest-pinned candidate image are
built by relay-release-candidate.yml after `git land`; promotion to
relay@<version> is relay-release.yml (see the build-release skill). This command
never creates a tag or pushes a branch.
USAGE
}

fail_hint() {
  local mode="$1"
  echo ""
  echo "Build failed."
  echo "Retry with: apps/relay/build.sh ${mode}"
  echo "Verify with: ~/.cargo/bin/relay --version"
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

# Every manifest below carries exactly one bare `image: relay:<tag>` pin. The
# name stays registry-free on purpose: k8s/overlays/template and the GKE
# harness override the image by that name (`kustomize edit set image relay=…`),
# and a registry-qualified pin would silently stop matching that override.
RELAY_IMAGE_PINS=(
  apps/relay/k8s/base/statefulset.yaml
  apps/relay/k8s/operator/deployment.yaml
)

# `relay dockerfile render --variant release` substitutes CARGO_PKG_VERSION into
# exactly these three lines of the committed Dockerfile.release and must
# reproduce the file byte-for-byte (`cargo test -p relay --test deploy_cli`), so
# a Cargo.toml bump without this sync turns that test red; this sync is the fix.
RELAY_DOCKERFILE=apps/relay/Dockerfile.release

install_relay() {
  local profile="$1"
  install -d "$HOME/.cargo/bin"
  install -m 755 "target/${profile}/relay" "$HOME/.cargo/bin/relay"
  if command -v codesign >/dev/null 2>&1; then
    codesign -s - -f "$HOME/.cargo/bin/relay" >/dev/null 2>&1 || true
  fi
  echo "Installed: $HOME/.cargo/bin/relay"
  echo "Verify with: ~/.cargo/bin/relay --version"
}

sync_relay_release_image_pins() {
  local version="$1"; shift
  local manifest matches
  for manifest in "$@"; do
    matches="$(awk '/^[[:space:]]*image:[[:space:]]*relay:[^[:space:]]+$/ { count++ } END { print count + 0 }' "$manifest")"
    if [[ "$matches" -ne 1 ]]; then
      echo "error: expected exactly one relay image pin in ${manifest}; found ${matches}" >&2
      return 1
    fi
    RELAY_RELEASE_IMAGE_VERSION="$version" perl -0pi -e 's#(^[[:space:]]*image:[[:space:]]*relay:)\S+$#$1$ENV{RELAY_RELEASE_IMAGE_VERSION}#m' "$manifest"
    if ! grep -Eq "^[[:space:]]*image: relay:${version//./\\.}$" "$manifest"; then
      echo "error: failed to pin ${manifest} to relay ${version}" >&2
      return 1
    fi
  done
}

sync_relay_release_dockerfile() {
  local version="$1" dockerfile="$2"
  RELAY_RELEASE_VERSION="$version" perl -pi -e '
    s#^ARG RELAY_VERSION=.*$#ARG RELAY_VERSION=relay\@$ENV{RELAY_RELEASE_VERSION}#;
    s#^\#   docker build -f apps/relay/Dockerfile.release -t relay:.*$#\#   docker build -f apps/relay/Dockerfile.release -t relay:$ENV{RELAY_RELEASE_VERSION} \\#;
    s#^\#     --build-arg RELAY_VERSION=.*$#\#     --build-arg RELAY_VERSION=relay\@$ENV{RELAY_RELEASE_VERSION} .#;
  ' "$dockerfile"
  if ! grep -Fxq "ARG RELAY_VERSION=relay@${version}" "$dockerfile" \
    || ! grep -Fxq "#   docker build -f apps/relay/Dockerfile.release -t relay:${version} \\" "$dockerfile" \
    || ! grep -Fxq "#     --build-arg RELAY_VERSION=relay@${version} ." "$dockerfile"; then
    echo "error: failed to pin ${dockerfile} to relay@${version}" >&2
    return 1
  fi
}

case "$MODE" in
  debug)
    cargo build -p relay --bin relay
    install_relay debug
    echo "next: done"
    ;;
  release)
    CURRENT_VERSION="$(project_build_read_version apps/relay/Cargo.toml)"
    sync_relay_release_image_pins "$CURRENT_VERSION" "${RELAY_IMAGE_PINS[@]}"
    sync_relay_release_dockerfile "$CURRENT_VERSION" "$RELAY_DOCKERFILE"
    cargo build --release --locked -p relay --bin relay
    install_relay release
    echo "Local release preparation complete for relay@${CURRENT_VERSION}."
    echo "Next: git land main, then run build-release relay."
    ;;
esac
