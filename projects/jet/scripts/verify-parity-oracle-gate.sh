#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$repo_root"

echo "[jet parity gate] DOM oracle crate"
cargo test -p jet-parity-oracle -- --nocapture

echo "[jet parity gate] Live Chromium DOM oracle harness"
JET_PARITY_ORACLE_LIVE=1 cargo test -p jet-parity-oracle test_runner_live_chromium_smoke -- --nocapture

echo "[jet parity gate] Jet re-export smoke"
cargo test -p jet --test parity_oracle_reexport -- --nocapture
