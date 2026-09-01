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
ACCEPTANCE_APPS="${ACCEPTANCE_APPS:-lumen sift}"
KUBECONFIG="${KUBECONFIG:-$STATE_DIR/kubeconfig}"
TERRAFORM_ENVIRONMENT_DIR="${TERRAFORM_ENVIRONMENT_DIR:-$STATE_DIR/environment}"
export KUBECONFIG
case "$ACCEPTANCE_APPS" in
  "lumen sift") acceptance_mode="lumen-sift" ;;
  "lumen auth") acceptance_mode="lumen-auth" ;;
  "sift") acceptance_mode="sift" ;;
  "tape") acceptance_mode="tape" ;;
  *)
    echo "ACCEPTANCE_APPS must be exactly 'lumen sift' (default), 'lumen auth', 'sift', or 'tape'" >&2
    exit 1
    ;;
esac

state="$STATE_DIR/environment.tfstate"
tf_data="$STATE_DIR/.terraform-environment"
mkdir -p "$EVIDENCE_DIR/kubernetes"

capture_failure_evidence() {
  kubectl get deployment,statefulset,cronjob,job,pod,pvc -A -o json \
    > "$EVIDENCE_DIR/kubernetes/workloads-before-cleanup.json" 2>/dev/null || true
  if [[ "$acceptance_mode" == "tape" ]]; then
    kubectl logs -n tape-system deployment/tape-operator --tail=500 --request-timeout=15s \
      > "$EVIDENCE_DIR/kubernetes/tape-operator.log" 2>&1 || true
  elif [[ "$acceptance_mode" == "lumen-auth" ]]; then
    kubectl logs -n lumen-system deployment/lumen-operator --tail=500 --request-timeout=15s \
      > "$EVIDENCE_DIR/kubernetes/lumen-operator.log" 2>&1 || true
  elif [[ "$acceptance_mode" == "sift" ]]; then
    kubectl logs -n sift-system deployment/sift-operator --tail=500 --request-timeout=15s \
      > "$EVIDENCE_DIR/kubernetes/sift-operator.log" 2>&1 || true
  else
    kubectl logs -n lumen-system deployment/lumen-operator --tail=500 --request-timeout=15s \
      > "$EVIDENCE_DIR/kubernetes/lumen-operator.log" 2>&1 || true
    kubectl logs -n sift-system deployment/sift-operator --tail=500 --request-timeout=15s \
      > "$EVIDENCE_DIR/kubernetes/sift-operator.log" 2>&1 || true
  fi
}

delete_sift_instance() {
  local namespace="$1"
  local name="$2"
  if ! kubectl get sift.sift.axiom.dev "$name" --namespace "$namespace" \
    >/dev/null 2>&1; then
    return 0
  fi
  if kubectl delete sift.sift.axiom.dev "$name" --namespace "$namespace" \
    --wait=true --timeout=180s >/dev/null 2>&1; then
    return 0
  fi

  # The instance and namespace are owned by this run. If the operator cannot
  # finish its cluster-child finalizer, remove only this known acceptance CR's
  # finalizers so cleanup cannot leave a billing resource behind forever.
  echo "Sift instance $namespace/$name did not finalize; removing its orphaned finalizers" >&2
  kubectl patch sift.sift.axiom.dev "$name" --namespace "$namespace" \
    --type=merge -p '{"metadata":{"finalizers":[]}}' >/dev/null 2>&1 || true
  kubectl wait --for=delete "sift.sift.axiom.dev/$name" --namespace "$namespace" \
    --timeout=60s >/dev/null 2>&1 || true
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
  if [[ "$acceptance_mode" == "tape" ]]; then
    namespaces=(tape tape-system)
  elif [[ "$acceptance_mode" == "lumen-auth" ]]; then
    namespaces=(lumen lumen-system lumen-auth-client)
  elif [[ "$acceptance_mode" == "sift" ]]; then
    namespaces=(sift sift-system sift-restore)
  else
    # lumen-fleet-a/-b are the data-plane namespaces the LumenFleet leg
    # materializes into. They are swept HERE, not only at the end of that leg,
    # because a leg that fails midway leaves StatefulSets and their PVCs behind
    # -- and this cluster is persistent, so a leaked PVC is a Persistent Disk
    # that bills forever with nothing left to point at it. Every namespace the
    # run can create belongs in this list, including the ones a passing run
    # tears down itself.
    # lumen-auth-client holds the client ServiceAccount the auth leg (#2879)
    # puts in a *second* namespace to prove a SubjectAccessReview scoped to the
    # serving namespace does not honour a grant written elsewhere.
    namespaces=(lumen lumen-system sift sift-system lumen-fleet-a lumen-fleet-b lumen-auth-client)
  fi
  # Sift owns cluster-scoped RBAC through a CR finalizer. Delete each known
  # acceptance instance while sift-system still hosts the operator. Deleting
  # the operator namespace first strands the finalizer and blocks both the
  # data namespace and CRD in Terminating.
  if [[ "$acceptance_mode" == "sift" || "$acceptance_mode" == "lumen-sift" ]]; then
    delete_sift_instance sift sift
    delete_sift_instance sift-restore sift-restore
  fi
  # The fleet controller reconciles cluster-wide, so it must lose its API
  # before its target namespaces start terminating; otherwise a pass that
  # lands between two deletes re-materializes a Lumen into a namespace on its
  # way out and the no-leftovers gate trips on a resource cleanup just removed.
  if [[ "$acceptance_mode" != "tape" && "$acceptance_mode" != "sift" ]]; then
    kubectl delete customresourcedefinition lumenfleets.lumen.dev \
      --ignore-not-found --wait=true --timeout=180s >/dev/null 2>&1 || true
  fi
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
  if [[ "$acceptance_mode" == "tape" ]]; then
    kubectl delete customresourcedefinition tapes.tape.dev \
      --ignore-not-found --wait=true --timeout=180s >/dev/null 2>&1 || true
  else
    if [[ "$acceptance_mode" == "lumen-auth" ]]; then
      kubectl delete customresourcedefinition lumens.lumen.dev \
        --ignore-not-found --wait=true --timeout=180s >/dev/null 2>&1 || true
    elif [[ "$acceptance_mode" == "sift" ]]; then
      kubectl delete customresourcedefinition sifts.sift.axiom.dev \
        --ignore-not-found --wait=true --timeout=180s >/dev/null 2>&1 || true
      kubectl delete clusterrolebinding \
        -l axiom-owner=gcp-operator-acceptance,axiom-run-id="$RUN_ID" \
        --ignore-not-found --wait=true --timeout=180s >/dev/null 2>&1 || true
      kubectl delete clusterrolebinding \
        -l app.kubernetes.io/component=auth-delegation,app.kubernetes.io/name=sift \
        --ignore-not-found --wait=true --timeout=180s >/dev/null 2>&1 || true
    else
      kubectl delete customresourcedefinition lumens.lumen.dev sifts.sift.axiom.dev \
      --ignore-not-found --wait=true --timeout=180s >/dev/null 2>&1 || true
    fi
    # The per-instance `system:auth-delegator` binding (#2876) is cluster-scoped,
    # so nothing above reaches it: a cluster-scoped object cannot carry an owner
    # reference to a namespaced CR, and the operator's own sweep dies with the
    # namespace that hosts it. Deleting the namespace therefore leaves a live
    # delegated-authentication grant naming a ServiceAccount that no longer
    # exists. Labels are the only link back to the instance, which is exactly
    # what they were rendered for.
    kubectl delete clusterrolebinding \
      -l app.kubernetes.io/component=auth-delegation,app.kubernetes.io/name=lumen \
      --ignore-not-found --wait=true --timeout=180s >/dev/null 2>&1 || true
  fi
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
  if [[ "$acceptance_mode" == "tape" ]]; then
    destroy_args+=(-var="acceptance_apps=tape")
  elif [[ "$acceptance_mode" == "lumen-auth" ]]; then
    destroy_args+=(-var="acceptance_apps=lumen-auth")
  elif [[ "$acceptance_mode" == "sift" ]]; then
    destroy_args+=(-var="acceptance_apps=sift")
  fi
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

if [[ "$acceptance_mode" == "sift" ]]; then
  delete_run_image sift
  delete_run_image rig
elif [[ "$acceptance_mode" == "tape" ]]; then
  delete_run_image tape
elif [[ "$acceptance_mode" == "lumen-auth" ]]; then
  delete_run_image lumen
else
  delete_run_image lumen
  delete_run_image sift
fi
gcloud storage rm --recursive "${GCS_SOURCE_PREFIX}/**" >/dev/null 2>&1 || true

PROJECT_ID="$PROJECT_ID" REGION="$REGION" GKE_ZONE="$GKE_ZONE" RUN_ID="$RUN_ID" \
  REGISTRY="$REGISTRY" IMAGE_TAG="$IMAGE_TAG" \
  GCS_SOURCE_PREFIX="$GCS_SOURCE_PREFIX" EVIDENCE_DIR="$EVIDENCE_DIR" \
  PERSISTENT_CLUSTER_NAME="$PERSISTENT_CLUSTER_NAME" \
  ACCEPTANCE_APPS="$ACCEPTANCE_APPS" \
  "$ACCEPTANCE_ROOT/scripts/verify-clean.sh"

if [[ "$acceptance_mode" == "sift" \
  && -f "$EVIDENCE_DIR/acceptance.json" \
  && -f "$EVIDENCE_DIR/cleanup.json" ]]; then
  acceptance_tmp="$(mktemp "$EVIDENCE_DIR/.acceptance.json.XXXXXX")"
  jq '.acceptance.sift.cleanup_evidence = "cleanup.json"' \
    "$EVIDENCE_DIR/acceptance.json" > "$acceptance_tmp"
  mv "$acceptance_tmp" "$EVIDENCE_DIR/acceptance.json"
  cp "$EVIDENCE_DIR/acceptance.json" "$EVIDENCE_DIR/sift-mvp-acceptance.json"
fi
