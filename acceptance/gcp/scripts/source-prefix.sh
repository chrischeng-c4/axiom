#!/usr/bin/env bash

# Validate the only Cloud Build staging prefix that this acceptance run owns.
# The exact shape keeps cleanup away from the bucket root and from other runs.
validated_source_bucket() {
  local prefix="$1"
  local run_id="$2"

  [[ "$run_id" =~ ^[a-z0-9][a-z0-9-]{0,17}$ ]] || return 1
  [[ "$prefix" =~ ^gs://([A-Za-z0-9._-]+)/source/axiom-gcp-operator-([a-z0-9][a-z0-9-]{0,17})$ ]] \
    || return 1
  [[ "${BASH_REMATCH[2]}" == "$run_id" ]] || return 1
  printf '%s\n' "${BASH_REMATCH[1]}"
}

validated_source_object_uri() {
  local prefix="$1"
  local run_id="$2"
  local object_bucket="$3"
  local object_name="$4"
  local expected_bucket object_prefix

  expected_bucket="$(validated_source_bucket "$prefix" "$run_id")" || return 1
  [[ "$object_bucket" == "$expected_bucket" ]] || return 1
  object_prefix="${prefix#gs://${expected_bucket}/}/"
  [[ -n "$object_name" && "$object_name" == "$object_prefix"* ]] || return 1
  printf 'gs://%s/%s\n' "$object_bucket" "$object_name"
}

verify_cloud_build_source_evidence() {
  local evidence_dir="$1"
  local prefix="$2"
  local run_id="$3"
  local submit="$evidence_dir/cloud-build-submit.json"
  local binding="$evidence_dir/cloud-build-source-binding.json"
  local object_bucket object_name source_uri

  if [[ ! -e "$submit" ]]; then
    [[ ! -e "$binding" ]]
    return
  fi
  [[ -f "$submit" && ! -L "$submit" ]] || return 1
  object_bucket="$(jq -er \
    '.source.storageSource.bucket | strings | select(length > 0)' "$submit")" \
    || return 1
  object_name="$(jq -er \
    '.source.storageSource.object | strings | select(length > 0)' "$submit")" \
    || return 1
  source_uri="$(validated_source_object_uri \
    "$prefix" "$run_id" "$object_bucket" "$object_name")" || return 1

  if [[ -e "$binding" ]]; then
    [[ -f "$binding" && ! -L "$binding" ]] || return 1
    jq -e --arg source_uri "$source_uri" '
      .source_uri == $source_uri
    ' "$binding" >/dev/null || return 1
  fi
}

write_source_prefix_receipt() {
  local receipt="$1"
  local project_id="$2"
  local run_id="$3"
  local prefix="$4"
  local bucket receipt_dir temporary

  bucket="$(validated_source_bucket "$prefix" "$run_id")" || return 1
  receipt_dir="$(dirname "$receipt")"
  mkdir -p "$receipt_dir"
  temporary="$(mktemp "$receipt_dir/.source-prefix.XXXXXX")"
  jq -n \
    --arg project_id "$project_id" \
    --arg run_id "$run_id" \
    --arg bucket "$bucket" \
    --arg prefix "$prefix" '
      {
        schema: "axiom.gcp.operator.source-prefix.v1",
        project_id: $project_id,
        run_id: $run_id,
        bucket: $bucket,
        prefix: $prefix
      }
    ' > "$temporary"
  chmod 0600 "$temporary"
  mv "$temporary" "$receipt"
}

verify_source_prefix_receipt() {
  local receipt="$1"
  local project_id="$2"
  local run_id="$3"
  local prefix="$4"
  local bucket

  bucket="$(validated_source_bucket "$prefix" "$run_id")" || return 1
  [[ -f "$receipt" && ! -L "$receipt" ]] || return 1
  jq -e \
    --arg project_id "$project_id" \
    --arg run_id "$run_id" \
    --arg bucket "$bucket" \
    --arg prefix "$prefix" '
      type == "object"
      and keys == ["bucket", "prefix", "project_id", "run_id", "schema"]
      and .schema == "axiom.gcp.operator.source-prefix.v1"
      and .project_id == $project_id
      and .run_id == $run_id
      and .bucket == $bucket
      and .prefix == $prefix
    ' "$receipt" >/dev/null
}
