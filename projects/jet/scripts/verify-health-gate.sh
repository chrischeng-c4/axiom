#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$repo_root"

echo "[jet health gate] Basic FE-on-DOM evidence packaging"
cargo test -p jet --lib package_copies_managed_serve_log_artifact -- --nocapture
cargo test -p jet --lib evidence_bundle_can_carry_serve_session -- --nocapture
cargo test -p jet --lib pm_report_mode_surfaces_top_level_artifacts_panel -- --nocapture

echo "[jet health gate] Basic FE-on-DOM toolchain"
projects/jet/scripts/verify-basic-dom-gates.sh

echo "[jet health gate] Advanced FE-on-WASM toolchain"
projects/jet/scripts/verify-advanced-wasm-gates.sh
