#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$repo_root"

phases="${JET_BASIC_DOM_PHASES:-package,browser}"
build_samples="${JET_BASIC_DOM_BUILD_SAMPLES:-3}"
runtime_smoke="${JET_BASIC_DOM_RUNTIME_SMOKE:-required}"
command_timeout_ms="${JET_BASIC_DOM_COMMAND_TIMEOUT_MS:-120000}"
package_baselines="${JET_BASIC_DOM_PACKAGE_BASELINES:-npm,pnpm}"
require_package_baselines="${JET_BASIC_DOM_REQUIRE_BASELINES:-1}"
browser_baselines="${JET_BASIC_DOM_BROWSER_BASELINES:-playwright}"
require_browser_baselines="${JET_BASIC_DOM_REQUIRE_BROWSER_BASELINES:-1}"

usage() {
  cat <<'EOF'
usage: verify-basic-dom-gates.sh [--phase package,browser,build] [--all]

Phases:
  package    jet pkg management replacement contract
  browser    jet bb Browser Bridge replacement contract
  build      DOM production build corpus; intentionally third
  serve      jet serve/HMR unit gate
  workspace  workspace/task-runner unit gate
  test       native test/reporter unit gate
  e2e        product-flow e2e unit gate
  trace      trace evidence unit gate

Environment:
  JET_BASIC_DOM_PACKAGE_BASELINES=npm,pnpm selects isolated package baselines.
  JET_BASIC_DOM_REQUIRE_BASELINES=1 makes package baselines blocking.
  JET_BASIC_DOM_BROWSER_BASELINES=playwright selects isolated Browser Bridge baselines.
  JET_BASIC_DOM_REQUIRE_BROWSER_BASELINES=1 makes browser baselines blocking.
  JET_BASIC_DOM_COMMAND_TIMEOUT_MS=120000 sets child command timeout.
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --phase|--phases)
      phases="$2"
      shift 2
      ;;
    --all)
      phases="package,browser,build,serve,workspace,test,e2e,trace"
      shift
      ;;
    --build-samples)
      build_samples="$2"
      shift 2
      ;;
    --runtime-smoke)
      runtime_smoke="$2"
      shift 2
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      echo "[jet basic dom gate] unknown argument: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

phase_enabled() {
  [[ ",$phases," == *",all,"* || ",$phases," == *",$1,"* ]]
}

release_built=0
ensure_release_jet() {
  if [[ "$release_built" -eq 0 ]]; then
    cargo build -p jet --release
    release_built=1
  fi
}

echo "[jet basic dom gate] phases=$phases"

if phase_enabled package; then
  echo "[jet basic dom gate] 1 Package manager replacement"
  cargo test -p jet --lib pkg_manager -- --nocapture
  ensure_release_jet
  package_args=(
    --jet-bin target/release/jet
    --evidence /tmp/jet-basic-dom-gate/pkg-management-compare.json
    --baseline-tools "$package_baselines"
    --command-timeout-ms "$command_timeout_ms"
  )
  if [[ "$require_package_baselines" == "1" ]]; then
    package_args+=(--require-baselines)
  else
    package_args+=(--no-require-baselines)
  fi
  node projects/jet/scripts/compare-pkg-management.mjs "${package_args[@]}"
fi

if phase_enabled browser; then
  echo "[jet basic dom gate] 2 Browser Bridge replacement"
  cargo test -p jet --lib browser -- --nocapture
  ensure_release_jet
  browser_args=(
    --jet-bin target/release/jet
    --evidence /tmp/jet-basic-dom-gate/browser-bridge-replacement.json
    --timeout-ms "$command_timeout_ms"
    --baseline-timeout-ms "$command_timeout_ms"
  )
  if [[ -n "$browser_baselines" && "$browser_baselines" != "none" ]]; then
    browser_args+=(--baseline-tools "$browser_baselines")
  fi
  if [[ "$require_browser_baselines" == "1" ]]; then
    browser_args+=(--require-baselines)
  fi
  node projects/jet/scripts/verify-browser-bridge-replacement.mjs "${browser_args[@]}"
fi

if phase_enabled build; then
  echo "[jet basic dom gate] 3 Production build and bundler"
  ensure_release_jet
  if rg -n "try_esbuild_minify|node_modules.*\\.bin.*esbuild|Command::new\\(&esbuild\\)" projects/jet/src/cli.rs; then
    echo "[jet basic dom gate] ERROR: jet build must use Jet's internal minifier, not optional esbuild"
    exit 1
  fi
  cargo test -p jet --lib bundler -- --nocapture
  cargo test -p jet --lib transform -- --nocapture
  cargo test -p jet --lib asset -- --nocapture
  node projects/jet/scripts/compare-basic-builds.mjs \
    --self-test \
    --runtime-smoke "$runtime_smoke" \
    --require-css \
    --require-public brand.svg \
    --semantic-strings "DOM Production Assets"
  node projects/jet/scripts/compare-dom-build-corpus.mjs \
    --jet-bin target/release/jet \
    --runtime-smoke "$runtime_smoke" \
    --build-samples "$build_samples" \
    --out-dir /tmp/jet-basic-dom-gate \
    --evidence /tmp/jet-basic-dom-gate/basic-build-corpus.json
fi

if phase_enabled serve; then
  echo "[jet basic dom gate] Serve and HMR"
  cargo test -p jet --lib dev_server -- --nocapture
  ensure_release_jet
  node projects/jet/scripts/compare-prod-static-serve.mjs \
    --jet-bin target/release/jet \
    --out-dir /tmp/jet-basic-dom-gate/prod-static \
    --evidence /tmp/jet-basic-dom-gate/prod-static-serve.json
fi

if phase_enabled workspace; then
  echo "[jet basic dom gate] Workspace and task runner"
  cargo test -p jet --lib task_runner -- --nocapture
fi

if phase_enabled test; then
  echo "[jet basic dom gate] Native test runtime"
  cargo test -p jet --lib test_runner -- --nocapture
  cargo test -p jet --lib reporter -- --nocapture
fi

if phase_enabled e2e; then
  echo "[jet basic dom gate] Product-flow e2e"
  cargo test -p jet --lib e2e -- --nocapture
fi

if phase_enabled trace; then
  echo "[jet basic dom gate] Trace evidence"
  cargo test -p jet --lib trace -- --nocapture
fi
