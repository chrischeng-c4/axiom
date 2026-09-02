#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
: "${EVIDENCE_DIR:?EVIDENCE_DIR is required}"

verification="$EVIDENCE_DIR/sift-mvp-verification.json"
cleanup="$EVIDENCE_DIR/cleanup.json"
output="$EVIDENCE_DIR/acceptance.json"
named_output="$EVIDENCE_DIR/sift-mvp-acceptance.json"
schema="$SCRIPT_DIR/../evidence/schema.json"
validator="$SCRIPT_DIR/validate-sift-mvp-evidence.py"

[[ -f "$verification" ]] || {
  echo "Sift MVP verification evidence is missing: $verification" >&2
  exit 1
}
[[ -f "$cleanup" ]] || {
  echo "Sift MVP cleanup evidence is missing: $cleanup" >&2
  exit 1
}

python3 "$validator" --schema "$schema" --document "$verification" \
  --mode verification

output_tmp="$(mktemp "$EVIDENCE_DIR/.acceptance.json.XXXXXX")"
cleanup_tmp() {
  rm -f "$output_tmp"
}
trap cleanup_tmp EXIT INT TERM

jq --slurpfile cleanup "$cleanup" '
  if .schema != "axiom.gcp.operator.verification.v1"
    or .acceptance.sift.schema != "axiom.gcp.sift.mvp.verification.v1"
    or .acceptance.sift.status != "verification-passed"
    or .acceptance.sift.cleanup_evidence != null
    or $cleanup[0].schema != "axiom.gcp.operator.cleanup.v1"
    or $cleanup[0].status != "clean"
    or $cleanup[0].project_id != .project_id
    or $cleanup[0].region != .region
    or $cleanup[0].gke_zone != .gke_zone
    or $cleanup[0].run_id != .run_id
    or $cleanup[0].verified_at == null
    or $cleanup[0].preserved.artifact_registry != true
    or $cleanup[0].preserved.preexisting_apis != true
  then error("Sift verification or cleanup evidence is not terminal")
  else
    .schema = "axiom.gcp.operator.acceptance.v1"
    | .acceptance.sift.schema = "axiom.gcp.sift.mvp.acceptance.v1"
    | .acceptance.sift.status = "passed"
    | .acceptance.sift.cleanup_evidence = $cleanup[0]
  end
' "$verification" > "$output_tmp"

jq -e '
  .schema == "axiom.gcp.operator.acceptance.v1"
  and .acceptance.sift.schema == "axiom.gcp.sift.mvp.acceptance.v1"
  and .acceptance.sift.status == "passed"
  and .acceptance.sift.cleanup_evidence.schema == "axiom.gcp.operator.cleanup.v1"
  and .acceptance.sift.cleanup_evidence.status == "clean"
' "$output_tmp" >/dev/null
python3 "$validator" --schema "$schema" --document "$output_tmp" \
  --mode acceptance
mv "$output_tmp" "$output"
cp "$output" "$named_output"
python3 "$validator" --schema "$schema" --document "$output" \
  --mode acceptance
python3 "$validator" --schema "$schema" --document "$named_output" \
  --mode acceptance
trap - EXIT INT TERM
