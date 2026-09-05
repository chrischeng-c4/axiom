#!/usr/bin/env bash

KUBERNETES_ACCEPTANCE_OWNER="gcp-operator-acceptance"

cleanup_lumen_auth_delegation_bindings_for_mode() {
  local acceptance_mode="$1"
  case "$acceptance_mode" in
    lumen-auth|lumen-sift)
      kubectl delete clusterrolebinding \
        -l app.kubernetes.io/component=auth-delegation,app.kubernetes.io/name=lumen \
        --ignore-not-found --wait=true --timeout=180s >/dev/null 2>&1 || true
      ;;
    sift|tape) ;;
    *) return 1 ;;
  esac
}

kubernetes_ownership_identity_valid() {
  local project_id="$1"
  local run_id="$2"
  local acquisition_id="$3"
  [[ "$project_id" =~ ^[a-z][a-z0-9-]{4,29}$ \
    && "$run_id" =~ ^[a-z0-9][a-z0-9-]{0,17}$ \
    && "$acquisition_id" =~ ^[0-9a-f]{32}$ ]]
}

kubernetes_ownership_resource_values() {
  local resource_type="$1"
  case "$resource_type" in
    namespace)
      KUBERNETES_OWNERSHIP_API_VERSION="v1"
      KUBERNETES_OWNERSHIP_KIND="Namespace"
      KUBERNETES_OWNERSHIP_CLI_RESOURCE="namespace"
      ;;
    customresourcedefinition)
      KUBERNETES_OWNERSHIP_API_VERSION="apiextensions.k8s.io/v1"
      KUBERNETES_OWNERSHIP_KIND="CustomResourceDefinition"
      KUBERNETES_OWNERSHIP_CLI_RESOURCE="customresourcedefinition"
      ;;
    *) return 1 ;;
  esac
}

kubernetes_ownership_file_slug() {
  local resource_type="$1"
  local name="$2"
  [[ "$name" =~ ^[a-z0-9]([-a-z0-9.]*[a-z0-9])?$ ]] || return 1
  kubernetes_ownership_resource_values "$resource_type" || return 1
  printf '%s-%s\n' "$resource_type" "$name"
}

kubernetes_ownership_intent_path() {
  local root="$1"
  local slug
  slug="$(kubernetes_ownership_file_slug "$2" "$3")" || return 1
  printf '%s/%s.intent.json\n' "$root" "$slug"
}

kubernetes_ownership_receipt_path() {
  local root="$1"
  local slug
  slug="$(kubernetes_ownership_file_slug "$2" "$3")" || return 1
  printf '%s/%s.json\n' "$root" "$slug"
}

kubernetes_ownership_deletion_path() {
  local root="$1"
  local slug
  slug="$(kubernetes_ownership_file_slug "$2" "$3")" || return 1
  printf '%s/deleted-%s.json\n' "$root" "$slug"
}

kubernetes_ownership_manifest_path() {
  local root="$1"
  local slug
  slug="$(kubernetes_ownership_file_slug "$2" "$3")" || return 1
  printf '%s/%s.manifest.json\n' "$root" "$slug"
}

kubernetes_ownership_file_sha256() {
  local input="$1"
  openssl dgst -sha256 "$input" | awk '{print $NF}'
}

kubernetes_api_error_is_not_found() {
  local error_file="$1"
  grep -Eiq '(^|[^[:alpha:]])(not[ _-]?found|404)([^[:alpha:]]|$)' \
    "$error_file"
}

kubernetes_get_resource_json() {
  local resource_type="$1"
  local name="$2"
  kubernetes_ownership_resource_values "$resource_type" || return 1
  kubectl get "$KUBERNETES_OWNERSHIP_CLI_RESOURCE" "$name" -o json
}

require_kubernetes_resource_absent() {
  local resource_type="$1"
  local name="$2"
  local error_file
  kubernetes_ownership_file_slug "$resource_type" "$name" >/dev/null || return 1
  error_file="$(mktemp "${TMPDIR:-/tmp}/sift-kubernetes-absence.XXXXXX")"
  if kubernetes_get_resource_json "$resource_type" "$name" \
      >/dev/null 2> "$error_file"; then
    echo "$resource_type $name already exists; refusing to reuse it" >&2
    rm -f "$error_file"
    return 1
  fi
  if kubernetes_api_error_is_not_found "$error_file"; then
    rm -f "$error_file"
    return 0
  fi
  echo "could not prove that $resource_type $name is absent:" >&2
  cat "$error_file" >&2
  rm -f "$error_file"
  return 1
}

verify_kubernetes_ownership_intent() {
  local intent="$1"
  local resource_type="$2"
  local name="$3"
  local project_id="$4"
  local run_id="$5"
  local acquisition_id="$6"
  kubernetes_ownership_resource_values "$resource_type" || return 1
  [[ -f "$intent" && ! -L "$intent" ]] || return 1
  jq -e \
    --arg resource_type "$resource_type" --arg api_version "$KUBERNETES_OWNERSHIP_API_VERSION" \
    --arg kind "$KUBERNETES_OWNERSHIP_KIND" --arg name "$name" \
    --arg project_id "$project_id" --arg run_id "$run_id" \
    --arg acquisition_id "$acquisition_id" '
      type == "object"
      and (
        (
          .schema == "axiom.gcp.kubernetes-ownership-intent.v1"
          and keys == [
            "acceptance_acquisition_id", "api_version", "kind", "name",
            "project_id", "resource_type", "run_id", "schema"
          ]
        )
        or (
          .schema == "axiom.gcp.kubernetes-ownership-intent.v2"
          and keys == [
            "acceptance_acquisition_id", "api_version", "kind",
            "manifest_sha256", "name", "project_id",
            "request_deadline_unix", "resource_type", "run_id", "schema"
          ]
          and (.manifest_sha256 | type) == "string"
          and (.manifest_sha256 | test("^[0-9a-f]{64}$"))
          and (.request_deadline_unix | type) == "number"
          and .request_deadline_unix > 0
          and (.request_deadline_unix | floor) == .request_deadline_unix
        )
      )
      and .resource_type == $resource_type
      and .api_version == $api_version
      and .kind == $kind
      and .name == $name
      and .project_id == $project_id
      and .run_id == $run_id
      and .acceptance_acquisition_id == $acquisition_id
    ' "$intent" >/dev/null
}

write_kubernetes_ownership_create_intent() {
  local intent="$1"
  local resource_type="$2"
  local name="$3"
  local project_id="$4"
  local run_id="$5"
  local acquisition_id="$6"
  local manifest="$7"
  local timeout_seconds="$8"
  local manifest_sha256 request_deadline temporary grace_seconds
  kubernetes_ownership_identity_valid \
    "$project_id" "$run_id" "$acquisition_id" || return 1
  kubernetes_ownership_resource_values "$resource_type" || return 1
  [[ -f "$manifest" && ! -L "$manifest" \
    && "$timeout_seconds" =~ ^[1-9][0-9]*$ \
    && "$timeout_seconds" -le 120 ]] || return 1
  grace_seconds="${KUBERNETES_OWNERSHIP_CREATE_GRACE_SECONDS:-5}"
  [[ "$grace_seconds" =~ ^[0-9]+$ && "$grace_seconds" -le 30 ]] \
    || return 1
  manifest_sha256="$(kubernetes_ownership_file_sha256 "$manifest")" \
    || return 1
  [[ "$manifest_sha256" =~ ^[0-9a-f]{64}$ ]] || return 1
  request_deadline=$(( $(date +%s) + timeout_seconds + grace_seconds ))
  if [[ -e "$intent" ]]; then
    verify_kubernetes_ownership_intent \
      "$intent" "$resource_type" "$name" "$project_id" "$run_id" \
      "$acquisition_id" || return 1
    jq -e --arg sha "$manifest_sha256" \
      '.schema == "axiom.gcp.kubernetes-ownership-intent.v2"
       and .manifest_sha256 == $sha' "$intent" >/dev/null
    return
  fi
  mkdir -p "$(dirname "$intent")"
  temporary="$(mktemp "$(dirname "$intent")/.ownership-intent.XXXXXX")"
  jq -n \
    --arg resource_type "$resource_type" \
    --arg api_version "$KUBERNETES_OWNERSHIP_API_VERSION" \
    --arg kind "$KUBERNETES_OWNERSHIP_KIND" --arg name "$name" \
    --arg project_id "$project_id" --arg run_id "$run_id" \
    --arg acquisition_id "$acquisition_id" \
    --arg manifest_sha256 "$manifest_sha256" \
    --argjson request_deadline_unix "$request_deadline" '
      {
        schema:"axiom.gcp.kubernetes-ownership-intent.v2",
        resource_type:$resource_type,
        api_version:$api_version,
        kind:$kind,
        name:$name,
        project_id:$project_id,
        run_id:$run_id,
        acceptance_acquisition_id:$acquisition_id,
        manifest_sha256:$manifest_sha256,
        request_deadline_unix:$request_deadline_unix
      }
    ' > "$temporary" || {
      rm -f "$temporary"
      return 1
    }
  chmod 0600 "$temporary"
  mv "$temporary" "$intent"
}

write_kubernetes_ownership_intent() {
  local intent="$1"
  local resource_type="$2"
  local name="$3"
  local project_id="$4"
  local run_id="$5"
  local acquisition_id="$6"
  local temporary
  kubernetes_ownership_identity_valid \
    "$project_id" "$run_id" "$acquisition_id" || return 1
  kubernetes_ownership_resource_values "$resource_type" || return 1
  if [[ -e "$intent" ]]; then
    verify_kubernetes_ownership_intent \
      "$intent" "$resource_type" "$name" "$project_id" "$run_id" \
      "$acquisition_id"
    return
  fi
  mkdir -p "$(dirname "$intent")"
  temporary="$(mktemp "$(dirname "$intent")/.ownership-intent.XXXXXX")"
  jq -n \
    --arg resource_type "$resource_type" --arg api_version "$KUBERNETES_OWNERSHIP_API_VERSION" \
    --arg kind "$KUBERNETES_OWNERSHIP_KIND" --arg name "$name" \
    --arg project_id "$project_id" --arg run_id "$run_id" \
    --arg acquisition_id "$acquisition_id" '
      {
        schema:"axiom.gcp.kubernetes-ownership-intent.v1",
        resource_type:$resource_type,
        api_version:$api_version,
        kind:$kind,
        name:$name,
        project_id:$project_id,
        run_id:$run_id,
        acceptance_acquisition_id:$acquisition_id
      }
    ' > "$temporary" || {
      rm -f "$temporary"
      return 1
    }
  chmod 0600 "$temporary"
  mv "$temporary" "$intent"
}

verify_owned_kubernetes_resource_json() {
  local resource="$1"
  local resource_type="$2"
  local name="$3"
  local project_id="$4"
  local run_id="$5"
  local acquisition_id="$6"
  kubernetes_ownership_resource_values "$resource_type" || return 1
  jq -e \
    --arg api_version "$KUBERNETES_OWNERSHIP_API_VERSION" \
    --arg kind "$KUBERNETES_OWNERSHIP_KIND" --arg name "$name" \
    --arg owner "$KUBERNETES_ACCEPTANCE_OWNER" \
    --arg project_id "$project_id" --arg run_id "$run_id" \
    --arg acquisition_id "$acquisition_id" '
      .apiVersion == $api_version
      and .kind == $kind
      and .metadata.name == $name
      and .metadata.labels["axiom.axiom.dev/acceptance-owner"] == $owner
      and .metadata.labels["axiom.axiom.dev/acceptance-project"] == $project_id
      and .metadata.labels["axiom.axiom.dev/acceptance-run-id"] == $run_id
      and .metadata.labels["axiom.axiom.dev/acceptance-acquisition-id"] == $acquisition_id
    ' <<<"$resource" >/dev/null
}

verify_kubernetes_ownership_receipt() {
  local receipt="$1"
  local resource="$2"
  local resource_type="$3"
  local name="$4"
  local project_id="$5"
  local run_id="$6"
  local acquisition_id="$7"
  local live_uid
  kubernetes_ownership_resource_values "$resource_type" || return 1
  verify_owned_kubernetes_resource_json \
    "$resource" "$resource_type" "$name" "$project_id" "$run_id" \
    "$acquisition_id" || return 1
  live_uid="$(jq -er '.metadata.uid | strings | select(length > 0)' \
    <<<"$resource")" || return 1
  [[ -f "$receipt" && ! -L "$receipt" ]] || return 1
  jq -e \
    --arg resource_type "$resource_type" --arg api_version "$KUBERNETES_OWNERSHIP_API_VERSION" \
    --arg kind "$KUBERNETES_OWNERSHIP_KIND" --arg name "$name" \
    --arg project_id "$project_id" --arg run_id "$run_id" \
    --arg acquisition_id "$acquisition_id" --arg uid "$live_uid" '
      type == "object"
      and keys == [
        "acceptance_acquisition_id", "api_version", "created_resource_version",
        "kind", "name", "project_id", "resource_type", "run_id", "schema", "uid"
      ]
      and .schema == "axiom.gcp.kubernetes-ownership.v1"
      and .resource_type == $resource_type
      and .api_version == $api_version
      and .kind == $kind
      and .name == $name
      and .project_id == $project_id
      and .run_id == $run_id
      and .acceptance_acquisition_id == $acquisition_id
      and .uid == $uid
      and (.created_resource_version | type) == "string"
      and (.created_resource_version | length) > 0
    ' "$receipt" >/dev/null
}

write_kubernetes_ownership_receipt() {
  local receipt="$1"
  local resource="$2"
  local resource_type="$3"
  local name="$4"
  local project_id="$5"
  local run_id="$6"
  local acquisition_id="$7"
  local uid resource_version temporary
  verify_owned_kubernetes_resource_json \
    "$resource" "$resource_type" "$name" "$project_id" "$run_id" \
    "$acquisition_id" || return 1
  uid="$(jq -er '.metadata.uid | strings | select(length > 0)' \
    <<<"$resource")" || return 1
  resource_version="$(jq -er \
    '.metadata.resourceVersion | strings | select(length > 0)' \
    <<<"$resource")" || return 1
  if [[ -e "$receipt" ]]; then
    verify_kubernetes_ownership_receipt \
      "$receipt" "$resource" "$resource_type" "$name" "$project_id" \
      "$run_id" "$acquisition_id"
    return
  fi
  temporary="$(mktemp "$(dirname "$receipt")/.ownership-receipt.XXXXXX")"
  jq -n \
    --arg resource_type "$resource_type" \
    --arg api_version "$KUBERNETES_OWNERSHIP_API_VERSION" \
    --arg kind "$KUBERNETES_OWNERSHIP_KIND" --arg name "$name" \
    --arg project_id "$project_id" --arg run_id "$run_id" \
    --arg acquisition_id "$acquisition_id" --arg uid "$uid" \
    --arg resource_version "$resource_version" '
      {
        schema:"axiom.gcp.kubernetes-ownership.v1",
        resource_type:$resource_type,
        api_version:$api_version,
        kind:$kind,
        name:$name,
        project_id:$project_id,
        run_id:$run_id,
        acceptance_acquisition_id:$acquisition_id,
        uid:$uid,
        created_resource_version:$resource_version
      }
    ' > "$temporary" || {
      rm -f "$temporary"
      return 1
    }
  chmod 0600 "$temporary"
  mv "$temporary" "$receipt"
}

# Return 0 when the live object is owned, 2 when it is absent, and 1 when its
# identity is unsafe or the API result is unknown.
assert_owned_kubernetes_resource() {
  local resource_type="$1"
  local name="$2"
  local receipt_root="$3"
  local project_id="$4"
  local run_id="$5"
  local acquisition_id="$6"
  local intent receipt resource error_file
  mkdir -p "$receipt_root"
  intent="$(kubernetes_ownership_intent_path \
    "$receipt_root" "$resource_type" "$name")" || return 1
  receipt="$(kubernetes_ownership_receipt_path \
    "$receipt_root" "$resource_type" "$name")" || return 1
  error_file="$(mktemp "$receipt_root/.owned-resource-assert.XXXXXX")"

  if [[ ! -e "$intent" && ! -e "$receipt" ]]; then
    if kubernetes_get_resource_json "$resource_type" "$name" \
        >/dev/null 2> "$error_file"; then
      echo "$resource_type $name exists without this run's ownership intent" >&2
      rm -f "$error_file"
      return 1
    fi
    if kubernetes_api_error_is_not_found "$error_file"; then
      rm -f "$error_file"
      return 2
    fi
    echo "could not inspect unmanaged $resource_type $name:" >&2
    cat "$error_file" >&2
    rm -f "$error_file"
    return 1
  fi
  verify_kubernetes_ownership_intent \
    "$intent" "$resource_type" "$name" "$project_id" "$run_id" \
    "$acquisition_id" || {
    echo "invalid Kubernetes ownership intent for $resource_type $name" >&2
    rm -f "$error_file"
    return 1
  }
  if ! resource="$(kubernetes_get_resource_json "$resource_type" "$name" \
      2> "$error_file")"; then
    if kubernetes_api_error_is_not_found "$error_file"; then
      rm -f "$error_file"
      return 2
    fi
    echo "could not inspect owned $resource_type $name:" >&2
    cat "$error_file" >&2
    rm -f "$error_file"
    return 1
  fi
  rm -f "$error_file"
  verify_owned_kubernetes_resource_json \
    "$resource" "$resource_type" "$name" "$project_id" "$run_id" \
    "$acquisition_id" || {
    echo "$resource_type $name is not owned by this acceptance run" >&2
    return 1
  }
  if [[ ! -e "$receipt" ]]; then
    write_kubernetes_ownership_receipt \
      "$receipt" "$resource" "$resource_type" "$name" "$project_id" \
      "$run_id" "$acquisition_id" || return 1
    echo "recovered the ownership receipt for $resource_type $name" >&2
  fi
  verify_kubernetes_ownership_receipt \
    "$receipt" "$resource" "$resource_type" "$name" "$project_id" \
    "$run_id" "$acquisition_id" || {
    echo "$resource_type $name was replaced; refusing mutation" >&2
    return 1
  }
}

create_owned_kubernetes_resource() {
  local resource_type="$1"
  local name="$2"
  local manifest="$3"
  local receipt_root="$4"
  local project_id="$5"
  local run_id="$6"
  local acquisition_id="$7"
  local intent receipt rendered persisted_manifest resource error_file
  local create_timeout_seconds
  kubernetes_ownership_identity_valid \
    "$project_id" "$run_id" "$acquisition_id" || return 1
  [[ -f "$manifest" && ! -L "$manifest" ]] || return 1
  mkdir -p "$receipt_root"
  intent="$(kubernetes_ownership_intent_path \
    "$receipt_root" "$resource_type" "$name")" || return 1
  receipt="$(kubernetes_ownership_receipt_path \
    "$receipt_root" "$resource_type" "$name")" || return 1
  persisted_manifest="$(kubernetes_ownership_manifest_path \
    "$receipt_root" "$resource_type" "$name")" || return 1
  create_timeout_seconds="${KUBERNETES_OWNERSHIP_CREATE_TIMEOUT_SECONDS:-30}"
  [[ "$create_timeout_seconds" =~ ^[1-9][0-9]*$ \
    && "$create_timeout_seconds" -le 120 ]] || return 1

  if [[ -e "$receipt" ]]; then
    resource="$(kubernetes_get_resource_json "$resource_type" "$name")" \
      || return 1
    verify_kubernetes_ownership_receipt \
      "$receipt" "$resource" "$resource_type" "$name" "$project_id" \
      "$run_id" "$acquisition_id"
    return
  fi

  rendered="$(mktemp "$receipt_root/.owned-resource.XXXXXX")"
  error_file="$(mktemp "$receipt_root/.owned-resource-create.XXXXXX")"
  if ! kubectl label --local --overwrite -f "$manifest" \
      "axiom.axiom.dev/acceptance-owner=$KUBERNETES_ACCEPTANCE_OWNER" \
      "axiom.axiom.dev/acceptance-project=$project_id" \
      "axiom.axiom.dev/acceptance-run-id=$run_id" \
      "axiom.axiom.dev/acceptance-acquisition-id=$acquisition_id" \
      -o json > "$rendered"; then
    rm -f "$rendered" "$error_file"
    return 1
  fi
  if ! verify_owned_kubernetes_resource_json \
      "$(cat "$rendered")" "$resource_type" "$name" "$project_id" \
      "$run_id" "$acquisition_id"; then
    echo "owned manifest does not describe $resource_type $name" >&2
    rm -f "$rendered" "$error_file"
    return 1
  fi
  if [[ -e "$persisted_manifest" ]]; then
    [[ -f "$persisted_manifest" && ! -L "$persisted_manifest" \
      && "$(kubernetes_ownership_file_sha256 "$persisted_manifest")" \
        == "$(kubernetes_ownership_file_sha256 "$rendered")" ]] || {
      echo "persisted owned manifest changed for $resource_type $name" >&2
      rm -f "$rendered" "$error_file"
      return 1
    }
  else
    chmod 0600 "$rendered" || {
      rm -f "$rendered" "$error_file"
      return 1
    }
    mv "$rendered" "$persisted_manifest" || {
      rm -f "$rendered" "$error_file"
      return 1
    }
    rendered="$persisted_manifest"
  fi
  write_kubernetes_ownership_create_intent \
    "$intent" "$resource_type" "$name" "$project_id" "$run_id" \
    "$acquisition_id" "$persisted_manifest" "$create_timeout_seconds" \
    || {
      [[ "$rendered" == "$persisted_manifest" ]] || rm -f "$rendered"
      rm -f "$error_file"
      return 1
    }
  if ! resource="$(kubectl create \
      --request-timeout="${create_timeout_seconds}s" \
      -f "$persisted_manifest" -o json 2> "$error_file")"; then
    if resource="$(kubernetes_get_resource_json "$resource_type" "$name" \
        2>> "$error_file")" \
        && verify_owned_kubernetes_resource_json \
          "$resource" "$resource_type" "$name" "$project_id" "$run_id" \
          "$acquisition_id"; then
      echo "recovered accepted $resource_type $name after an uncertain create response" >&2
    else
      echo "could not create or recover owned $resource_type $name:" >&2
      cat "$error_file" >&2
      [[ "$rendered" == "$persisted_manifest" ]] || rm -f "$rendered"
      rm -f "$error_file"
      return 1
    fi
  fi
  [[ "$rendered" == "$persisted_manifest" ]] || rm -f "$rendered"
  rm -f "$error_file"
  write_kubernetes_ownership_receipt \
    "$receipt" "$resource" "$resource_type" "$name" "$project_id" \
    "$run_id" "$acquisition_id"
}

create_owned_namespace() {
  local name="$1"
  local receipt_root="$2"
  local project_id="$3"
  local run_id="$4"
  local acquisition_id="$5"
  local manifest status
  mkdir -p "$receipt_root"
  manifest="$(mktemp "$receipt_root/.namespace.XXXXXX")"
  jq -n --arg name "$name" \
    '{apiVersion:"v1",kind:"Namespace",metadata:{name:$name}}' \
    > "$manifest" || {
    rm -f "$manifest"
    return 1
  }
  status=0
  create_owned_kubernetes_resource \
    namespace "$name" "$manifest" "$receipt_root" "$project_id" "$run_id" \
    "$acquisition_id" || status=$?
  rm -f "$manifest"
  return "$status"
}

kubernetes_ownership_raw_path() {
  local resource_type="$1"
  local name="$2"
  case "$resource_type" in
    namespace) printf '/api/v1/namespaces/%s\n' "$name" ;;
    customresourcedefinition)
      printf '/apis/apiextensions.k8s.io/v1/customresourcedefinitions/%s\n' "$name"
      ;;
    *) return 1 ;;
  esac
}

write_kubernetes_deletion_receipt() {
  local output="$1"
  local resource_type="$2"
  local name="$3"
  local project_id="$4"
  local run_id="$5"
  local acquisition_id="$6"
  local uid="$7"
  local temporary
  if [[ -e "$output" ]]; then
    jq -e \
      --arg resource_type "$resource_type" --arg name "$name" \
      --arg project_id "$project_id" --arg run_id "$run_id" \
      --arg acquisition_id "$acquisition_id" --arg uid "$uid" '
        .schema == "axiom.gcp.kubernetes-deletion.v1"
        and .resource_type == $resource_type
        and .name == $name
        and .project_id == $project_id
        and .run_id == $run_id
        and .acceptance_acquisition_id == $acquisition_id
        and .uid == $uid
        and .status == "absent"
      ' "$output" >/dev/null
    return
  fi
  temporary="$(mktemp "$(dirname "$output")/.deletion-receipt.XXXXXX")"
  jq -n \
    --arg resource_type "$resource_type" --arg name "$name" \
    --arg project_id "$project_id" --arg run_id "$run_id" \
    --arg acquisition_id "$acquisition_id" --arg uid "$uid" \
    --arg deleted_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" '
      {
        schema:"axiom.gcp.kubernetes-deletion.v1",
        resource_type:$resource_type,
        name:$name,
        project_id:$project_id,
        run_id:$run_id,
        acceptance_acquisition_id:$acquisition_id,
        uid:$uid,
        status:"absent",
        deleted_at:$deleted_at
      }
    ' > "$temporary" || {
      rm -f "$temporary"
      return 1
    }
  chmod 0600 "$temporary"
  mv "$temporary" "$output"
}

delete_owned_kubernetes_resource() {
  local resource_type="$1"
  local name="$2"
  local receipt_root="$3"
  local project_id="$4"
  local run_id="$5"
  local acquisition_id="$6"
  local timeout_seconds="${7:-300}"
  local intent receipt deletion manifest resource error_file uid resource_version raw_path
  local delete_error deletion_timestamp wait_ref delete_status
  local resource_found intent_schema request_deadline_unix now_seconds
  local create_timeout_seconds manifest_sha256
  kubernetes_ownership_identity_valid \
    "$project_id" "$run_id" "$acquisition_id" || return 1
  [[ "$timeout_seconds" =~ ^[1-9][0-9]*$ ]] || return 1
  create_timeout_seconds="${KUBERNETES_OWNERSHIP_CREATE_TIMEOUT_SECONDS:-30}"
  [[ "$create_timeout_seconds" =~ ^[1-9][0-9]*$ \
    && "$create_timeout_seconds" -le 120 ]] || return 1
  mkdir -p "$receipt_root"
  intent="$(kubernetes_ownership_intent_path \
    "$receipt_root" "$resource_type" "$name")" || return 1
  receipt="$(kubernetes_ownership_receipt_path \
    "$receipt_root" "$resource_type" "$name")" || return 1
  manifest="$(kubernetes_ownership_manifest_path \
    "$receipt_root" "$resource_type" "$name")" || return 1
  deletion="$(kubernetes_ownership_deletion_path \
    "$receipt_root" "$resource_type" "$name")" || return 1
  [[ -e "$intent" || -e "$receipt" ]] || return 0
  verify_kubernetes_ownership_intent \
    "$intent" "$resource_type" "$name" "$project_id" "$run_id" \
    "$acquisition_id" || {
    echo "invalid Kubernetes ownership intent for $resource_type $name" >&2
    return 1
  }

  resource_found=0
  error_file="$(mktemp "$receipt_root/.owned-resource-get.XXXXXX")"
  if resource="$(kubernetes_get_resource_json "$resource_type" "$name" \
      2> "$error_file")"; then
    resource_found=1
  elif ! kubernetes_api_error_is_not_found "$error_file"; then
    echo "could not inspect owned $resource_type $name before deletion:" >&2
    cat "$error_file" >&2
    rm -f "$error_file"
    return 1
  fi
  rm -f "$error_file"
  if [[ "$resource_found" != "1" ]]; then
    if [[ -f "$receipt" ]]; then
      uid="$(jq -er '.uid | strings | select(length > 0)' "$receipt")" \
        || return 1
      write_kubernetes_deletion_receipt \
        "$deletion" "$resource_type" "$name" "$project_id" "$run_id" \
        "$acquisition_id" "$uid"
      return
    fi

    # A create whose response was lost must never be treated as absent after a
    # fixed number of GET calls. Every owned create has a server request
    # deadline and a persisted, hashed manifest. Wait beyond that deadline,
    # then retry the exact create. The retry either recovers the original UID
    # or installs a known UID that can be deleted with preconditions.
    intent_schema="$(jq -er '.schema' "$intent")" || return 1
    [[ "$intent_schema" == "axiom.gcp.kubernetes-ownership-intent.v2" \
      && -f "$manifest" && ! -L "$manifest" ]] || {
      echo "uncertain $resource_type $name create has no bounded manifest; retry is unsafe" >&2
      return 1
    }
    manifest_sha256="$(jq -er '.manifest_sha256' "$intent")" || return 1
    [[ "$(kubernetes_ownership_file_sha256 "$manifest")" == "$manifest_sha256" ]] \
      || {
        echo "persisted manifest changed for uncertain $resource_type $name create" >&2
        return 1
      }
    request_deadline_unix="$(jq -er '.request_deadline_unix' "$intent")" \
      || return 1
    now_seconds="$(date +%s)"
    if (( now_seconds <= request_deadline_unix )); then
      sleep $((request_deadline_unix - now_seconds + 1))
    fi
    error_file="$(mktemp "$receipt_root/.owned-resource-recreate.XXXXXX")"
    if resource="$(kubectl create \
        --request-timeout="${create_timeout_seconds}s" \
        -f "$manifest" -o json 2> "$error_file")"; then
      resource_found=1
    elif resource="$(kubernetes_get_resource_json "$resource_type" "$name" \
        2>> "$error_file")" \
        && verify_owned_kubernetes_resource_json \
          "$resource" "$resource_type" "$name" "$project_id" "$run_id" \
          "$acquisition_id"; then
      resource_found=1
      echo "recovered the uncertain $resource_type $name create during cleanup" >&2
    else
      echo "could not fence uncertain $resource_type $name create:" >&2
      cat "$error_file" >&2
      rm -f "$error_file"
      return 1
    fi
    rm -f "$error_file"
  fi
  verify_owned_kubernetes_resource_json \
    "$resource" "$resource_type" "$name" "$project_id" "$run_id" \
    "$acquisition_id" || {
    echo "$resource_type $name is not owned by this acceptance run" >&2
    return 1
  }
  if [[ ! -e "$receipt" ]]; then
    write_kubernetes_ownership_receipt \
      "$receipt" "$resource" "$resource_type" "$name" "$project_id" \
      "$run_id" "$acquisition_id" || return 1
    echo "recovered the ownership receipt for $resource_type $name" >&2
  fi
  verify_kubernetes_ownership_receipt \
    "$receipt" "$resource" "$resource_type" "$name" "$project_id" \
    "$run_id" "$acquisition_id" || {
    echo "$resource_type $name was replaced; refusing deletion" >&2
    return 1
  }
  uid="$(jq -er '.metadata.uid | strings | select(length > 0)' \
    <<<"$resource")" || return 1
  resource_version="$(jq -er \
    '.metadata.resourceVersion | strings | select(length > 0)' \
    <<<"$resource")" || return 1
  raw_path="$(kubernetes_ownership_raw_path "$resource_type" "$name")" \
    || return 1
  kubernetes_ownership_resource_values "$resource_type" || return 1
  wait_ref="$KUBERNETES_OWNERSHIP_CLI_RESOURCE/$name"
  delete_error="$(mktemp "$receipt_root/.owned-resource-delete.XXXXXX")"
  delete_status=0
  jq -n --arg uid "$uid" --arg resource_version "$resource_version" '
    {
      apiVersion:"v1",
      kind:"DeleteOptions",
      propagationPolicy:"Foreground",
      preconditions:{uid:$uid,resourceVersion:$resource_version}
    }
  ' | kubectl delete --raw="$raw_path" -f - \
      >/dev/null 2> "$delete_error" || delete_status=$?
  if [[ "$delete_status" != "0" ]]; then
    error_file="$(mktemp "$receipt_root/.owned-resource-recheck.XXXXXX")"
    if ! resource="$(kubernetes_get_resource_json "$resource_type" "$name" \
        2> "$error_file")"; then
      if kubernetes_api_error_is_not_found "$error_file"; then
        rm -f "$error_file" "$delete_error"
        write_kubernetes_deletion_receipt \
          "$deletion" "$resource_type" "$name" "$project_id" "$run_id" \
          "$acquisition_id" "$uid"
        return
      fi
      cat "$error_file" >&2
      rm -f "$error_file" "$delete_error"
      return 1
    fi
    rm -f "$error_file"
    if [[ "$(jq -r '.metadata.uid // ""' <<<"$resource")" != "$uid" ]]; then
      echo "$resource_type $name was replaced during deletion" >&2
      rm -f "$delete_error"
      return 1
    fi
    deletion_timestamp="$(jq -r '.metadata.deletionTimestamp // ""' \
      <<<"$resource")"
    if [[ -z "$deletion_timestamp" ]]; then
      echo "preconditioned deletion failed for $resource_type $name:" >&2
      cat "$delete_error" >&2
      rm -f "$delete_error"
      return 1
    fi
  fi
  rm -f "$delete_error"
  if ! kubectl wait --for=delete "$wait_ref" \
      --timeout="${timeout_seconds}s" >/dev/null 2>&1; then
    error_file="$(mktemp "$receipt_root/.owned-resource-final.XXXXXX")"
    if kubernetes_get_resource_json "$resource_type" "$name" \
        >/dev/null 2> "$error_file" \
        || ! kubernetes_api_error_is_not_found "$error_file"; then
      echo "$resource_type $name did not finish deletion" >&2
      cat "$error_file" >&2
      rm -f "$error_file"
      return 1
    fi
    rm -f "$error_file"
  fi
  error_file="$(mktemp "$receipt_root/.owned-resource-post-delete.XXXXXX")"
  if resource="$(kubernetes_get_resource_json "$resource_type" "$name" \
      2> "$error_file")"; then
    echo "$resource_type $name was recreated after preconditioned deletion; refusing completion" >&2
    rm -f "$error_file"
    return 1
  fi
  if ! kubernetes_api_error_is_not_found "$error_file"; then
    echo "could not prove that $resource_type $name stayed absent after deletion:" >&2
    cat "$error_file" >&2
    rm -f "$error_file"
    return 1
  fi
  rm -f "$error_file"
  write_kubernetes_deletion_receipt \
    "$deletion" "$resource_type" "$name" "$project_id" "$run_id" \
    "$acquisition_id" "$uid"
}
