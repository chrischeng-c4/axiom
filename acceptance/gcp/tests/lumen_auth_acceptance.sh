#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FINALIZER="$ROOT/scripts/finalize-lumen-acceptance.sh"
tmp="$(mktemp -d "${TMPDIR:-/tmp}/lumen-auth-finalizer.XXXXXX")"
trap 'rm -rf "$tmp"' EXIT
export PROJECT_ID=axiom-test-project REGION=asia-east1 GKE_ZONE=asia-east1-a RUN_ID=auth-proof EVIDENCE_DIR="$tmp"
AUTH="$tmp/lumen-auth-acceptance.json"
valid='{"schema":"axiom.gcp.lumen.auth.acceptance.v1","run_id":"auth-proof","project_id":"axiom-test-project","issuers":[{"kind":"google-user","kubernetes_username":"user:test","cluster_admin":true},{"kind":"google-service-account","kubernetes_username":"gsa:test","cluster_admin":false}],"rows":{"identity_observations":1,"non_ksa_rejections":1,"authorization":1,"sibling_refusals":1,"revocations":1,"redaction":1,"teardown":1},"sibling_mint_refusals":1,"revocation":{"issuer_token_request_seconds":1,"lumen_authorization_seconds":2,"documented_bound_seconds":360},"status":"passed"}'
printf '%s\n' "$valid" > "$AUTH"
assertions=0
ok() { assertions=$((assertions + 1)); }
expect_fail() {
  rm -f "$EVIDENCE_DIR/acceptance.json"
  if bash "$FINALIZER" lumen-auth >/dev/null 2>&1; then echo "expected rejection" >&2; exit 1; fi
  [[ ! -e "$EVIDENCE_DIR/acceptance.json" ]] || { echo "stale terminal output survived" >&2; exit 1; }
  ok
}
bash "$FINALIZER" lumen-auth >/dev/null
jq -e '.acceptance.auth.status == "passed"' "$EVIDENCE_DIR/acceptance.json" >/dev/null; ok
printf '%s\n' '{"schema":"axiom.gcp.lumen.acceptance.v1","operator_reconcile_1x1":"passed"}' > "$tmp/lumen.json"
printf '%s\n' '{"schema":"axiom.gcp.sift.acceptance.v1","operator_reconcile_1x1":"passed"}' > "$tmp/sift-acceptance.json"
export LUMEN_ACCEPTANCE_EVIDENCE="$tmp/lumen.json" LUMEN_ACCEPTANCE_PROVENANCE=current-run BACKUP_BUCKET=axiom-test-backup
bash "$FINALIZER" lumen-sift >/dev/null
jq -e '(.acceptance | keys) == ["auth","lumen","sift"]' "$EVIDENCE_DIR/acceptance.json" >/dev/null; ok
for row in identity_observations non_ksa_rejections authorization sibling_refusals revocations redaction teardown; do
  jq --arg row "$row" '.rows[$row]=0' "$AUTH" > "$tmp/bad.json"; mv "$tmp/bad.json" "$AUTH"; expect_fail
  printf '%s\n' "$valid" > "$AUTH"
done
jq '.issuers[1].cluster_admin=true' "$AUTH" > "$tmp/bad.json"; mv "$tmp/bad.json" "$AUTH"; expect_fail
printf '%s\n' "$valid" > "$AUTH"
jq '.revocation.lumen_authorization_seconds=361' "$AUTH" > "$tmp/bad.json"; mv "$tmp/bad.json" "$AUTH"; expect_fail
printf '%s\n' "$valid" > "$AUTH"
jq '.revocation.issuer_token_request_seconds=361' "$AUTH" > "$tmp/bad.json"; mv "$tmp/bad.json" "$AUTH"; expect_fail
printf '%s\n' "$valid" > "$AUTH"
jq '.rows.unexpected=1' "$AUTH" > "$tmp/bad.json"; mv "$tmp/bad.json" "$AUTH"; expect_fail
printf '%s\n' "$valid" > "$AUTH"
jq '.rows.authorization=1.5' "$AUTH" > "$tmp/bad.json"; mv "$tmp/bad.json" "$AUTH"; expect_fail
printf '%s\n' "$valid" > "$AUTH"
jq '.sibling_mint_refusals=1.5' "$AUTH" > "$tmp/bad.json"; mv "$tmp/bad.json" "$AUTH"; expect_fail
printf '%s\n' "$valid" > "$AUTH"
jq '.revocation.documented_bound_seconds=1.5' "$AUTH" > "$tmp/bad.json"; mv "$tmp/bad.json" "$AUTH"; expect_fail
printf '%s\n' "$valid" > "$AUTH"
jq '.issuers[0].kubernetes_username=""' "$AUTH" > "$tmp/bad.json"; mv "$tmp/bad.json" "$AUTH"; expect_fail
printf '%s\n' "$valid" > "$AUTH"
if bash "$FINALIZER" unknown >/dev/null 2>&1; then echo "expected unknown mode rejection" >&2; exit 1; fi
[[ ! -e "$EVIDENCE_DIR/acceptance.json" ]] || { echo "unknown mode preserved stale output" >&2; exit 1; }
ok
printf '%s\n' "$valid" > "$AUTH"
printf '{' > "$AUTH"; expect_fail
printf '%s\n' "$valid" > "$AUTH"
rm "$AUTH"; expect_fail
echo "lumen auth finalizer checks: $assertions"
(( assertions >= 12 ))
