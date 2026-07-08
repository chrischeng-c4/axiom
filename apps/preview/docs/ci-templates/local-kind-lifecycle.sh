#!/usr/bin/env bash
set -euo pipefail

: "${PREVIEW_MR:=123}"
: "${PREVIEW_SHA:=local}"
: "${PREVIEW_APP:=checkout}"
: "${PREVIEW_HOST:=uat.example.com}"
: "${PREVIEW_BASE_NAMESPACE:=uat-base}"
: "${PREVIEW_CONTEXT:=kind-preview-ec}"
: "${PREVIEW_TTL_HOURS:=48}"
: "${PREVIEW_IMAGE:=preview-kind-smoke:local}"

mkdir -p dist

preview discover-base \
  --context "$PREVIEW_CONTEXT" \
  --namespace "$PREVIEW_BASE_NAMESPACE" \
  --app "$PREVIEW_APP" \
  --out dist/base-contract.json

preview render \
  --mr "$PREVIEW_MR" \
  --sha "$PREVIEW_SHA" \
  --image "$PREVIEW_IMAGE" \
  --app "$PREVIEW_APP" \
  --host "$PREVIEW_HOST" \
  --base-contract dist/base-contract.json \
  --ttl-hours "$PREVIEW_TTL_HOURS" \
  --out dist/preview

preview apply --dir dist/preview --context "$PREVIEW_CONTEXT" --plan-only
preview apply --dir dist/preview --context "$PREVIEW_CONTEXT" --dry-run
preview apply --dir dist/preview --context "$PREVIEW_CONTEXT"
kubectl --context "$PREVIEW_CONTEXT" rollout status "deployment/${PREVIEW_APP}" -n "uat-mr-${PREVIEW_MR}" --timeout=120s
preview router resolve --context "$PREVIEW_CONTEXT" --host "$PREVIEW_HOST" --header-target "mr-${PREVIEW_MR}"
preview comment --mr "$PREVIEW_MR" --sha "$PREVIEW_SHA" --image "$PREVIEW_IMAGE" --app "$PREVIEW_APP" --host "$PREVIEW_HOST"

preview cleanup plan \
  --mr "$PREVIEW_MR" \
  --closed \
  --namespace-exists \
  --route-binding-exists \
  --base-namespace "$PREVIEW_BASE_NAMESPACE" \
  --control-namespace preview-system > dist/cleanup-plan.json
preview cleanup apply --plan dist/cleanup-plan.json --context "$PREVIEW_CONTEXT"
