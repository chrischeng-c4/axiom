#!/usr/bin/env bash
set -euo pipefail

# HANDWRITE-BEGIN gap="missing-generator:unit-test:a87c6c67" tracker="2370" reason="The GKE harness needs a static regression oracle for Lumen-only phase selection until shell-control-flow generation exists."
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ACCEPTANCE_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
RUN_SCRIPT="$ACCEPTANCE_ROOT/scripts/run.sh"
RENDER_SCRIPT="$ACCEPTANCE_ROOT/scripts/render-manifests.sh"
ENVIRONMENT_DIR="$ACCEPTANCE_ROOT/environment"
CLEANUP_SCRIPT="$ACCEPTANCE_ROOT/scripts/cleanup.sh"
VERIFY_CLEAN_SCRIPT="$ACCEPTANCE_ROOT/scripts/verify-clean.sh"
SCHEMA="$ACCEPTANCE_ROOT/evidence/schema.json"

bash -n "$RUN_SCRIPT"
bash -n "$RENDER_SCRIPT"
bash -n "$CLEANUP_SCRIPT"
bash -n "$VERIFY_CLEAN_SCRIPT"
jq empty "$SCHEMA"

rg -F 'LUMEN_ONLY="${LUMEN_ONLY:-0}"' "$RUN_SCRIPT" >/dev/null
rg -F 'LUMEN_ONLY=1 requires LUMEN_IMAGE as an immutable @sha256 digest reference' "$RUN_SCRIPT" >/dev/null
rg -F 'LUMEN_PRIOR_ACCEPTANCE is not allowed' "$RUN_SCRIPT" >/dev/null
rg -F 'Lumen-only acceptance passed; mandatory cleanup runs on EXIT' "$RUN_SCRIPT" >/dev/null
rg -F 'if [[ "$LUMEN_ONLY" != "1" ]]; then' "$RUN_SCRIPT" >/dev/null
rg -F 'if [[ "$LUMEN_ONLY" != "1" ]]; then' "$RENDER_SCRIPT" >/dev/null
rg -F 'variable "lumen_only"' "$ENVIRONMENT_DIR/variables.tf" >/dev/null
rg -F 'var.lumen_only ? [] : ["sift/sift-backup"]' "$ENVIRONMENT_DIR/storage.tf" >/dev/null
rg -F '-var="lumen_only=$LUMEN_ONLY"' "$RUN_SCRIPT" >/dev/null
rg -F '-var="lumen_only=$LUMEN_ONLY"' "$CLEANUP_SCRIPT" >/dev/null
rg -F 'LUMEN_ONLY="$LUMEN_ONLY"' "$RUN_SCRIPT" >/dev/null
rg -F 'LUMEN_ONLY="$LUMEN_ONLY"' "$CLEANUP_SCRIPT" >/dev/null
jq -e '.properties.mode.enum == ["full", "lumen-only"]' "$SCHEMA" >/dev/null
jq -e '.properties.acceptance.required == ["lumen"]' "$SCHEMA" >/dev/null

lumen_bundle_line="$(rg -n -F 'kubectl kustomize "$MANIFEST_DIR/lumen/operator"' "$RENDER_SCRIPT" | cut -d: -f1)"
sift_manifest_line="$(rg -n -F 'cat > "$MANIFEST_DIR/sift/operator/kustomization.yaml"' "$RENDER_SCRIPT" | cut -d: -f1)"
[[ "$lumen_bundle_line" =~ ^[0-9]+$ && "$sift_manifest_line" =~ ^[0-9]+$ ]]
(( lumen_bundle_line < sift_manifest_line )) || {
  echo "Lumen bundles must render before the deferred Sift-only branch" >&2
  exit 1
}

lumen_exit_line="$(rg -n -F '>> Lumen-only acceptance passed; mandatory cleanup runs on EXIT' "$RUN_SCRIPT" | cut -d: -f1)"
sift_deploy_line="$(rg -n -F '"$SCRIPT_DIR/deploy.sh" sift' "$RUN_SCRIPT" | cut -d: -f1)"
[[ "$lumen_exit_line" =~ ^[0-9]+$ && "$sift_deploy_line" =~ ^[0-9]+$ ]]
(( lumen_exit_line < sift_deploy_line )) || {
  echo "Lumen-only terminal path must precede Sift deployment" >&2
  exit 1
}
# HANDWRITE-END
