#!/bin/sh
# HANDWRITE-BEGIN gap="missing-generator:logic:workbench-xcode-core-embed" tracker="#2278" reason="Embed the real Rust sidecar in the native Xcode application product."
set -eu

if [ "$#" -ne 1 ]; then
    echo "usage: embed-workbench-core.sh <destination>" >&2
    exit 64
fi

workbench_destination="$1"
workbench_script_dir="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
workbench_repo_root="$(CDPATH= cd -- "${workbench_script_dir}/../../../.." && pwd)"

if [ -n "${WORKBENCH_CORE_BIN:-}" ]; then
    workbench_core_bin="${WORKBENCH_CORE_BIN}"
else
    workbench_cargo_bin="$(command -v cargo || true)"
    if [ -z "${workbench_cargo_bin}" ] && [ -x "${HOME}/.cargo/bin/cargo" ]; then
        workbench_cargo_bin="${HOME}/.cargo/bin/cargo"
    fi
    if [ -z "${workbench_cargo_bin}" ]; then
        echo "cargo is unavailable; set WORKBENCH_CORE_BIN to a built workbench-core" >&2
        exit 69
    fi
    "${workbench_cargo_bin}" build \
        --manifest-path "${workbench_repo_root}/Cargo.toml" \
        --target-dir "${workbench_repo_root}/target" \
        -p workbench \
        --bin workbench-core
    workbench_core_bin="${workbench_repo_root}/target/debug/workbench-core"
fi

if [ ! -x "${workbench_core_bin}" ]; then
    echo "workbench-core is not executable: ${workbench_core_bin}" >&2
    exit 66
fi

/bin/mkdir -p "$(dirname -- "${workbench_destination}")"
/bin/cp "${workbench_core_bin}" "${workbench_destination}"
/bin/chmod 755 "${workbench_destination}"
# HANDWRITE-END
