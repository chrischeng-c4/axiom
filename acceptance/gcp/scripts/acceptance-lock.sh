#!/usr/bin/env bash

acceptance_lock_name() {
  printf '%s\n' "axiom-gcp-operator-acceptance-lock"
}

acceptance_lock_namespace() {
  printf '%s\n' "kube-system"
}

acceptance_lock_holder() {
  local project_id="$1"
  local run_id="$2"
  local acceptance_mode="$3"
  printf '%s/%s/%s\n' "$project_id" "$run_id" "$acceptance_mode"
}

acceptance_run_claim_path() {
  local claim_root="$1"
  local project_id="$2"
  local run_id="$3"
  local acceptance_mode="$4"
  local claim_digest

  claim_digest="$(
    acceptance_lock_holder "$project_id" "$run_id" "$acceptance_mode" \
      | openssl dgst -sha256 \
      | awk '{print $NF}'
  )" || return 1
  [[ "$claim_digest" =~ ^[0-9a-f]{64}$ ]] || return 1
  printf '%s/acceptance-run-%s.json\n' "$claim_root" "$claim_digest"
}

acceptance_cleanup_session_intent_path() {
  local intent_root="$1"
  local session_id="$2"
  [[ "$session_id" =~ ^[0-9a-f]{32}$ ]] || return 1
  printf '%s/%s.json\n' "$intent_root" "$session_id"
}

verify_acceptance_run_owner_identity() {
  local receipt="$1"
  local project_id="$2"
  local run_id="$3"
  local acceptance_mode="$4"
  local state_dir="$5"
  local evidence_dir="$6"

  [[ -f "$receipt" && ! -L "$receipt" ]] || return 1
  jq -e \
    --arg project_id "$project_id" \
    --arg run_id "$run_id" \
    --arg acceptance_mode "$acceptance_mode" \
    --arg state_dir "$state_dir" \
    --arg evidence_dir "$evidence_dir" '
      type == "object"
      and keys == ["acceptance_mode","acquisition_id","cleanup_handoff_digest","evidence_dir","owner_pgid","owner_pid","owner_start_token","project_id","run_id","schema","state_dir"]
      and .schema == "axiom.gcp.operator.acceptance-run-owner.v1"
      and .project_id == $project_id
      and .run_id == $run_id
      and .acceptance_mode == $acceptance_mode
      and (.acquisition_id | type) == "string"
      and (.acquisition_id | test("^[0-9a-f]{32}$"))
      and (.cleanup_handoff_digest | type) == "string"
      and (.cleanup_handoff_digest | test("^[0-9a-f]{64}$"))
      and .state_dir == $state_dir
      and .evidence_dir == $evidence_dir
      and (.owner_pid | type) == "string"
      and (.owner_pid | test("^[1-9][0-9]*$"))
      and (.owner_pgid | type) == "string"
      and (.owner_pgid | test("^[1-9][0-9]*$"))
      and (.owner_start_token | type) == "string"
      and (.owner_start_token | length) > 0
    ' "$receipt" >/dev/null
}

verify_acceptance_run_owner() {
  local receipt="$1"
  local project_id="$2"
  local run_id="$3"
  local acceptance_mode="$4"
  local acquisition_id="$5"
  local state_dir="$6"
  local evidence_dir="$7"
  local owner_pid="$8"
  local owner_pgid="$9"
  local owner_start_token="${10}"
  local cleanup_handoff_digest="${11}"

  verify_acceptance_run_owner_identity \
    "$receipt" "$project_id" "$run_id" "$acceptance_mode" \
    "$state_dir" "$evidence_dir" || return 1
  jq -e \
    --arg acquisition_id "$acquisition_id" \
    --arg owner_pid "$owner_pid" \
    --arg owner_pgid "$owner_pgid" \
    --arg owner_start_token "$owner_start_token" \
    --arg cleanup_handoff_digest "$cleanup_handoff_digest" '
      .acquisition_id == $acquisition_id
      and .owner_pid == $owner_pid
      and .owner_pgid == $owner_pgid
      and .owner_start_token == $owner_start_token
      and .cleanup_handoff_digest == $cleanup_handoff_digest
    ' "$receipt" >/dev/null
}

write_acceptance_run_owner() {
  local receipt="$1"
  local project_id="$2"
  local run_id="$3"
  local acceptance_mode="$4"
  local acquisition_id="$5"
  local state_dir="$6"
  local evidence_dir="$7"
  local owner_pid="$8"
  local owner_pgid="$9"
  local owner_start_token="${10}"
  local cleanup_handoff_digest="${11}"
  local receipt_dir temporary

  [[ "$acquisition_id" =~ ^[0-9a-f]{32}$ \
    && "$owner_pid" =~ ^[1-9][0-9]*$ \
    && "$owner_pgid" =~ ^[1-9][0-9]*$ \
    && -n "$owner_start_token" \
    && "$cleanup_handoff_digest" =~ ^[0-9a-f]{64}$ ]] || return 1
  receipt_dir="$(dirname "$receipt")" || return 1
  mkdir -p "$receipt_dir" || return 1
  temporary="$(mktemp "$receipt_dir/.acceptance-run-owner.XXXXXX")" || return 1
  if ! jq -n \
      --arg project_id "$project_id" \
      --arg run_id "$run_id" \
      --arg acceptance_mode "$acceptance_mode" \
      --arg acquisition_id "$acquisition_id" \
      --arg state_dir "$state_dir" \
      --arg evidence_dir "$evidence_dir" \
      --arg owner_pid "$owner_pid" \
      --arg owner_pgid "$owner_pgid" \
      --arg owner_start_token "$owner_start_token" \
      --arg cleanup_handoff_digest "$cleanup_handoff_digest" '
        {
          schema:"axiom.gcp.operator.acceptance-run-owner.v1",
          project_id:$project_id,
          run_id:$run_id,
          acceptance_mode:$acceptance_mode,
          acquisition_id:$acquisition_id,
          cleanup_handoff_digest:$cleanup_handoff_digest,
          state_dir:$state_dir,
          evidence_dir:$evidence_dir,
          owner_pid:$owner_pid,
          owner_pgid:$owner_pgid,
          owner_start_token:$owner_start_token
        }
      ' > "$temporary"; then
    rm -f "$temporary"
    return 1
  fi
  chmod 0600 "$temporary" || {
    rm -f "$temporary"
    return 1
  }
  mv "$temporary" "$receipt" || {
    rm -f "$temporary"
    return 1
  }
  verify_acceptance_run_owner \
    "$receipt" "$project_id" "$run_id" "$acceptance_mode" \
    "$acquisition_id" "$state_dir" "$evidence_dir" \
    "$owner_pid" "$owner_pgid" "$owner_start_token" \
    "$cleanup_handoff_digest" || {
    rm -f "$receipt"
    return 1
  }
}

write_acceptance_lock_intent() {
  local intent="$1"
  local project_id="$2"
  local run_id="$3"
  local acceptance_mode="$4"
  local acquisition_id="$5"
  local intent_dir temporary

  [[ "$acquisition_id" =~ ^[0-9a-f]{32}$ ]] || return 1
  intent_dir="$(dirname "$intent")" || return 1
  mkdir -p "$intent_dir" || return 1
  temporary="$(mktemp "$intent_dir/.acceptance-lock-intent.XXXXXX")" || return 1
  if ! jq -n \
    --arg project_id "$project_id" \
    --arg run_id "$run_id" \
    --arg acceptance_mode "$acceptance_mode" \
    --arg acquisition_id "$acquisition_id" \
    --arg name "$(acceptance_lock_name)" \
    --arg namespace "$(acceptance_lock_namespace)" \
    --arg holder "$(acceptance_lock_holder "$project_id" "$run_id" "$acceptance_mode")" '
      {
        schema:"axiom.gcp.operator.acceptance-lock-intent.v1",
        project_id:$project_id,
        run_id:$run_id,
        acceptance_mode:$acceptance_mode,
        acquisition_id:$acquisition_id,
        name:$name,
        namespace:$namespace,
        holder_identity:$holder
      }
    ' > "$temporary"; then
    rm -f "$temporary"
    return 1
  fi
  chmod 0600 "$temporary" || {
    rm -f "$temporary"
    return 1
  }
  mv "$temporary" "$intent" || {
    rm -f "$temporary"
    return 1
  }
  verify_acceptance_lock_intent \
    "$intent" "$project_id" "$run_id" "$acceptance_mode" || {
    rm -f "$intent"
    return 1
  }
}

verify_acceptance_lock_intent() {
  local intent="$1"
  local project_id="$2"
  local run_id="$3"
  local acceptance_mode="$4"

  [[ -f "$intent" && ! -L "$intent" ]] || return 1
  jq -e \
    --arg project_id "$project_id" \
    --arg run_id "$run_id" \
    --arg acceptance_mode "$acceptance_mode" \
    --arg name "$(acceptance_lock_name)" \
    --arg namespace "$(acceptance_lock_namespace)" \
    --arg holder "$(acceptance_lock_holder "$project_id" "$run_id" "$acceptance_mode")" '
      type == "object"
      and keys == ["acceptance_mode","acquisition_id","holder_identity","name","namespace","project_id","run_id","schema"]
      and .schema == "axiom.gcp.operator.acceptance-lock-intent.v1"
      and .project_id == $project_id
      and .run_id == $run_id
      and .acceptance_mode == $acceptance_mode
      and (.acquisition_id | type) == "string"
      and (.acquisition_id | test("^[0-9a-f]{32}$"))
      and .name == $name
      and .namespace == $namespace
      and .holder_identity == $holder
    ' "$intent" >/dev/null
}

acceptance_lock_manifest() {
  local project_id="$1"
  local run_id="$2"
  local acceptance_mode="$3"
  local acquisition_id="$4"
  local holder
  [[ "$acquisition_id" =~ ^[0-9a-f]{32}$ ]] || return 1
  holder="$(acceptance_lock_holder "$project_id" "$run_id" "$acceptance_mode")"
  jq -n \
    --arg name "$(acceptance_lock_name)" \
    --arg namespace "$(acceptance_lock_namespace)" \
    --arg project_id "$project_id" \
    --arg run_id "$run_id" \
    --arg acceptance_mode "$acceptance_mode" \
    --arg acquisition_id "$acquisition_id" \
    --arg holder "$holder" '
      {
        apiVersion:"coordination.k8s.io/v1",
        kind:"Lease",
        metadata:{
          name:$name,
          namespace:$namespace,
          labels:{
            "app.kubernetes.io/name":"axiom-gcp-operator-acceptance",
            "axiom.axiom.dev/project-id":$project_id,
            "axiom.axiom.dev/run-id":$run_id,
            "axiom.axiom.dev/acceptance-mode":$acceptance_mode
          },
          annotations:{"axiom.axiom.dev/acquisition-id":$acquisition_id}
        },
        spec:{holderIdentity:$holder}
      }
    '
}

verify_acceptance_lock_json() {
  local resource="$1"
  local project_id="$2"
  local run_id="$3"
  local acceptance_mode="$4"
  local acquisition_id="$5"
  [[ "$acquisition_id" =~ ^[0-9a-f]{32}$ ]] || return 1
  jq -e \
    --arg name "$(acceptance_lock_name)" \
    --arg namespace "$(acceptance_lock_namespace)" \
    --arg project_id "$project_id" \
    --arg run_id "$run_id" \
    --arg acceptance_mode "$acceptance_mode" \
    --arg acquisition_id "$acquisition_id" \
    --arg holder "$(acceptance_lock_holder "$project_id" "$run_id" "$acceptance_mode")" '
      .apiVersion == "coordination.k8s.io/v1"
      and .kind == "Lease"
      and .metadata.name == $name
      and .metadata.namespace == $namespace
      and (.metadata.uid | type) == "string"
      and (.metadata.uid | length) > 0
      and (.metadata.resourceVersion | type) == "string"
      and (.metadata.resourceVersion | length) > 0
      and .metadata.labels["app.kubernetes.io/name"] == "axiom-gcp-operator-acceptance"
      and .metadata.labels["axiom.axiom.dev/project-id"] == $project_id
      and .metadata.labels["axiom.axiom.dev/run-id"] == $run_id
      and .metadata.labels["axiom.axiom.dev/acceptance-mode"] == $acceptance_mode
      and .metadata.annotations["axiom.axiom.dev/acquisition-id"] == $acquisition_id
      and .spec.holderIdentity == $holder
    ' <<<"$resource" >/dev/null
}

write_acceptance_lock_receipt() {
  local receipt="$1"
  local resource="$2"
  local project_id="$3"
  local run_id="$4"
  local acceptance_mode="$5"
  local acquisition_id="$6"
  local receipt_dir temporary

  verify_acceptance_lock_json \
    "$resource" "$project_id" "$run_id" "$acceptance_mode" "$acquisition_id" || return 1
  receipt_dir="$(dirname "$receipt")" || return 1
  mkdir -p "$receipt_dir" || return 1
  temporary="$(mktemp "$receipt_dir/.acceptance-lock.XXXXXX")" || return 1
  if ! jq -n \
    --arg project_id "$project_id" \
    --arg run_id "$run_id" \
    --arg acceptance_mode "$acceptance_mode" \
    --arg acquisition_id "$acquisition_id" \
    --arg name "$(acceptance_lock_name)" \
    --arg namespace "$(acceptance_lock_namespace)" \
    --arg holder "$(acceptance_lock_holder "$project_id" "$run_id" "$acceptance_mode")" \
    --arg uid "$(jq -r '.metadata.uid' <<<"$resource")" \
    --arg resource_version "$(jq -r '.metadata.resourceVersion' <<<"$resource")" '
      {
        schema:"axiom.gcp.operator.acceptance-lock.v1",
        project_id:$project_id,
        run_id:$run_id,
        acceptance_mode:$acceptance_mode,
        acquisition_id:$acquisition_id,
        name:$name,
        namespace:$namespace,
        holder_identity:$holder,
        uid:$uid,
        resource_version:$resource_version
      }
    ' > "$temporary"; then
    rm -f "$temporary"
    return 1
  fi
  chmod 0600 "$temporary" || {
    rm -f "$temporary"
    return 1
  }
  mv "$temporary" "$receipt" || {
    rm -f "$temporary"
    return 1
  }
  verify_acceptance_lock_receipt \
    "$receipt" "$resource" "$project_id" "$run_id" "$acceptance_mode" || {
    rm -f "$receipt"
    return 1
  }
}

verify_acceptance_lock_receipt() {
  local receipt="$1"
  local resource="$2"
  local project_id="$3"
  local run_id="$4"
  local acceptance_mode="$5"
  local acquisition_id

  verify_acceptance_lock_receipt_identity \
    "$receipt" "$project_id" "$run_id" "$acceptance_mode" || return 1
  acquisition_id="$(jq -er \
    '.acquisition_id | strings | select(test("^[0-9a-f]{32}$"))' "$receipt")" \
    || return 1
  verify_acceptance_lock_json \
    "$resource" "$project_id" "$run_id" "$acceptance_mode" "$acquisition_id" || return 1
  jq -e \
    --arg project_id "$project_id" \
    --arg run_id "$run_id" \
    --arg acceptance_mode "$acceptance_mode" \
    --arg acquisition_id "$acquisition_id" \
    --arg name "$(acceptance_lock_name)" \
    --arg namespace "$(acceptance_lock_namespace)" \
    --arg holder "$(acceptance_lock_holder "$project_id" "$run_id" "$acceptance_mode")" \
    --arg uid "$(jq -r '.metadata.uid' <<<"$resource")" \
    --arg resource_version "$(jq -r '.metadata.resourceVersion' <<<"$resource")" '
      type == "object"
      and keys == ["acceptance_mode","acquisition_id","holder_identity","name","namespace","project_id","resource_version","run_id","schema","uid"]
      and .schema == "axiom.gcp.operator.acceptance-lock.v1"
      and .project_id == $project_id
      and .run_id == $run_id
      and .acceptance_mode == $acceptance_mode
      and .acquisition_id == $acquisition_id
      and .name == $name
      and .namespace == $namespace
      and .holder_identity == $holder
      and .uid == $uid
      and .resource_version == $resource_version
    ' "$receipt" >/dev/null
}

verify_acceptance_lock_receipt_owner() {
  local receipt="$1"
  local resource="$2"
  local project_id="$3"
  local run_id="$4"
  local acceptance_mode="$5"
  local acquisition_id uid

  verify_acceptance_lock_receipt_identity \
    "$receipt" "$project_id" "$run_id" "$acceptance_mode" || return 1
  acquisition_id="$(jq -er \
    '.acquisition_id | strings | select(test("^[0-9a-f]{32}$"))' "$receipt")" \
    || return 1
  uid="$(jq -er '.uid | strings | select(length > 0)' "$receipt")" || return 1
  verify_acceptance_lock_json \
    "$resource" "$project_id" "$run_id" "$acceptance_mode" "$acquisition_id" || return 1
  [[ "$(jq -r '.metadata.uid' <<<"$resource")" == "$uid" ]]
}

acceptance_cleanup_session_patch() {
  local resource="$1"
  local acquisition_id="$2"
  local session_id="$3"
  local started_at="$4"
  local uid resource_version holder

  [[ "$acquisition_id" =~ ^[0-9a-f]{32}$ \
    && "$session_id" =~ ^[0-9a-f]{32}$ \
    && -n "$started_at" ]] || return 1
  uid="$(jq -er '.metadata.uid | strings | select(length > 0)' <<<"$resource")" || return 1
  resource_version="$(jq -er \
    '.metadata.resourceVersion | strings | select(length > 0)' <<<"$resource")" || return 1
  holder="$(jq -er '.spec.holderIdentity | strings | select(length > 0)' <<<"$resource")" \
    || return 1
  jq -cn \
    --arg uid "$uid" \
    --arg resource_version "$resource_version" \
    --arg holder "$holder" \
    --arg acquisition_id "$acquisition_id" \
    --arg session_id "$session_id" \
    --arg started_at "$started_at" '
      [
        {op:"test",path:"/metadata/uid",value:$uid},
        {op:"test",path:"/metadata/resourceVersion",value:$resource_version},
        {op:"test",path:"/metadata/annotations/axiom.axiom.dev~1acquisition-id",value:$acquisition_id},
        {op:"test",path:"/spec/holderIdentity",value:$holder},
        {op:"add",path:"/metadata/annotations/axiom.axiom.dev~1cleanup-session-id",value:$session_id},
        {op:"add",path:"/metadata/annotations/axiom.axiom.dev~1cleanup-started-at",value:$started_at}
      ]
    '
}

acceptance_cleanup_session_takeover_patch() {
  local resource="$1"
  local project_id="$2"
  local run_id="$3"
  local acceptance_mode="$4"
  local acquisition_id="$5"
  local previous_session_id="$6"
  local next_session_id="$7"
  local started_at="$8"
  local uid resource_version holder

  [[ "$acquisition_id" =~ ^[0-9a-f]{32}$ \
    && "$previous_session_id" =~ ^[0-9a-f]{32}$ \
    && "$next_session_id" =~ ^[0-9a-f]{32}$ \
    && "$previous_session_id" != "$next_session_id" \
    && -n "$started_at" ]] || return 1
  verify_acceptance_cleanup_session_json \
    "$resource" "$project_id" "$run_id" "$acceptance_mode" \
    "$acquisition_id" "$previous_session_id" || return 1
  uid="$(jq -er '.metadata.uid | strings | select(length > 0)' <<<"$resource")" \
    || return 1
  resource_version="$(jq -er \
    '.metadata.resourceVersion | strings | select(length > 0)' <<<"$resource")" \
    || return 1
  holder="$(jq -er '.spec.holderIdentity | strings | select(length > 0)' <<<"$resource")" \
    || return 1
  jq -cn \
    --arg uid "$uid" \
    --arg resource_version "$resource_version" \
    --arg holder "$holder" \
    --arg acquisition_id "$acquisition_id" \
    --arg previous_session_id "$previous_session_id" \
    --arg next_session_id "$next_session_id" \
    --arg started_at "$started_at" '
      [
        {op:"test",path:"/metadata/uid",value:$uid},
        {op:"test",path:"/metadata/resourceVersion",value:$resource_version},
        {op:"test",path:"/metadata/annotations/axiom.axiom.dev~1acquisition-id",value:$acquisition_id},
        {op:"test",path:"/spec/holderIdentity",value:$holder},
        {op:"test",path:"/metadata/annotations/axiom.axiom.dev~1cleanup-session-id",value:$previous_session_id},
        {op:"replace",path:"/metadata/annotations/axiom.axiom.dev~1cleanup-session-id",value:$next_session_id},
        {op:"replace",path:"/metadata/annotations/axiom.axiom.dev~1cleanup-started-at",value:$started_at}
      ]
    '
}

verify_acceptance_cleanup_session_json() {
  local resource="$1"
  local project_id="$2"
  local run_id="$3"
  local acceptance_mode="$4"
  local acquisition_id="$5"
  local session_id="$6"

  verify_acceptance_lock_json \
    "$resource" "$project_id" "$run_id" "$acceptance_mode" "$acquisition_id" || return 1
  [[ "$session_id" =~ ^[0-9a-f]{32}$ ]] || return 1
  jq -e --arg session_id "$session_id" '
    .metadata.annotations["axiom.axiom.dev/cleanup-session-id"] == $session_id
    and (.metadata.annotations["axiom.axiom.dev/cleanup-started-at"] | type) == "string"
    and (.metadata.annotations["axiom.axiom.dev/cleanup-started-at"] | length) > 0
  ' <<<"$resource" >/dev/null
}

write_acceptance_cleanup_session_intent() {
  local intent="$1"
  local project_id="$2"
  local run_id="$3"
  local acceptance_mode="$4"
  local acquisition_id="$5"
  local session_id="$6"
  local cleanup_owner_pid="$7"
  local cleanup_owner_start_token="$8"
  local intent_dir temporary

  [[ "$acquisition_id" =~ ^[0-9a-f]{32}$ \
    && "$session_id" =~ ^[0-9a-f]{32}$ \
    && "$cleanup_owner_pid" =~ ^[1-9][0-9]*$ \
    && -n "$cleanup_owner_start_token" ]] || return 1
  intent_dir="$(dirname "$intent")" || return 1
  mkdir -p "$intent_dir" || return 1
  temporary="$(mktemp "$intent_dir/.acceptance-cleanup-session-intent.XXXXXX")" || return 1
  if ! jq -n \
    --arg project_id "$project_id" \
    --arg run_id "$run_id" \
    --arg acceptance_mode "$acceptance_mode" \
    --arg acquisition_id "$acquisition_id" \
    --arg cleanup_session_id "$session_id" \
    --arg cleanup_owner_pid "$cleanup_owner_pid" \
    --arg cleanup_owner_start_token "$cleanup_owner_start_token" \
    --arg name "$(acceptance_lock_name)" \
    --arg namespace "$(acceptance_lock_namespace)" '
      {
        schema:"axiom.gcp.operator.acceptance-cleanup-session-intent.v1",
        project_id:$project_id,
        run_id:$run_id,
        acceptance_mode:$acceptance_mode,
        acquisition_id:$acquisition_id,
        cleanup_session_id:$cleanup_session_id,
        cleanup_owner_pid:$cleanup_owner_pid,
        cleanup_owner_start_token:$cleanup_owner_start_token,
        name:$name,
        namespace:$namespace
      }
    ' > "$temporary"; then
    rm -f "$temporary"
    return 1
  fi
  chmod 0600 "$temporary" || {
    rm -f "$temporary"
    return 1
  }
  mv "$temporary" "$intent" || {
    rm -f "$temporary"
    return 1
  }
  verify_acceptance_cleanup_session_intent \
    "$intent" "$project_id" "$run_id" "$acceptance_mode" \
    "$acquisition_id" "$session_id" \
    "$cleanup_owner_pid" "$cleanup_owner_start_token" || {
    rm -f "$intent"
    return 1
  }
}

verify_acceptance_cleanup_session_intent_identity() {
  local intent="$1"
  local project_id="$2"
  local run_id="$3"
  local acceptance_mode="$4"
  local acquisition_id="$5"
  local session_id="$6"

  [[ -f "$intent" && ! -L "$intent" ]] || return 1
  jq -e \
    --arg project_id "$project_id" \
    --arg run_id "$run_id" \
    --arg acceptance_mode "$acceptance_mode" \
    --arg acquisition_id "$acquisition_id" \
    --arg cleanup_session_id "$session_id" \
    --arg name "$(acceptance_lock_name)" \
    --arg namespace "$(acceptance_lock_namespace)" '
      type == "object"
      and keys == ["acceptance_mode","acquisition_id","cleanup_owner_pid","cleanup_owner_start_token","cleanup_session_id","name","namespace","project_id","run_id","schema"]
      and .schema == "axiom.gcp.operator.acceptance-cleanup-session-intent.v1"
      and .project_id == $project_id
      and .run_id == $run_id
      and .acceptance_mode == $acceptance_mode
      and .acquisition_id == $acquisition_id
      and (.acquisition_id | test("^[0-9a-f]{32}$"))
      and .cleanup_session_id == $cleanup_session_id
      and (.cleanup_session_id | test("^[0-9a-f]{32}$"))
      and (.cleanup_owner_pid | type) == "string"
      and (.cleanup_owner_pid | test("^[1-9][0-9]*$"))
      and (.cleanup_owner_start_token | type) == "string"
      and (.cleanup_owner_start_token | length) > 0
      and .name == $name
      and .namespace == $namespace
    ' "$intent" >/dev/null
}

verify_acceptance_cleanup_session_intent() {
  local intent="$1"
  local project_id="$2"
  local run_id="$3"
  local acceptance_mode="$4"
  local acquisition_id="$5"
  local session_id="$6"
  local cleanup_owner_pid="$7"
  local cleanup_owner_start_token="$8"

  verify_acceptance_cleanup_session_intent_identity \
    "$intent" "$project_id" "$run_id" "$acceptance_mode" \
    "$acquisition_id" "$session_id" || return 1
  jq -e \
    --arg cleanup_owner_pid "$cleanup_owner_pid" \
    --arg cleanup_owner_start_token "$cleanup_owner_start_token" '
      .cleanup_owner_pid == $cleanup_owner_pid
      and .cleanup_owner_start_token == $cleanup_owner_start_token
    ' "$intent" >/dev/null
}

write_acceptance_cleanup_session_receipt() {
  local receipt="$1"
  local resource="$2"
  local project_id="$3"
  local run_id="$4"
  local acceptance_mode="$5"
  local acquisition_id="$6"
  local session_id="$7"
  local receipt_dir temporary

  verify_acceptance_cleanup_session_json \
    "$resource" "$project_id" "$run_id" "$acceptance_mode" \
    "$acquisition_id" "$session_id" || return 1
  receipt_dir="$(dirname "$receipt")" || return 1
  mkdir -p "$receipt_dir" || return 1
  temporary="$(mktemp "$receipt_dir/.acceptance-cleanup-session.XXXXXX")" || return 1
  if ! jq -n \
    --arg project_id "$project_id" \
    --arg run_id "$run_id" \
    --arg acceptance_mode "$acceptance_mode" \
    --arg acquisition_id "$acquisition_id" \
    --arg cleanup_session_id "$session_id" \
    --arg name "$(acceptance_lock_name)" \
    --arg namespace "$(acceptance_lock_namespace)" \
    --arg holder "$(acceptance_lock_holder "$project_id" "$run_id" "$acceptance_mode")" \
    --arg uid "$(jq -r '.metadata.uid' <<<"$resource")" \
    --arg resource_version "$(jq -r '.metadata.resourceVersion' <<<"$resource")" \
    --arg started_at "$(jq -r '.metadata.annotations["axiom.axiom.dev/cleanup-started-at"]' <<<"$resource")" '
      {
        schema:"axiom.gcp.operator.acceptance-cleanup-session.v1",
        project_id:$project_id,
        run_id:$run_id,
        acceptance_mode:$acceptance_mode,
        acquisition_id:$acquisition_id,
        cleanup_session_id:$cleanup_session_id,
        name:$name,
        namespace:$namespace,
        holder_identity:$holder,
        uid:$uid,
        resource_version:$resource_version,
        started_at:$started_at
      }
    ' > "$temporary"; then
    rm -f "$temporary"
    return 1
  fi
  chmod 0600 "$temporary" || {
    rm -f "$temporary"
    return 1
  }
  mv "$temporary" "$receipt" || {
    rm -f "$temporary"
    return 1
  }
  verify_acceptance_cleanup_session_receipt \
    "$receipt" "$resource" "$project_id" "$run_id" "$acceptance_mode" \
    "$acquisition_id" "$session_id" || {
    rm -f "$receipt"
    return 1
  }
}

verify_acceptance_cleanup_session_receipt() {
  local receipt="$1"
  local resource="$2"
  local project_id="$3"
  local run_id="$4"
  local acceptance_mode="$5"
  local acquisition_id="$6"
  local session_id="$7"

  [[ -f "$receipt" && ! -L "$receipt" ]] || return 1
  verify_acceptance_cleanup_session_json \
    "$resource" "$project_id" "$run_id" "$acceptance_mode" \
    "$acquisition_id" "$session_id" || return 1
  jq -e \
    --arg project_id "$project_id" \
    --arg run_id "$run_id" \
    --arg acceptance_mode "$acceptance_mode" \
    --arg acquisition_id "$acquisition_id" \
    --arg cleanup_session_id "$session_id" \
    --arg name "$(acceptance_lock_name)" \
    --arg namespace "$(acceptance_lock_namespace)" \
    --arg holder "$(acceptance_lock_holder "$project_id" "$run_id" "$acceptance_mode")" \
    --arg uid "$(jq -r '.metadata.uid' <<<"$resource")" \
    --arg resource_version "$(jq -r '.metadata.resourceVersion' <<<"$resource")" \
    --arg started_at "$(jq -r '.metadata.annotations["axiom.axiom.dev/cleanup-started-at"]' <<<"$resource")" '
      type == "object"
      and keys == ["acceptance_mode","acquisition_id","cleanup_session_id","holder_identity","name","namespace","project_id","resource_version","run_id","schema","started_at","uid"]
      and .schema == "axiom.gcp.operator.acceptance-cleanup-session.v1"
      and .project_id == $project_id
      and .run_id == $run_id
      and .acceptance_mode == $acceptance_mode
      and .acquisition_id == $acquisition_id
      and .cleanup_session_id == $cleanup_session_id
      and .name == $name
      and .namespace == $namespace
      and .holder_identity == $holder
      and .uid == $uid
      and .resource_version == $resource_version
      and .started_at == $started_at
    ' "$receipt" >/dev/null
}

verify_acceptance_lock_receipt_identity() {
  local receipt="$1"
  local project_id="$2"
  local run_id="$3"
  local acceptance_mode="$4"

  [[ -f "$receipt" && ! -L "$receipt" ]] || return 1
  jq -e \
    --arg project_id "$project_id" \
    --arg run_id "$run_id" \
    --arg acceptance_mode "$acceptance_mode" \
    --arg name "$(acceptance_lock_name)" \
    --arg namespace "$(acceptance_lock_namespace)" \
    --arg holder "$(acceptance_lock_holder "$project_id" "$run_id" "$acceptance_mode")" '
      type == "object"
      and keys == ["acceptance_mode","acquisition_id","holder_identity","name","namespace","project_id","resource_version","run_id","schema","uid"]
      and .schema == "axiom.gcp.operator.acceptance-lock.v1"
      and .project_id == $project_id
      and .run_id == $run_id
      and .acceptance_mode == $acceptance_mode
      and (.acquisition_id | type) == "string"
      and (.acquisition_id | test("^[0-9a-f]{32}$"))
      and .name == $name
      and .namespace == $namespace
      and .holder_identity == $holder
      and (.uid | type) == "string"
      and (.uid | length) > 0
      and (.resource_version | type) == "string"
      and (.resource_version | length) > 0
    ' "$receipt" >/dev/null
}

write_acceptance_lock_release_receipt() {
  local receipt="$1"
  local lock_receipt="$2"
  local temporary
  [[ -f "$lock_receipt" && ! -L "$lock_receipt" ]] || return 1
  mkdir -p "$(dirname "$receipt")" || return 1
  temporary="$(mktemp "$(dirname "$receipt")/.acceptance-lock-release.XXXXXX")" || return 1
  if ! jq '
    {
      schema:"axiom.gcp.operator.acceptance-lock-release.v1",
      project_id,
      run_id,
      acceptance_mode,
      acquisition_id,
      name,
      namespace,
      holder_identity,
      uid,
      resource_version
    }
  ' "$lock_receipt" > "$temporary"; then
    rm -f "$temporary"
    return 1
  fi
  chmod 0600 "$temporary" || {
    rm -f "$temporary"
    return 1
  }
  mv "$temporary" "$receipt" || {
    rm -f "$temporary"
    return 1
  }
  verify_acceptance_lock_release_receipt "$receipt" "$lock_receipt" || {
    rm -f "$receipt"
    return 1
  }
}

verify_acceptance_lock_release_receipt() {
  local receipt="$1"
  local lock_receipt="$2"

  [[ -f "$receipt" && ! -L "$receipt" \
    && -f "$lock_receipt" && ! -L "$lock_receipt" ]] || return 1
  jq -e --slurpfile lock "$lock_receipt" '
    type == "object"
    and keys == ["acceptance_mode","acquisition_id","holder_identity","name","namespace","project_id","resource_version","run_id","schema","uid"]
    and .schema == "axiom.gcp.operator.acceptance-lock-release.v1"
    and .project_id == $lock[0].project_id
    and .run_id == $lock[0].run_id
    and .acceptance_mode == $lock[0].acceptance_mode
    and .acquisition_id == $lock[0].acquisition_id
    and .name == $lock[0].name
    and .namespace == $lock[0].namespace
    and .holder_identity == $lock[0].holder_identity
    and .uid == $lock[0].uid
    and .resource_version == $lock[0].resource_version
  ' "$receipt" >/dev/null
}
