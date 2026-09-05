#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ACCEPTANCE_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
source "$ACCEPTANCE_ROOT/scripts/source-prefix.sh"

run_id="prefix-red"
valid="gs://candidate-source/source/axiom-gcp-operator-${run_id}"
[[ "$(validated_source_bucket "$valid" "$run_id")" == "candidate-source" ]]
valid_object="source/axiom-gcp-operator-${run_id}/source-deadbeef.tgz"
[[ "$(validated_source_object_uri "$valid" "$run_id" "candidate-source" "$valid_object")" \
  == "gs://candidate-source/${valid_object}" ]] || {
  echo "rejected a Cloud Build source object inside the exact run prefix" >&2
  exit 1
}

for object_case in \
  "other-source:${valid_object}" \
  "candidate-source:source/axiom-gcp-operator-another-run/source.tgz" \
  "candidate-source:source/axiom-gcp-operator-${run_id}-suffix/source.tgz" \
  "candidate-source:source/axiom-gcp-operator-${run_id}"; do
  object_bucket="${object_case%%:*}"
  object_name="${object_case#*:}"
  if validated_source_object_uri \
      "$valid" "$run_id" "$object_bucket" "$object_name" >/dev/null 2>&1; then
    echo "accepted Cloud Build source object outside the run prefix: gs://${object_bucket}/${object_name}" >&2
    exit 1
  fi
done

for unsafe in \
  "gs://candidate-source" \
  "gs://candidate-source/source" \
  "gs://candidate-source/source/axiom-gcp-operator" \
  "gs://candidate-source/source/axiom-gcp-operator-another-run" \
  "gs://candidate-source/source/axiom-gcp-operator-${run_id}/descendant"; do
  if validated_source_bucket "$unsafe" "$run_id" >/dev/null 2>&1; then
    echo "accepted unsafe Cloud Build source prefix: $unsafe" >&2
    exit 1
  fi
done

test_root="$(mktemp -d "${TMPDIR:-/tmp}/sift-source-prefix.XXXXXX")"
receipt="$test_root/source-prefix.json"
cleanup_test() {
  find "$test_root" -type f -delete
  find "$test_root" -depth -type d -empty -delete
}
trap cleanup_test EXIT INT TERM
write_source_prefix_receipt "$receipt" "project-1" "$run_id" "$valid"
verify_source_prefix_receipt "$receipt" "project-1" "$run_id" "$valid"

cat > "$test_root/cloud-build-submit.json" <<EOF
{"source":{"storageSource":{"bucket":"candidate-source","object":"${valid_object}"}}}
EOF
cat > "$test_root/cloud-build-source-binding.json" <<EOF
{"source_uri":"gs://candidate-source/${valid_object}"}
EOF
verify_cloud_build_source_evidence "$test_root" "$valid" "$run_id"

jq '.source.storageSource.object = "source/axiom-gcp-operator-another-run/source.tgz"' \
  "$test_root/cloud-build-submit.json" > "$test_root/cloud-build-submit.tmp"
mv "$test_root/cloud-build-submit.tmp" "$test_root/cloud-build-submit.json"
if verify_cloud_build_source_evidence "$test_root" "$valid" "$run_id"; then
  echo "accepted Cloud Build evidence for an object outside the run prefix" >&2
  exit 1
fi

jq '.run_id = "another-run"' "$receipt" > "${receipt}.tmp"
mv "${receipt}.tmp" "$receipt"
if verify_source_prefix_receipt "$receipt" "project-1" "$run_id" "$valid"; then
  echo "accepted a source-prefix receipt from another run" >&2
  exit 1
fi

echo "run-scoped Cloud Build source prefix E2E: ok"
