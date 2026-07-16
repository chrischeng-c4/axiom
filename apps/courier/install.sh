#!/usr/bin/env sh
# <HANDWRITE gap="project-root-install-script" tracker="#4158" reason="project-specific repository installer dispatch contract">
set -eu

MODE="${COURIER_BUILD_MODE:-debug}"

say() { printf 'courier-install: %s\n' "$*" >&2; }
die() { say "error: $*"; exit 1; }

case "${MODE}" in
  debug|release) ;;
  *) die "COURIER_BUILD_MODE must be debug or release" ;;
esac

ROOT="$(git rev-parse --show-toplevel 2>/dev/null)" \
  || die "courier install currently requires a cloned cclab checkout"
cd "${ROOT}"

apps/courier/build.sh "${MODE}"
say "ready: $("$HOME/.cargo/bin/courier" --version 2>/dev/null || echo courier)"
# </HANDWRITE>
