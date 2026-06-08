#!/usr/bin/env bash
# Rebuild tdc and reinstall cclab's own .claude/skills/tdc-* + .claude/agents/tdc-handwrite.md
# from projects/tdc/templates/. Run this after editing any file under templates/.
#
# Usage:
#   ./scripts/tdc-refresh-self.sh            # debug build, --force install
#   ./scripts/tdc-refresh-self.sh --release  # release build (slower, optimized)

set -euo pipefail

cd "$(dirname "$0")/.."

PROFILE_FLAG=""
if [[ "${1:-}" == "--release" ]]; then
  PROFILE_FLAG="--release"
fi

echo "[1/2] cargo build -p tdc-cli ${PROFILE_FLAG}"
cargo build -p tdc-cli ${PROFILE_FLAG}

if [[ "${PROFILE_FLAG}" == "--release" ]]; then
  TDC_BIN="target/release/tdc"
else
  TDC_BIN="target/debug/tdc"
fi

echo "[2/2] ${TDC_BIN} setup --force --dir ."
"${TDC_BIN}" setup --force --dir .

echo
echo "✔ refreshed .claude/skills/tdc-*/ and .claude/agents/tdc-handwrite.md from templates/"
