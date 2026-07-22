#!/usr/bin/env bash
set -euo pipefail

: "${PROJECT_ID:?PROJECT_ID is required}"
: "${REGION:?REGION is required}"
: "${GKE_ZONE:?GKE_ZONE is required}"
: "${RUN_ID:?RUN_ID is required}"
: "${STATE_DIR:?STATE_DIR is required}"
: "${ACCEPTANCE_ROOT:?ACCEPTANCE_ROOT is required}"
: "${REGISTRY:?REGISTRY is required}"
: "${IMAGE_TAG:?IMAGE_TAG is required}"
: "${GCS_SOURCE_PREFIX:?GCS_SOURCE_PREFIX is required}"
: "${EVIDENCE_DIR:?EVIDENCE_DIR is required}"
ARTIFACT_REGISTRY_REPOSITORY="${ARTIFACT_REGISTRY_REPOSITORY:-courier}"
PERSISTENT_CLUSTER_NAME="${PERSISTENT_CLUSTER_NAME:-axiom-operator-acceptance}"
KUBECONFIG="${KUBECONFIG:-$STATE_DIR/kubeconfig}"
TERRAFORM_ENVIRONMENT_DIR="${TERRAFORM_ENVIRONMENT_DIR:-$STATE_DIR/environment}"
export KUBECONFIG

state="$STATE_DIR/environment.tfstate"
tf_data="$STATE_DIR/.terraform-environment"
mkdir -p "$EVIDENCE_DIR/kubernetes"

capture_failure_evidence() {
  kubectl get deployment,statefulset,cronjob,job,pod,pvc -A -o json \
    > "$EVIDENCE_DIR/kubernetes/workloads-before-cleanup.json" 2>/dev/null || true
  kubectl logs -n lumen-system deployment/lumen-operator --tail=500 \
    > "$EVIDENCE_DIR/kubernetes/lumen-operator.log" 2>&1 || true
  kubectl logs -n sift-system deployment/sift-operator --tail=500 \
    > "$EVIDENCE_DIR/kubernetes/sift-operator.log" 2>&1 || true
}

delete_run_image() {
  local image="$1"
  local tagged="$REGISTRY/$image:$IMAGE_TAG"
  local digest_ref digest preexisting inventory tags
  digest_ref="$(jq -r --arg image "$image" '.[$image] // empty' "$EVIDENCE_DIR/images.json" 2>/dev/null || true)"
  digest="${digest_ref##*@}"

  # A tag is the only artifact this run can conclusively own. Never delete a
  # version through its tag: Artifact Registry may attach a pre-existing tag to
  # the same digest. Remove the tag first, then delete the digest only when the
  # before-run inventory proves it was new and the post-removal version has no
  # remaining tags.
  gcloud artifacts docker tags delete "$tagged" --project="$PROJECT_ID" --quiet \
    >/dev/null 2>&1 || true
  [[ "$digest" == sha256:* ]] || return 0

  inventory="$EVIDENCE_DIR/preexisting-${image}-images.json"
  if [[ -f "$inventory" ]] && jq -e --arg digest "$digest" \
    'any(.. | strings; contains($digest))' "$inventory" >/dev/null; then
    return 0
  fi

  local current
  current="$(gcloud artifacts docker images list "$REGISTRY/$image" \
    --project="$PROJECT_ID" --include-tags --format=json 2>/dev/null || true)"
  tags="$(jq -r --arg digest "$digest" '
    [.[] | select((tojson | contains($digest))) | (.tags // [])[]] | length
  ' <<<"$current" 2>/dev/null || printf '1')"
  if [[ "$tags" == "0" ]]; then
    gcloud artifacts docker images delete "$REGISTRY/$image@$digest" \
      --project="$PROJECT_ID" --quiet >/dev/null 2>&1 || true
  fi
}

if [[ -f "$STATE_DIR/kube-context-ready.txt" ]]; then
  capture_failure_evidence
  namespaces=(lumen lumen-system sift sift-system)
  for namespace in "${namespaces[@]}"; do
    kubectl delete namespace "$namespace" --ignore-not-found --wait=false \
      >/dev/null 2>&1 || true
  done
  # Namespace deletion can outlive kubectl's delete response while the
  # apiserver clears finalizers.  Do not run the no-leftovers gate against that
  # transient state: wait on the actual namespace objects instead.
  namespace_deadline=$((SECONDS + 300))
  while true; do
    remaining_namespaces=()
    for namespace in "${namespaces[@]}"; do
      if kubectl get namespace "$namespace" --no-headers >/dev/null 2>&1; then
        remaining_namespaces+=("$namespace")
      fi
    done
    [[ "${#remaining_namespaces[@]}" == "0" ]] && break
    if (( SECONDS >= namespace_deadline )); then
      echo "namespaces still terminating after cleanup wait: ${remaining_namespaces[*]}" >&2
      break
    fi
    sleep 5
  done
  kubectl delete customresourcedefinition lumens.lumen.dev sifts.sift.axiom.dev \
    --ignore-not-found --wait=true --timeout=180s >/dev/null 2>&1 || true
fi

if [[ -f "$STATE_DIR/cloud-build-id.txt" ]]; then
  build_id="$(sed -n '1p' "$STATE_DIR/cloud-build-id.txt")"
  build_status="$(gcloud builds describe "$build_id" --project="$PROJECT_ID" \
    --region="$REGION" --format='value(status)' 2>/dev/null || true)"
  case "$build_status" in
    QUEUED|PENDING|WORKING)
      gcloud builds cancel "$build_id" --project="$PROJECT_ID" \
        --region="$REGION" --quiet >/dev/null 2>&1 || true
      ;;
  esac
fi

if [[ -f "$state" ]]; then
  destroy_args=(
    -state="$state"
    -auto-approve
    -var="project_id=$PROJECT_ID"
    -var="region=$REGION"
    -var="gke_zone=$GKE_ZONE"
    -var="run_id=$RUN_ID"
    -var="artifact_registry_repository=$ARTIFACT_REGISTRY_REPOSITORY"
    -var="image_tag=$IMAGE_TAG"
  )
  for attempt in 1 2 3; do
    if TF_DATA_DIR="$tf_data" terraform -chdir="$TERRAFORM_ENVIRONMENT_DIR" \
      destroy "${destroy_args[@]}"; then
      break
    fi
    if [[ "$attempt" == "3" ]]; then
      echo "Terraform destroy failed after three attempts; state retained at $state" >&2
      exit 1
    fi
    sleep 15
  done
fi

delete_run_image lumen
delete_run_image sift
gcloud storage rm --recursive "${GCS_SOURCE_PREFIX}/**" >/dev/null 2>&1 || true

PROJECT_ID="$PROJECT_ID" REGION="$REGION" GKE_ZONE="$GKE_ZONE" RUN_ID="$RUN_ID" \
  REGISTRY="$REGISTRY" IMAGE_TAG="$IMAGE_TAG" \
  GCS_SOURCE_PREFIX="$GCS_SOURCE_PREFIX" EVIDENCE_DIR="$EVIDENCE_DIR" \
  PERSISTENT_CLUSTER_NAME="$PERSISTENT_CLUSTER_NAME" \
  "$ACCEPTANCE_ROOT/scripts/verify-clean.sh"
