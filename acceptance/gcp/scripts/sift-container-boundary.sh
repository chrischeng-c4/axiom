#!/usr/bin/env bash

sift_container_nonce_digest() {
  local nonce="$1"
  [[ "$nonce" =~ ^[0-9a-f]{64}$ ]] || return 1
  printf '%s' "$nonce" | openssl dgst -sha256 | awk '{print $NF}'
}

write_sift_container_owner() {
  local output="$1"
  local container_id="$2"
  local controller_image="$3"
  local state_dir="$4"
  local evidence_dir="$5"
  local handoff_digest="$6"
  local output_dir temporary
  [[ "$container_id" =~ ^[0-9a-f]{64}$ \
    && "$controller_image" == *@sha256:* \
    && "$state_dir" == /* \
    && "$evidence_dir" == /* \
    && "$handoff_digest" =~ ^[0-9a-f]{64}$ ]] || return 1
  output_dir="$(dirname "$output")" || return 1
  temporary="$(mktemp "$output_dir/.sift-container-owner.XXXXXX")" || return 1
  if ! jq -n \
      --arg container_id "$container_id" \
      --arg controller_image "$controller_image" \
      --arg state_dir "$state_dir" \
      --arg evidence_dir "$evidence_dir" \
      --arg cleanup_handoff_digest "$handoff_digest" '
        {
          schema:"axiom.gcp.sift.container-owner.v1",
          container_id:$container_id,
          controller_image:$controller_image,
          state_dir:$state_dir,
          evidence_dir:$evidence_dir,
          cleanup_handoff_digest:$cleanup_handoff_digest
        }
      ' > "$temporary" \
      || ! chmod 0600 "$temporary" \
      || ! mv "$temporary" "$output"; then
    rm -f "$temporary"
    return 1
  fi
}

verify_sift_container_owner() {
  local input="$1"
  local controller_image="$2"
  local state_dir="$3"
  local evidence_dir="$4"
  [[ -f "$input" && ! -L "$input" ]] || return 1
  jq -e \
    --arg controller_image "$controller_image" \
    --arg state_dir "$state_dir" \
    --arg evidence_dir "$evidence_dir" '
      type == "object"
      and keys == [
        "cleanup_handoff_digest",
        "container_id",
        "controller_image",
        "evidence_dir",
        "schema",
        "state_dir"
      ]
      and .schema == "axiom.gcp.sift.container-owner.v1"
      and (.container_id | type) == "string"
      and (.container_id | test("^[0-9a-f]{64}$"))
      and .controller_image == $controller_image
      and (.controller_image | test("@sha256:[0-9a-f]{64}$"))
      and .state_dir == $state_dir
      and .evidence_dir == $evidence_dir
      and (.cleanup_handoff_digest | type) == "string"
      and (.cleanup_handoff_digest | test("^[0-9a-f]{64}$"))
    ' "$input" >/dev/null
}

write_sift_container_stopped_receipt() {
  local output="$1"
  local inspect_json="$2"
  local expected_id="$3"
  local expected_image="$4"
  local handoff_digest="$5"
  local output_dir temporary
  [[ -f "$inspect_json" && ! -L "$inspect_json" \
    && "$expected_id" =~ ^[0-9a-f]{64}$ \
    && "$expected_image" == *@sha256:* \
    && "$handoff_digest" =~ ^[0-9a-f]{64}$ ]] || return 1
  jq -e \
    --arg id "$expected_id" \
    --arg image "$expected_image" '
      type == "object"
      and .Id == $id
      and .Config.Image == $image
      and .State.Running == false
      and (.State.Status | IN("exited", "dead"))
      and (.State.ExitCode | type) == "number"
      and (.State.FinishedAt | type) == "string"
      and .State.FinishedAt != ""
      and .State.FinishedAt != "0001-01-01T00:00:00Z"
    ' "$inspect_json" >/dev/null || return 1
  output_dir="$(dirname "$output")" || return 1
  temporary="$(mktemp "$output_dir/.sift-container-stopped.XXXXXX")" || return 1
  if ! jq \
      --arg cleanup_handoff_digest "$handoff_digest" \
      --arg observed_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" '
        {
          schema:"axiom.gcp.sift.container-stopped.v1",
          container_id:.Id,
          controller_image:.Config.Image,
          status:.State.Status,
          running:.State.Running,
          exit_code:.State.ExitCode,
          finished_at:.State.FinishedAt,
          observed_at:$observed_at,
          cleanup_handoff_digest:$cleanup_handoff_digest
        }
      ' "$inspect_json" > "$temporary" \
      || ! chmod 0600 "$temporary" \
      || ! mv "$temporary" "$output"; then
    rm -f "$temporary"
    return 1
  fi
}

authorize_sift_container_cleanup() {
  local owner="$1"
  local stopped="$2"
  local nonce="$3"
  local controller_image="$4"
  local state_dir="$5"
  local evidence_dir="$6"
  local expected_digest provided_digest expected_id
  verify_sift_container_owner \
    "$owner" "$controller_image" "$state_dir" "$evidence_dir" || return 1
  [[ -f "$stopped" && ! -L "$stopped" ]] || return 1
  expected_digest="$(jq -er '.cleanup_handoff_digest' "$owner")" || return 1
  expected_id="$(jq -er '.container_id' "$owner")" || return 1
  provided_digest="$(sift_container_nonce_digest "$nonce")" || return 1
  [[ "$provided_digest" == "$expected_digest" ]] || return 1
  jq -e \
    --arg container_id "$expected_id" \
    --arg controller_image "$controller_image" \
    --arg cleanup_handoff_digest "$expected_digest" '
      type == "object"
      and keys == [
        "cleanup_handoff_digest",
        "container_id",
        "controller_image",
        "exit_code",
        "finished_at",
        "observed_at",
        "running",
        "schema",
        "status"
      ]
      and .schema == "axiom.gcp.sift.container-stopped.v1"
      and .container_id == $container_id
      and .controller_image == $controller_image
      and .cleanup_handoff_digest == $cleanup_handoff_digest
      and .running == false
      and (.status | IN("exited", "dead"))
      and (.exit_code | type) == "number"
      and (.finished_at | type) == "string"
      and (.finished_at | length) > 0
      and (.observed_at | type) == "string"
      and (.observed_at | length) > 0
    ' "$stopped" >/dev/null
}

write_sift_cleanup_container_owner() {
  local output="$1"
  local container_id="$2"
  local controller_image="$3"
  local run_container_id="$4"
  local handoff_digest="$5"
  local output_dir temporary
  [[ "$container_id" =~ ^[0-9a-f]{64}$ \
    && "$run_container_id" =~ ^[0-9a-f]{64}$ \
    && "$controller_image" == *@sha256:* \
    && "$handoff_digest" =~ ^[0-9a-f]{64}$ ]] || return 1
  output_dir="$(dirname "$output")" || return 1
  temporary="$(mktemp "$output_dir/.sift-cleanup-container-owner.XXXXXX")" \
    || return 1
  if ! jq -n \
      --arg container_id "$container_id" \
      --arg controller_image "$controller_image" \
      --arg run_container_id "$run_container_id" \
      --arg cleanup_handoff_digest "$handoff_digest" '
        {
          schema:"axiom.gcp.sift.cleanup-container-owner.v1",
          container_id:$container_id,
          controller_image:$controller_image,
          run_container_id:$run_container_id,
          cleanup_handoff_digest:$cleanup_handoff_digest
        }
      ' > "$temporary" \
      || ! chmod 0600 "$temporary" \
      || ! mv "$temporary" "$output"; then
    rm -f "$temporary"
    return 1
  fi
}

verify_sift_cleanup_container_owner() {
  local input="$1"
  local controller_image="$2"
  local run_container_id="$3"
  local handoff_digest="$4"
  [[ -f "$input" && ! -L "$input" ]] || return 1
  jq -e \
    --arg controller_image "$controller_image" \
    --arg run_container_id "$run_container_id" \
    --arg cleanup_handoff_digest "$handoff_digest" '
      type == "object"
      and keys == [
        "cleanup_handoff_digest",
        "container_id",
        "controller_image",
        "run_container_id",
        "schema"
      ]
      and .schema == "axiom.gcp.sift.cleanup-container-owner.v1"
      and (.container_id | type) == "string"
      and (.container_id | test("^[0-9a-f]{64}$"))
      and .controller_image == $controller_image
      and .run_container_id == $run_container_id
      and .cleanup_handoff_digest == $cleanup_handoff_digest
    ' "$input" >/dev/null
}
