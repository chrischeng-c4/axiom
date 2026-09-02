#!/usr/bin/env bash
set -euo pipefail

# Park the shared acceptance cluster: resize acceptance-pool to 0 nodes so the
# idle cluster costs only the free-tier zonal management fee. The autoscaler
# never evicts the last node on its own (kube-dns and the other kube-system
# singletons pin it), so parking must be this explicit resize.
#
# CAUTION: the pool is shared with the acceptance/gcp lumen/sift/tape harness.
# Parking while one of its runs is in flight kills that run's nodes. v1 is
# manual-dispatch only, so the operator arbitrates; see ../README.md.

: "${PROJECT_ID:?PROJECT_ID is required}"
GKE_ZONE="${GKE_ZONE:-asia-east1-a}"
PERSISTENT_CLUSTER_NAME="${PERSISTENT_CLUSTER_NAME:-axiom-operator-acceptance}"
ACCEPTANCE_POOL_NAME="${ACCEPTANCE_POOL_NAME:-acceptance-pool}"

# Nothing to park is success, not failure: park.sh runs from `if: always()`
# cleanup paths that must stay green when the run died before cluster creation.
if ! gcloud container clusters describe "$PERSISTENT_CLUSTER_NAME" \
  --project="$PROJECT_ID" --zone="$GKE_ZONE" >/dev/null 2>&1; then
  echo "cluster $PERSISTENT_CLUSTER_NAME absent; nothing to park"
  exit 0
fi

gcloud container clusters resize "$PERSISTENT_CLUSTER_NAME" \
  --project="$PROJECT_ID" --zone="$GKE_ZONE" \
  --node-pool="$ACCEPTANCE_POOL_NAME" --num-nodes=0 --quiet
echo "parked: $ACCEPTANCE_POOL_NAME at 0 nodes"
