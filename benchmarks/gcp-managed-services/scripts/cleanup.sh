#!/usr/bin/env bash
set -euo pipefail

: "${PROJECT_ID:?PROJECT_ID is required}"
: "${REGION:?REGION is required}"
: "${RUN_ID:?RUN_ID is required}"
: "${STATE_DIR:?STATE_DIR is required}"
: "${BENCH_ROOT:?BENCH_ROOT is required}"
BOOTSTRAP_RUN_ID="${BOOTSTRAP_RUN_ID:-$RUN_ID}"
BOOTSTRAP_STATE_DIR="${BOOTSTRAP_STATE_DIR:-$STATE_DIR}"
GCS_SOURCE_PREFIX="${GCS_SOURCE_PREFIX:-gs://${PROJECT_ID}_cloudbuild/source/axiom-bench-${BOOTSTRAP_RUN_ID}}"

destroy_state() {
  local name="$1"
  local state_root="$2"
  local state_run_id="$3"
  local config="$BENCH_ROOT/$name"
  local state="$state_root/$name.tfstate"
  local data="$state_root/.terraform-$name"
  [[ -f "$state" ]] || return 0
  local args=(
    -state="$state"
    -auto-approve
    -var="project_id=$PROJECT_ID"
    -var="region=$REGION"
    -var="run_id=$state_run_id"
  )
  if [[ "$name" == "environment" ]]; then
    args+=(
      -var="registry=${REGISTRY:?REGISTRY is required for environment destroy}"
      -var="image_tag=${IMAGE_TAG:?IMAGE_TAG is required for environment destroy}"
    )
  fi
  local attempts=1
  [[ "$name" == "bootstrap" ]] && attempts=4
  local attempt
  for (( attempt=1; attempt<=attempts; attempt++ )); do
    if TF_DATA_DIR="$data" terraform -chdir="$config" destroy "${args[@]}"; then
      return 0
    fi
    if [[ "$attempt" == "$attempts" ]]; then
      return 1
    fi
    echo ">> ${name} destroy retry ${attempt}/${attempts}: waiting for GCP asset inventory convergence" >&2
    sleep 15
  done
}

destroy_state environment "$STATE_DIR" "$RUN_ID"
CHECK_APIS=0 PROJECT_ID="$PROJECT_ID" REGION="$REGION" RUN_ID="$RUN_ID" \
  "$BENCH_ROOT/scripts/verify-clean.sh"
bootstrap_complete=1
if ! destroy_state bootstrap "$BOOTSTRAP_STATE_DIR" "$BOOTSTRAP_RUN_ID"; then
  remaining="$(TF_DATA_DIR="$BOOTSTRAP_STATE_DIR/.terraform-bootstrap" terraform \
    -chdir="$BENCH_ROOT/bootstrap" state list \
    -state="$BOOTSTRAP_STATE_DIR/bootstrap.tfstate")"
  if [[ -n "$remaining" && "$remaining" != "google_project_service.container" ]]; then
    echo "bootstrap cleanup left unexpected Terraform resources:" >&2
    echo "$remaining" >&2
    exit 1
  fi
  bootstrap_complete=0
  echo ">> GKE API disable is delayed by stale GCP asset inventory; all billable resources are already absent" >&2
fi

# Cloud Build stages the source outside Terraform in an existing project
# bucket. Its run-scoped prefix is safe to remove independently.
gcloud storage rm --recursive "$GCS_SOURCE_PREFIX" >/dev/null 2>&1 || true

if [[ "$bootstrap_complete" == "1" ]]; then
  CHECK_APIS=1 PROJECT_ID="$PROJECT_ID" REGION="$REGION" RUN_ID="$BOOTSTRAP_RUN_ID" \
    "$BENCH_ROOT/scripts/verify-clean.sh"
else
  CHECK_APIS=1 ALLOW_CONTAINER_API=1 PROJECT_ID="$PROJECT_ID" \
    REGION="$REGION" RUN_ID="$BOOTSTRAP_RUN_ID" "$BENCH_ROOT/scripts/verify-clean.sh"
fi

# Terraform state contains the ephemeral receiver secret. Delete only this
# run's /tmp state after both destroy and independent cloud inventory pass.
find "$STATE_DIR" -type f -delete
find "$STATE_DIR" -depth -type d -empty -delete
if [[ "$BOOTSTRAP_STATE_DIR" != "$STATE_DIR" ]]; then
  find "$BOOTSTRAP_STATE_DIR" -type f -delete
  find "$BOOTSTRAP_STATE_DIR" -depth -type d -empty -delete
fi
