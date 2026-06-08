#!/usr/bin/env sh
# <HANDWRITE gap="project-root-install-script" tracker="#4158" reason="project-specific repository installer dispatch contract">
set -eu

MODE="${VAT_BUILD_MODE:-debug}"

say() { printf 'vat-install: %s\n' "$*" >&2; }
die() { say "error: $*"; exit 1; }

case "${MODE}" in
  debug|release) ;;
  *) die "VAT_BUILD_MODE must be debug or release" ;;
esac

ROOT="$(git rev-parse --show-toplevel 2>/dev/null)" \
  || die "vat install currently requires a cloned cclab checkout"
cd "${ROOT}"

projects/vat/build.sh "${MODE}"
say "ready: $("$HOME/.cargo/bin/vat" --version 2>/dev/null || echo vat)"
# </HANDWRITE>
