#!/usr/bin/env sh
# HANDWRITE-BEGIN gap="sift-install-entrypoint" tracker="1576" reason="Sift releases install a target-specific verified archive into the local PATH."
set -eu

repo="${SIFT_REPO:-chrischeng-c4/axiom}"
version="${SIFT_VERSION:-latest}"
install_dir="${SIFT_INSTALL:-$HOME/.local/bin}"

say() { printf 'sift-install: %s\n' "$*" >&2; }
die() { say "error: $1"; exit "${2:-1}"; }
need() { command -v "$1" >/dev/null 2>&1 || die "missing required tool: $1" 3; }
need curl
need tar

case "$(uname -s)" in
  Darwin) os=apple-darwin ;;
  Linux) os=unknown-linux-gnu ;;
  *) die "unsupported OS: $(uname -s)" 2 ;;
esac
case "$(uname -m)" in
  x86_64|amd64) arch=x86_64 ;;
  arm64|aarch64) arch=aarch64 ;;
  *) die "unsupported architecture: $(uname -m)" 2 ;;
esac
target="$arch-$os"

if [ "$version" = latest ]; then
  version="$(curl -fsSL -H 'Accept: application/vnd.github+json' "https://api.github.com/repos/$repo/releases?per_page=30" | grep -E '"tag_name": "sift@[^" ]+"' | head -n 1 | sed -E 's/.*"tag_name": "([^"]+)".*/\1/')" || true
  [ -n "$version" ] || die "no sift@ release found in $repo"
fi

asset="sift-$target.tar.gz"
base="https://github.com/$repo/releases/download/$version/$asset"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT INT TERM
curl -fsSL "$base" -o "$tmp/$asset" || die "download failed: $base"
if curl -fsSL "$base.sha256" -o "$tmp/$asset.sha256" 2>/dev/null; then
  expected="$(awk '{print $1}' "$tmp/$asset.sha256")"
  if command -v shasum >/dev/null 2>&1; then actual="$(shasum -a 256 "$tmp/$asset" | awk '{print $1}')"; else actual="$(sha256sum "$tmp/$asset" | awk '{print $1}')"; fi
  [ "$expected" = "$actual" ] || die "sha256 mismatch"
fi
tar -C "$tmp" -xzf "$tmp/$asset"
bin="$tmp/sift-$target/sift"
[ -f "$bin" ] || die "archive did not contain $bin"
mkdir -p "$install_dir"
install -m 755 "$bin" "$install_dir/sift"
say "installed $install_dir/sift"
printf 'next: done\n'
# HANDWRITE-END
