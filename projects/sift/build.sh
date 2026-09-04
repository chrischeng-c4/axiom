#!/usr/bin/env bash
# HANDWRITE-BEGIN gap="sift-build-entrypoint" tracker="1576" reason="Sift owns a Rustup-based debug/release build and local install entrypoint."
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: projects/sift/build.sh <debug|release>

debug    Build Sift and install target/debug/sift to ~/.cargo/bin/sift.
release  Prepare the local release candidate: read the version from
         projects/sift/Cargo.toml, pin every projects/sift/k8s/** image to
         ghcr.io/chrischeng-c4/sift:<version>, build with --release --locked,
         and install locally. It neither bumps the version nor commits.

Cross-platform release binaries and the digest-pinned candidate image are
built by sift-release-candidate.yml after `git land`; promotion to
sift@<version> is sift-release.yml (see the build-release skill). This command
never creates a tag or pushes a branch.
EOF
}

fail_hint() {
  local mode="$1"
  echo ""
  echo "Build failed."
  echo "Retry with: projects/sift/build.sh ${mode}"
  echo "Verify with: ~/.cargo/bin/sift --version"
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

# Every manifest below carries exactly one Sift GHCR pin. `src/deploy.rs`
# substitutes the same `ghcr.io/chrischeng-c4/sift:<CARGO_PKG_VERSION>` string
# when it renders `--image` overrides, so a pin that drifts from Cargo.toml
# silently disables those overrides; `cargo test -p sift --test deployment_cli`
# is the tripwire and this sync is the fix.
SIFT_IMAGE_PINS=(
  projects/sift/k8s/collector/daemonset.yaml
  projects/sift/k8s/instances/dev.yaml
  projects/sift/k8s/operator/operator.yaml
  projects/sift/k8s/overlays/dev/sift.yaml
  projects/sift/k8s/overlays/prod/sift.yaml
  projects/sift/k8s/overlays/staging/sift.yaml
)

install_sift() {
  local profile="$1"
  install -d "$HOME/.cargo/bin"
  install -m 755 "target/${profile}/sift" "$HOME/.cargo/bin/sift"
  if command -v codesign >/dev/null 2>&1; then
    codesign -s - -f "$HOME/.cargo/bin/sift" >/dev/null 2>&1 || true
  fi
  echo "Installed: $HOME/.cargo/bin/sift"
  echo "Verify with: ~/.cargo/bin/sift --version"
}

sync_sift_release_image_pins() {
  local version="$1"; shift
  local manifest matches
  for manifest in "$@"; do
    matches="$(awk '/^[[:space:]]*image:[[:space:]]*ghcr\.io\/chrischeng-c4\/sift:[^[:space:]]+$/ { count++ } END { print count + 0 }' "$manifest")"
    if [[ "$matches" -ne 1 ]]; then
      echo "error: expected exactly one Sift GHCR image pin in ${manifest}; found ${matches}" >&2
      return 1
    fi
    SIFT_RELEASE_IMAGE_VERSION="$version" perl -0pi -e 's#(^[[:space:]]*image:[[:space:]]*ghcr\.io/chrischeng-c4/sift:)\S+$#$1$ENV{SIFT_RELEASE_IMAGE_VERSION}#m' "$manifest"
    if ! grep -Fq "image: ghcr.io/chrischeng-c4/sift:${version}" "$manifest"; then
      echo "error: failed to pin ${manifest} to Sift ${version}" >&2
      return 1
    fi
  done
}

case "$MODE" in
  debug)
    cargo build -p sift --bin sift
    install_sift debug
    echo "next: done"
    ;;
  release)
    CURRENT_VERSION="$(project_build_read_version projects/sift/Cargo.toml)"
    sync_sift_release_image_pins "$CURRENT_VERSION" "${SIFT_IMAGE_PINS[@]}"
    cargo build --release --locked -p sift --bin sift
    install_sift release
    echo "Local release preparation complete for sift@${CURRENT_VERSION}."
    echo "Next: git land main, then run build-release sift."
    ;;
esac
# HANDWRITE-END
