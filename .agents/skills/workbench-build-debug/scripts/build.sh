#!/bin/sh
set -eu

workbench_script_dir="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
exec "${workbench_script_dir}/../../workbench-build-beta/scripts/build.sh" "$@"
