#!/bin/sh
# HANDWRITE-BEGIN gap="missing-generator:contract:4731e2b3" tracker="#2445" reason="Build and restart only the beta native product."
set -eu

if [ "$#" -ne 0 ]; then
    echo "usage: $0" >&2
    exit 64
fi

workbench_script_dir="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
workbench_root="$(CDPATH= cd -- "${workbench_script_dir}/../../../.." && pwd)"
workbench_core_bin="${workbench_root}/target/debug/workbench-core"
workbench_project="${workbench_root}/apps/workbench/macos/WorkbenchMac.xcodeproj"

cd "${workbench_root}"
cargo build -p workbench --bin workbench-core
workbench_settings="$(WORKBENCH_CORE_BIN="${workbench_core_bin}" xcodebuild -project "${workbench_project}" -scheme WorkbenchMac -configuration Debug -showBuildSettings)"
workbench_build_dir="$(printf '%s\n' "${workbench_settings}" | awk -F ' = ' '/^[[:space:]]*TARGET_BUILD_DIR = / { print $2; exit }')"

if [ -z "${workbench_build_dir}" ]; then
    echo "could not determine Xcode TARGET_BUILD_DIR" >&2
    exit 65
fi

WORKBENCH_CORE_BIN="${workbench_core_bin}" xcodebuild -project "${workbench_project}" -scheme WorkbenchMac -configuration Debug build
workbench_app="${workbench_build_dir}/Axiom Workbench Beta.app"
workbench_process="${workbench_app}/Contents/MacOS/Axiom Workbench Beta"

if [ ! -d "${workbench_app}" ]; then
    echo "beta product was not produced at ${workbench_app}" >&2
    exit 66
fi
if pgrep -f "${workbench_process}" >/dev/null 2>&1; then
    pkill -f "${workbench_process}"
fi
open -n "${workbench_app}"
echo "Built and opened ${workbench_app}"
# HANDWRITE-END
