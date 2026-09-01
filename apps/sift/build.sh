#!/usr/bin/env bash
# HANDWRITE-BEGIN gap="sift-build-entrypoint" tracker="1576" reason="Sift owns a Rustup-based debug/release build and local install entrypoint."
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: apps/sift/build.sh <debug|release>

debug    Build Sift and install target/debug/sift to ~/.cargo/bin/sift.
release  Build Sift with the release profile and install it locally.

Published release tags and cross-platform artifacts are a later release slice;
this command never creates a tag or pushes a branch.
EOF
}

mode="${1:-}"
case "$mode" in
  debug|release) ;;
  -h|--help|help|"") usage; exit 0 ;;
  *) usage >&2; exit 2 ;;
esac

root="$(git rev-parse --show-toplevel)"
cd "$root"
cargo_bin="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin/cargo"
if [[ ! -x "$cargo_bin" ]]; then
  echo "sift build: rustup stable-aarch64-apple-darwin cargo is required" >&2
  exit 1
fi

profile="debug"
args=(build -p sift --bin sift)
if [[ "$mode" == "release" ]]; then
  profile="release"
  args+=(--release)
fi
"$cargo_bin" "${args[@]}"

install -d "$HOME/.cargo/bin"
install -m 755 "target/$profile/sift" "$HOME/.cargo/bin/sift"
echo "installed: $($HOME/.cargo/bin/sift --version)"
echo "next: done"
# HANDWRITE-END
