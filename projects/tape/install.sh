#!/usr/bin/env sh
# SPEC-MANAGED: projects/tape/tech-design/semantic/tape-install-script.md#logic
# <HANDWRITE gap="missing-generator:project-bootstrap" tracker="#768" reason="Initial Tape installer wrapper matching the ecosystem release-asset convention.">
set -eu

REPO="${TAPE_REPO:-chrischeng-c4/axiom}"
INSTALL_DIR="${TAPE_INSTALL:-$HOME/.local/bin}"
VERSION="${TAPE_VERSION:-latest}"
TOKEN="${GH_TOKEN:-${GITHUB_TOKEN:-}}"

say() { printf 'tape-install: %s\n' "$*" >&2; }
die() { say "error: $*"; exit "${2:-1}"; }
need() { command -v "$1" >/dev/null 2>&1 || die "missing required tool: $1" 3; }

need curl
need tar
need uname

if [ -z "${TOKEN}" ] && command -v gh >/dev/null 2>&1; then
  if gh auth status >/dev/null 2>&1; then
    TOKEN="$(gh auth token 2>/dev/null || true)"
  fi
fi

auth_curl_api() {
  if [ -n "${TOKEN}" ]; then
    curl -fsSL -H "Authorization: Bearer ${TOKEN}" -H "Accept: application/vnd.github+json" "$@"
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

case "$(uname -s)" in
  Darwin) os=apple-darwin ;;
  Linux) os=unknown-linux-gnu ;;
  *) die "unsupported OS: $(uname -s)" 2 ;;
esac

case "$(uname -m)" in
  x86_64|amd64) arch=x86_64 ;;
  arm64|aarch64) arch=aarch64 ;;
  *) die "unsupported arch: $(uname -m)" 2 ;;
esac

target="${arch}-${os}"
if [ "${target}" = "x86_64-apple-darwin" ]; then
  die "tape provides no Intel-macOS binary. Use an arm64 Mac, or run the Linux binary." 2
fi

if [ "${VERSION}" = "latest" ]; then
  api="https://api.github.com/repos/${REPO}/releases?per_page=30"
  tag="$(
    auth_curl_api "${api}" \
      | grep -E '"tag_name": "tape@[^"]+"' \
      | head -n 1 \
      | sed -E 's/.*"tag_name": "([^"]+)".*/\1/'
  )" || true
  [ -n "${tag}" ] || die "could not find a tape@* release in ${REPO}"
else
  tag="${VERSION}"
fi

asset="tape-${target}.tar.gz"
url="https://github.com/${REPO}/releases/download/${tag}/${asset}"
sha_url="${url}.sha256"
tmpdir="$(mktemp -d 2>/dev/null || mktemp -d -t tape-install)"
trap 'rm -rf "${tmpdir}"' EXIT INT TERM

say "downloading ${url}"
auth_curl "${url}" -o "${tmpdir}/${asset}" || die "download failed: ${url}"

if auth_curl "${sha_url}" -o "${tmpdir}/${asset}.sha256" 2>/dev/null; then
  expected="$(cat "${tmpdir}/${asset}.sha256")"
  if command -v shasum >/dev/null 2>&1; then
    actual="$(shasum -a 256 "${tmpdir}/${asset}" | awk '{print $1}')"
  elif command -v sha256sum >/dev/null 2>&1; then
    actual="$(sha256sum "${tmpdir}/${asset}" | awk '{print $1}')"
  else
    actual=""
  fi
  [ -z "${actual}" ] || [ "${actual}" = "${expected}" ] || die "sha256 mismatch"
fi

tar -C "${tmpdir}" -xzf "${tmpdir}/${asset}" || die "extract failed: ${asset}"
bin="${tmpdir}/tape-${target}/tape"
[ -f "${bin}" ] || die "binary not found in archive: ${bin}"
chmod +x "${bin}"

mkdir -p "${INSTALL_DIR}"
mv "${bin}" "${INSTALL_DIR}/tape"
say "installed: ${INSTALL_DIR}/tape"

if "${INSTALL_DIR}/tape" --version >/dev/null 2>&1; then
  say "ready: $("${INSTALL_DIR}/tape" --version 2>/dev/null || echo unknown)"
fi
# </HANDWRITE>
