#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ACCEPTANCE_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

for script in "$SCRIPT_DIR"/*.sh; do
  bash -n "$script"
  [[ -x "$script" ]] || {
    echo "acceptance script is not executable: $script" >&2
    exit 1
  }
done
jq empty "$ACCEPTANCE_ROOT/evidence/schema.json"
terraform -chdir="$ACCEPTANCE_ROOT/environment" fmt -check -recursive

# Validate in a disposable copy so provider initialization never writes a
# lock/cache artifact into the source tree.
validate_root="$(mktemp -d "${TMPDIR:-/tmp}/axiom-gcp-operator-validate.XXXXXX")"
cleanup_validate() {
  find "$validate_root" -type f -delete
  find "$validate_root" -depth -type d -empty -delete
}
trap cleanup_validate EXIT INT TERM
mkdir -p "$validate_root/environment"
cp "$ACCEPTANCE_ROOT/environment"/*.tf "$validate_root/environment/"
TF_DATA_DIR="$validate_root/.terraform" terraform -chdir="$validate_root/environment" \
  init -backend=false -input=false
TF_DATA_DIR="$validate_root/.terraform" terraform -chdir="$validate_root/environment" validate
