#!/bin/bash
# HANDWRITE-BEGIN gap="missing-generator:logic:eb968e58" tracker="#2507" reason="Run native XCUI tests and reject false-green or incomplete Xcode result bundles."
set -euo pipefail

if [[ "$#" -ne 0 ]]; then
    echo "usage: $0" >&2
    exit 64
fi

workbench_script_dir="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
workbench_root="$(CDPATH= cd -- "${workbench_script_dir}/../../../.." && pwd)"
workbench_artifact_root="${workbench_root}/.axiom-workbench/test-artifacts/ui-tests"
workbench_project="${workbench_root}/apps/workbench/macos/WorkbenchMac.xcodeproj"
workbench_core_bin="${workbench_root}/target/debug/workbench-core"

mkdir -p "${workbench_artifact_root}"
workbench_run_dir="$(mktemp -d "${workbench_artifact_root}/native-e2e.XXXXXX")"
workbench_result_bundle="${workbench_run_dir}/WorkbenchMacUITests.xcresult"
workbench_build_log="${workbench_run_dir}/xcodebuild.log"
workbench_summary="${workbench_run_dir}/summary.json"

cd "${workbench_root}"
cargo build -p workbench --bin workbench-core

echo "Native E2E artifacts: ${workbench_run_dir}"
WORKBENCH_CORE_BIN="${workbench_core_bin}" xcodebuild \
    -project "${workbench_project}" \
    -scheme WorkbenchMac \
    -configuration Debug \
    -destination "platform=macOS,arch=arm64" \
    -resultBundlePath "${workbench_result_bundle}" \
    -test-timeouts-enabled YES \
    -maximum-test-execution-time-allowance 120 \
    test 2>&1 | tee "${workbench_build_log}"

if [[ ! -d "${workbench_result_bundle}" ]]; then
    echo "native E2E failed: result bundle was not produced" >&2
    echo "build log: ${workbench_build_log}" >&2
    exit 65
fi

xcrun xcresulttool get test-results summary \
    --path "${workbench_result_bundle}" \
    --compact > "${workbench_summary}"

workbench_total_tests="$(/usr/bin/plutil -extract totalTestCount raw -o - "${workbench_summary}")"
workbench_failed_tests="$(/usr/bin/plutil -extract failedTests raw -o - "${workbench_summary}")"
workbench_result="$(/usr/bin/plutil -extract result raw -o - "${workbench_summary}")"

if [[ "${workbench_total_tests}" -le 0 ]]; then
    echo "native E2E failed: result bundle contains zero executed tests" >&2
    echo "result bundle: ${workbench_result_bundle}" >&2
    exit 66
fi

if [[ "${workbench_failed_tests}" -ne 0 || "${workbench_result}" != "Passed" ]]; then
    echo "native E2E failed: ${workbench_failed_tests}/${workbench_total_tests} failed (${workbench_result})" >&2
    echo "result bundle: ${workbench_result_bundle}" >&2
    exit 67
fi

echo "Native E2E passed: ${workbench_total_tests} executed, ${workbench_failed_tests} failed"
echo "Result bundle: ${workbench_result_bundle}"
echo "Summary: ${workbench_summary}"
# HANDWRITE-END
