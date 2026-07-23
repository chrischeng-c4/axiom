#!/usr/bin/env bash
set -euo pipefail

# HANDWRITE-BEGIN gap="missing-generator:unit-test:a87c6c67" tracker="2370" reason="The GKE harness needs a static regression oracle for Lumen-only phase selection until shell-control-flow generation exists."
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ACCEPTANCE_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
RUN_SCRIPT="$ACCEPTANCE_ROOT/scripts/run.sh"
RENDER_SCRIPT="$ACCEPTANCE_ROOT/scripts/render-manifests.sh"
SCHEMA="$ACCEPTANCE_ROOT/evidence/schema.json"

bash -n "$RUN_SCRIPT"
bash -n "$RENDER_SCRIPT"
jq empty "$SCHEMA"

rg -F 'LUMEN_ONLY="${LUMEN_ONLY:-0}"' "$RUN_SCRIPT" >/dev/null
rg -F 'LUMEN_ONLY=1 requires LUMEN_IMAGE as an immutable @sha256 digest reference' "$RUN_SCRIPT" >/dev/null
rg -F 'LUMEN_PRIOR_ACCEPTANCE is not allowed' "$RUN_SCRIPT" >/dev/null
rg -F 'Lumen-only acceptance passed; mandatory cleanup runs on EXIT' "$RUN_SCRIPT" >/dev/null
rg -F 'if [[ "$LUMEN_ONLY" != "1" ]]; then' "$RUN_SCRIPT" >/dev/null
rg -F 'if [[ "$LUMEN_ONLY" != "1" ]]; then' "$RENDER_SCRIPT" >/dev/null
jq -e '.properties.mode.enum == ["full", "lumen-only"]' "$SCHEMA" >/dev/null
jq -e '.properties.acceptance.required == ["lumen"]' "$SCHEMA" >/dev/null
# HANDWRITE-END
