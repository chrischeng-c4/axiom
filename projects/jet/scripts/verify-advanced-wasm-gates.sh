#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$repo_root"

echo "[jet advanced wasm gate] WASM build config and manifest"
cargo test -p jet --lib wasm_build:: -- --nocapture

echo "[jet advanced wasm gate] WASM runtime subset"
cargo test -p jet-wasm -- --nocapture

echo "[jet advanced wasm gate] Renderer target output"
cargo test -p jet-wasm renderer -- --nocapture

echo "[jet advanced wasm gate] WebGPU build default"
cargo test -p jet --test wasm_build_end_to_end wasm_build_selects_webgpu_scaffold_by_default -- --nocapture

echo "[jet advanced wasm gate] WebGPU runtime status and visual probe"
cargo test -p jet --test wasm_build_end_to_end webgpu_renderer_reports_runtime_status_and_visual_probe_when_available -- --nocapture

echo "[jet advanced wasm gate] Library WASM lowering fixtures"
cargo test -p jet --test tsx_to_rust_imports -- --nocapture

echo "[jet advanced wasm gate] MUI visual DOM/WASM parity"
cargo test -p jet --test mui_visual_regression mui_visual_fixture_renders_on_react_dom_and_jet_wasm -- --nocapture

echo "[jet advanced wasm gate] DOM oracle parity skeleton"
projects/jet/scripts/verify-parity-oracle-gate.sh
