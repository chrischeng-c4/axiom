#!/usr/bin/env sh
# jet installer — downloads the right prebuilt binary from GitHub
# Releases and drops it on your PATH.
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/chrischeng-c4/axiom/main/projects/jet/install.sh | sh
#
# Env overrides:
#   JET_VERSION   tag to install (default: latest jet@* release, e.g. jet@0.1.0)
#   JET_INSTALL   install dir (default: $HOME/.local/bin)
#   JET_REPO      gh repo (default: chrischeng-c4/axiom)
#   GH_TOKEN      GitHub token for private-repo fetch (also: GITHUB_TOKEN).
#                 axiom is public so this is normally unnecessary; kept for
#                 forks/mirrors that are private. If unset and `gh` is logged
#                 in, `gh auth token` is used.
#
# Exit codes:
#   0  success
#   1  generic failure
#   2  unsupported OS / arch
#   3  missing curl / tar
set -eu

REPO="${JET_REPO:-chrischeng-c4/axiom}"
INSTALL_DIR="${JET_INSTALL:-$HOME/.local/bin}"
VERSION="${JET_VERSION:-latest}"
TOKEN="${GH_TOKEN:-${GITHUB_TOKEN:-}}"

say() { printf 'jet-install: %s\n' "$*" >&2; }
die() { say "error: $*"; exit "${2:-1}"; }

need() {
  command -v "$1" >/dev/null 2>&1 || die "missing required tool: $1" 3
}
need curl
need tar
need uname

# Transparent gh-auth fallback for private forks/mirrors.
if [ -z "${TOKEN}" ] && command -v gh >/dev/null 2>&1; then
  if gh auth status >/dev/null 2>&1; then
    TOKEN="$(gh auth token 2>/dev/null || true)"
  fi
fi

# Wrapped curl that adds the auth header iff we have one.
# Separate variants for the JSON API (Accept) vs binary downloads.
auth_curl_api() {
  if [ -n "${TOKEN}" ]; then
    curl -fsSL -H "Authorization: Bearer ${TOKEN}" \
                -H "Accept: application/vnd.github+json" "$@"
  else
    curl -fsSL -H "Accept: application/vnd.github+json" "$@"
  fi
}
auth_curl() {
  if [ -n "${TOKEN}" ]; then
    curl -fsSL -H "Authorization: Bearer ${TOKEN}" "$@"
  else
    curl -fsSL "$@"
  fi
}

# ---- detect platform ---------------------------------------------------
os_raw="$(uname -s)"
arch_raw="$(uname -m)"

case "${os_raw}" in
  Darwin) os=apple-darwin ;;
  Linux)  os=unknown-linux-gnu ;;
  *)      die "unsupported OS: ${os_raw}" 2 ;;
esac

case "${arch_raw}" in
  x86_64|amd64)        arch=x86_64 ;;
  arm64|aarch64)       arch=aarch64 ;;
  *)                   die "unsupported arch: ${arch_raw}" 2 ;;
esac

target="${arch}-${os}"

# jet ships Apple Silicon only for macOS (no x86_64-apple-darwin build).
if [ "${target}" = "x86_64-apple-darwin" ]; then
  die "jet provides no Intel-macOS binary (Apple Silicon only). Use an arm64 Mac, or run the Linux binary." 2
fi
say "detected target: ${target}"

# ---- resolve tag -------------------------------------------------------
if [ "${VERSION}" = "latest" ]; then
  # GitHub redirects /releases/latest to the highest release. Use the
  # API filter so we only pick `jet@*` tags (not mamba/lumen/etc).
  api="https://api.github.com/repos/${REPO}/releases?per_page=30"
  tag="$(
    auth_curl_api "${api}" \
      | grep -E '"tag_name": "jet@[^"]+"' \
      | head -n 1 \
      | sed -E 's/.*"tag_name": "([^"]+)".*/\1/'
  )" || true
  if [ -z "${tag}" ]; then
    if [ -z "${TOKEN}" ]; then
      die "could not find a jet@* release in ${REPO} (if the repo is private — export GH_TOKEN or \`gh auth login\`)"
    fi
    die "could not find a jet@* release in ${REPO}"
  fi
else
  tag="${VERSION}"
fi
say "installing tag: ${tag}"

# ---- download + verify -------------------------------------------------
asset="jet-${target}.tar.gz"
url="https://github.com/${REPO}/releases/download/${tag}/${asset}"
sha_url="${url}.sha256"

tmpdir="$(mktemp -d 2>/dev/null || mktemp -d -t jet-install)"
trap 'rm -rf "${tmpdir}"' EXIT INT TERM

say "downloading ${url}"
auth_curl "${url}" -o "${tmpdir}/${asset}" \
  || die "download failed: ${url}"

# Best-effort integrity check — if .sha256 is missing on a manual
# release we don't refuse to install.
if auth_curl "${sha_url}" -o "${tmpdir}/${asset}.sha256" 2>/dev/null; then
  expected="$(cat "${tmpdir}/${asset}.sha256")"
  if command -v shasum >/dev/null 2>&1; then
    actual="$(shasum -a 256 "${tmpdir}/${asset}" | awk '{print $1}')"
  elif command -v sha256sum >/dev/null 2>&1; then
    actual="$(sha256sum "${tmpdir}/${asset}" | awk '{print $1}')"
  else
    actual=""
  fi
  if [ -n "${actual}" ] && [ "${actual}" != "${expected}" ]; then
    die "sha256 mismatch (expected ${expected}, got ${actual})"
  fi
  say "sha256 verified"
fi

# ---- extract + install -------------------------------------------------
tar -C "${tmpdir}" -xzf "${tmpdir}/${asset}" \
  || die "extract failed: ${asset}"

bin="${tmpdir}/jet-${target}/jet"
[ -f "${bin}" ] || die "binary not found in archive: ${bin}"
chmod +x "${bin}"

mkdir -p "${INSTALL_DIR}"
mv "${bin}" "${INSTALL_DIR}/jet"
say "installed: ${INSTALL_DIR}/jet"

# ---- PATH hint ---------------------------------------------------------
case ":${PATH}:" in
  *":${INSTALL_DIR}:"*) ;;
  *)
    say "note: ${INSTALL_DIR} is not on \$PATH"
    say "      add this to your shell profile:"
    say "        export PATH=\"${INSTALL_DIR}:\$PATH\""
    ;;
esac

# ---- show version ------------------------------------------------------
if "${INSTALL_DIR}/jet" --version >/dev/null 2>&1; then
  ver="$("${INSTALL_DIR}/jet" --version 2>/dev/null || echo unknown)"
  say "ready: ${ver}"
fi
