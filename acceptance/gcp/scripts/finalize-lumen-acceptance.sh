#!/usr/bin/env bash
set -euo pipefail

: "${PROJECT_ID:?PROJECT_ID is required}"
: "${REGION:?REGION is required}"
: "${GKE_ZONE:?GKE_ZONE is required}"
: "${RUN_ID:?RUN_ID is required}"
: "${EVIDENCE_DIR:?EVIDENCE_DIR is required}"
auth_file="${LUMEN_AUTH_ACCEPTANCE_EVIDENCE:-$EVIDENCE_DIR/lumen-auth-acceptance.json}"
out="$EVIDENCE_DIR/acceptance.json"
tmp="$EVIDENCE_DIR/.acceptance.json.$$"
rm -f "$out" "$tmp"
fail() { rm -f "$out" "$tmp"; echo "lumen acceptance finalizer: $1" >&2; exit 1; }
mode="${1:-}"
case "$mode" in lumen-auth|lumen-sift) ;; *) fail "unknown mode" ;; esac
[[ -s "$auth_file" ]] || fail "auth evidence is missing"
jq -e --arg run "$RUN_ID" --arg project "$PROJECT_ID" '
  . as $root |
  .schema == "axiom.gcp.lumen.auth.acceptance.v1" and .status == "passed"
  and .run_id == $run and .project_id == $project
  and ((.issuers // []) | length >= 2)
  and all(.issuers[]?; (.kubernetes_username | type == "string" and length > 0))
  and any(.issuers[]?; .kind == "google-service-account" and .cluster_admin == false)
  and (.sibling_mint_refusals | type == "number" and floor == . and . >= 1)
  and (.rows | type == "object")
  and (($root.rows | keys | sort) == ["authorization","identity_observations","non_ksa_rejections","redaction","revocations","sibling_refusals","teardown"])
  and (["identity_observations","non_ksa_rejections","authorization","sibling_refusals","revocations","redaction","teardown"] | all(.[]; (. as $k | ($root.rows[$k] | type == "number" and floor == . and . > 0))))
  and (.revocation.issuer_token_request_seconds | type == "number" and floor == . and . >= 0)
  and (.revocation.lumen_authorization_seconds | type == "number" and floor == . and . >= 0)
  and (.revocation.documented_bound_seconds | type == "number" and floor == . and . >= 0)
  and (.revocation.lumen_authorization_seconds <= .revocation.documented_bound_seconds)
  and (.revocation.issuer_token_request_seconds <= .revocation.documented_bound_seconds)
' "$auth_file" >/dev/null || fail "auth evidence is malformed or incomplete"

if [[ "$mode" == "lumen-auth" ]]; then
  jq -n --arg schema "axiom.gcp.operator.acceptance.v1" --arg project "$PROJECT_ID" \
    --arg region "$REGION" --arg zone "$GKE_ZONE" --arg run "$RUN_ID" \
    --slurpfile auth "$auth_file" \
    '{schema:$schema, project_id:$project, region:$region, gke_zone:$zone, run_id:$run, acceptance:{auth:$auth[0]}}' > "$tmp" || fail "could not build auth-only evidence"
else
  : "${LUMEN_ACCEPTANCE_EVIDENCE:?LUMEN_ACCEPTANCE_EVIDENCE is required for lumen-sift mode}"
  : "${LUMEN_ACCEPTANCE_PROVENANCE:?LUMEN_ACCEPTANCE_PROVENANCE is required for lumen-sift mode}"
  : "${BACKUP_BUCKET:?BACKUP_BUCKET is required for lumen-sift mode}"
  sift="$EVIDENCE_DIR/sift-acceptance.json"
  [[ -s "$LUMEN_ACCEPTANCE_EVIDENCE" && -s "$sift" ]] || fail "Lumen or Sift evidence is missing"
  jq -n --arg schema "axiom.gcp.operator.acceptance.v1" --arg project "$PROJECT_ID" \
    --arg region "$REGION" --arg zone "$GKE_ZONE" --arg run "$RUN_ID" \
    --arg bucket "$BACKUP_BUCKET" --arg provenance "$LUMEN_ACCEPTANCE_PROVENANCE" \
    --arg le "$LUMEN_ACCEPTANCE_EVIDENCE" --slurpfile lumen "$LUMEN_ACCEPTANCE_EVIDENCE" \
    --slurpfile auth "$auth_file" --slurpfile sift "$sift" \
    '{schema:$schema,project_id:$project,region:$region,gke_zone:$zone,run_id:$run,backup_bucket:$bucket,lumen_evidence:$le,lumen_provenance:$provenance,acceptance:{lumen:$lumen[0],auth:$auth[0],sift:$sift[0]}}' > "$tmp" || fail "could not build full evidence"
fi
mv "$tmp" "$out"
echo "terminal acceptance evidence: $out"
