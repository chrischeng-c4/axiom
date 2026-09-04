#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: apps/defer/build.sh <debug|release>

debug    Build defer and install target/debug/defer to ~/.cargo/bin/defer.
release  Prepare the local release candidate: read the version from
         apps/defer/Cargo.toml, pin apps/defer/k8s/{base,operator} to
         defer:<version> and apps/defer/Dockerfile.release to defer@<version>,
         build with --release --locked, and install locally. It neither bumps
         the version nor commits.

Cross-platform release binaries and the digest-pinned candidate image are
built by defer-release-candidate.yml after `git land`; promotion to
defer@<version> is defer-release.yml (see the build-release skill). This command
never creates a tag or pushes a branch.
USAGE
}

fail_hint() {
  local mode="$1"
  echo ""
  echo "Build failed."
  echo "Retry with: apps/defer/build.sh ${mode}"
  echo "Verify with: ~/.cargo/bin/defer --version"
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

# Every manifest below carries exactly one bare `image: defer:<tag>` pin. The
# name stays registry-free on purpose: k8s/overlays/template and the GKE
# harness override the image by that name (`kustomize edit set image defer=…`),
# and a registry-qualified pin would silently stop matching that override.
DEFER_IMAGE_PINS=(
  apps/defer/k8s/base/statefulset.yaml
  apps/defer/k8s/operator/deployment.yaml
)

# `defer dockerfile render --variant release` substitutes CARGO_PKG_VERSION into
# exactly these three lines of the committed Dockerfile.release and must
# reproduce the file byte-for-byte (`cargo test -p defer --test deploy_cli`), so
# a Cargo.toml bump without this sync turns that test red; this sync is the fix.
DEFER_DOCKERFILE=apps/defer/Dockerfile.release

install_defer() {
  local profile="$1"
  install -d "$HOME/.cargo/bin"
  install -m 755 "target/${profile}/defer" "$HOME/.cargo/bin/defer"
  if command -v codesign >/dev/null 2>&1; then
    codesign -s - -f "$HOME/.cargo/bin/defer" >/dev/null 2>&1 || true
  fi
  echo "Installed: $HOME/.cargo/bin/defer"
  echo "Verify with: ~/.cargo/bin/defer --version"
}

sync_defer_release_image_pins() {
  local version="$1"; shift
  local manifest matches
  for manifest in "$@"; do
    matches="$(awk '/^[[:space:]]*image:[[:space:]]*defer:[^[:space:]]+$/ { count++ } END { print count + 0 }' "$manifest")"
    if [[ "$matches" -ne 1 ]]; then
      echo "error: expected exactly one defer image pin in ${manifest}; found ${matches}" >&2
      return 1
    fi
    DEFER_RELEASE_IMAGE_VERSION="$version" perl -0pi -e 's#(^[[:space:]]*image:[[:space:]]*defer:)\S+$#$1$ENV{DEFER_RELEASE_IMAGE_VERSION}#m' "$manifest"
    if ! grep -Eq "^[[:space:]]*image: defer:${version//./\\.}$" "$manifest"; then
      echo "error: failed to pin ${manifest} to defer ${version}" >&2
      return 1
    fi
  done
}

sync_defer_release_dockerfile() {
  local version="$1" dockerfile="$2"
  DEFER_RELEASE_VERSION="$version" perl -pi -e '
    s#^ARG DEFER_VERSION=.*$#ARG DEFER_VERSION=defer\@$ENV{DEFER_RELEASE_VERSION}#;
    s#^\#   docker build -f apps/defer/Dockerfile.release -t defer:.*$#\#   docker build -f apps/defer/Dockerfile.release -t defer:$ENV{DEFER_RELEASE_VERSION} \\#;
    s#^\#     --build-arg DEFER_VERSION=.*$#\#     --build-arg DEFER_VERSION=defer\@$ENV{DEFER_RELEASE_VERSION} .#;
  ' "$dockerfile"
  if ! grep -Fxq "ARG DEFER_VERSION=defer@${version}" "$dockerfile" \
    || ! grep -Fxq "#   docker build -f apps/defer/Dockerfile.release -t defer:${version} \\" "$dockerfile" \
    || ! grep -Fxq "#     --build-arg DEFER_VERSION=defer@${version} ." "$dockerfile"; then
    echo "error: failed to pin ${dockerfile} to defer@${version}" >&2
    return 1
  fi
}

case "$MODE" in
  debug)
    cargo build -p defer --bin defer
    install_defer debug
    echo "next: done"
    ;;
  release)
    CURRENT_VERSION="$(project_build_read_version apps/defer/Cargo.toml)"
    sync_defer_release_image_pins "$CURRENT_VERSION" "${DEFER_IMAGE_PINS[@]}"
    sync_defer_release_dockerfile "$CURRENT_VERSION" "$DEFER_DOCKERFILE"
    cargo build --release --locked -p defer --bin defer
    install_defer release
    echo "Local release preparation complete for defer@${CURRENT_VERSION}."
    echo "Next: git land main, then run build-release defer."
    ;;
esac
