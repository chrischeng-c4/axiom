#!/usr/bin/env bash
set -euo pipefail
umask 077

HARNESS_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)"
REPO_ROOT="$(cd "$HARNESS_ROOT/../.." && pwd -P)"
VALIDATOR="$HARNESS_ROOT/scripts/validate.rb"

die() {
  echo "lumen kustomize render: $*" >&2
  return 1
}

validate_patch() {
  local patch_kind="$1" token_mode="$2" patch_file="$3" expected_paths
  jq -e 'type == "array"' "$patch_file" >/dev/null || return 1
  case "$patch_kind" in
    client-namespace)
      expected_paths='["/metadata/name","/metadata/labels/lumen.axiom.dev~1gke-acceptance-run"]'
      ;;
    client-account)
      expected_paths='["/metadata/name","/metadata/namespace","/metadata/labels/lumen.axiom.dev~1gke-acceptance-run"]'
      ;;
    tooling)
      expected_paths='["/metadata/name","/metadata/namespace","/metadata/labels/lumen.axiom.dev~1gke-acceptance-run","/metadata/labels/lumen.axiom.dev~1gke-acceptance-job","/spec/template/metadata/labels/lumen.axiom.dev~1gke-acceptance-run","/spec/template/metadata/labels/lumen.axiom.dev~1gke-acceptance-job","/spec/template/spec/serviceAccountName"]'
      ;;
    api)
      expected_paths='["/metadata/name","/metadata/namespace","/metadata/labels/lumen.axiom.dev~1gke-acceptance-run","/metadata/labels/lumen.axiom.dev~1gke-acceptance-job","/spec/template/metadata/labels/lumen.axiom.dev~1gke-acceptance-run","/spec/template/metadata/labels/lumen.axiom.dev~1gke-acceptance-job","/spec/template/spec/serviceAccountName","/spec/template/spec/automountServiceAccountToken","/spec/template/spec/containers/0/env/0/value","/spec/template/spec/containers/0/env/1/value","/spec/template/spec/containers/0/env/2/value","/spec/template/spec/containers/0/env/3/value","/spec/template/spec/containers/0/env/4/value","/spec/template/spec/containers/0/env/5/value","/spec/template/spec/containers/0/env/6/value","/spec/template/spec/containers/0/env/7/value","/spec/template/spec/containers/0/env/8/value","/spec/template/spec/containers/0/env/9/value"]'
      if [[ "$token_mode" == projected ]]; then
        expected_paths="$(jq -c '. + ["/spec/template/spec/volumes/-","/spec/template/spec/containers/0/volumeMounts/-"]' <<<"$expected_paths")"
      fi
      ;;
    metrics)
      expected_paths='["/metadata/name","/metadata/namespace","/metadata/labels/lumen.axiom.dev~1gke-acceptance-run","/metadata/labels/lumen.axiom.dev~1gke-acceptance-job","/spec/template/metadata/labels/lumen.axiom.dev~1gke-acceptance-run","/spec/template/metadata/labels/lumen.axiom.dev~1gke-acceptance-job","/spec/template/spec/containers/0/env/0/value","/spec/template/spec/containers/0/env/1/value","/spec/template/spec/containers/0/env/2/value"]'
      ;;
    *) return 1 ;;
  esac

  jq -e --argjson paths "$expected_paths" --arg mode "$token_mode" '
    ([.[].path] == $paths) and
    (([.[].path] | length) == ([.[].path] | unique | length)) and
    (all(.[]; (keys | sort) == ["op","path","value"])) and
    (all(.[]; .op == "replace" or
      ($mode == "projected" and .op == "add" and
       (.path == "/spec/template/spec/volumes/-" or
        .path == "/spec/template/spec/containers/0/volumeMounts/-")))) and
    (if $mode == "projected" then
      .[-2] == {
        op:"add",
        path:"/spec/template/spec/volumes/-",
        value:{name:"projected",projected:{sources:[{serviceAccountToken:{path:"token",audience:"lumen.axiom.dev",expirationSeconds:600}}]}}
      } and
      .[-1] == {
        op:"add",
        path:"/spec/template/spec/containers/0/volumeMounts/-",
        value:{name:"projected",mountPath:"/run/lumen/projected",readOnly:true}
      }
    else
      all(.[]; .op == "replace")
    end)
  ' "$patch_file" >/dev/null
}

valid_dns_label() {
  [[ "$1" =~ ^[a-z0-9]([-a-z0-9]{0,61}[a-z0-9])?$ ]] && ((${#1} <= 63))
}

valid_row_label() {
  [[ "$1" =~ ^[a-z0-9]([-a-z0-9]{0,38}[a-z0-9])?$ ]] && ((${#1} <= 40))
}

mark_seen() {
  case " $seen " in
    *" $1 "*) die "duplicate --$1" ;;
  esac
  seen="$seen $1"
}

need_value() {
  (($# >= 2)) && [[ -n "$2" ]] || die "$1 requires a value"
}

require_flags() {
  local flag
  for flag in "$@"; do
    case " $seen " in
      *" $flag "*) ;;
      *) die "missing --$flag" ;;
    esac
  done
}

allow_only_flags() {
  local allowed=" $* " flag
  for flag in $seen; do
    case "$allowed" in
      *" $flag "*) ;;
      *) die "--$flag is not valid for $component" ;;
    esac
  done
}

prepare_output() {
  [[ "$out_dir" == /* ]] || die "--out-dir must be absolute"
  [[ ! -e "$out_dir" && ! -L "$out_dir" ]] || die "--out-dir must not already exist"
  local parent leaf parent_real output_real
  parent="$(dirname "$out_dir")"
  leaf="$(basename "$out_dir")"
  [[ "$leaf" != . && "$leaf" != .. && "$leaf" != */* ]] || die "invalid --out-dir"
  [[ -d "$parent" ]] || die "--out-dir parent must exist"
  parent_real="$(cd "$parent" && pwd -P)"
  output_real="$parent_real/$leaf"
  case "$output_real/" in
    "$REPO_ROOT/"*) die "--out-dir must be outside the repository" ;;
  esac
  mkdir "$output_real"
  out_dir="$output_real"
  mkdir "$out_dir/base"
}

write_kustomization() {
  local patch_name target
  {
    echo 'apiVersion: kustomize.config.k8s.io/v1beta1'
    echo 'kind: Kustomization'
    echo 'resources:'
    echo '  - base'
    echo 'patches:'
    while (($#)); do
      patch_name="$1"
      target="$2"
      shift 2
      echo "  - path: $patch_name"
      echo '    target:'
      printf '%s\n' "$target"
    done
  } >"$out_dir/kustomization.yaml"
}

render_client() {
  cp "$HARNESS_ROOT/client/kustomization.yaml" "$HARNESS_ROOT/client/namespace.yaml" "$HARNESS_ROOT/client/serviceaccounts.yaml" "$out_dir/base/"
  jq -n --arg namespace "$client_namespace" --arg run "$run_id" '[
    {op:"replace",path:"/metadata/name",value:$namespace},
    {op:"replace",path:"/metadata/labels/lumen.axiom.dev~1gke-acceptance-run",value:$run}
  ]' >"$out_dir/namespace.patch.json"
  jq -n --arg namespace "$client_namespace" --arg run "$run_id" '[
    {op:"replace",path:"/metadata/name",value:"app"},
    {op:"replace",path:"/metadata/namespace",value:$namespace},
    {op:"replace",path:"/metadata/labels/lumen.axiom.dev~1gke-acceptance-run",value:$run}
  ]' >"$out_dir/app.patch.json"
  jq -n --arg namespace "$client_namespace" --arg run "$run_id" '[
    {op:"replace",path:"/metadata/name",value:"unlisted"},
    {op:"replace",path:"/metadata/namespace",value:$namespace},
    {op:"replace",path:"/metadata/labels/lumen.axiom.dev~1gke-acceptance-run",value:$run}
  ]' >"$out_dir/unlisted.patch.json"
  validate_patch client-namespace none "$out_dir/namespace.patch.json"
  validate_patch client-account none "$out_dir/app.patch.json"
  validate_patch client-account none "$out_dir/unlisted.patch.json"
  write_kustomization \
    namespace.patch.json $'      version: v1\n      kind: Namespace\n      name: INVALID_CLIENT_NAMESPACE' \
    app.patch.json $'      version: v1\n      kind: ServiceAccount\n      name: INVALID_APP_KSA\n      namespace: INVALID_CLIENT_NAMESPACE' \
    unlisted.patch.json $'      version: v1\n      kind: ServiceAccount\n      name: INVALID_UNLISTED_KSA\n      namespace: INVALID_CLIENT_NAMESPACE'
}

render_tooling() {
  cp "$HARNESS_ROOT/jobs/tooling/kustomization.yaml" "$HARNESS_ROOT/jobs/tooling/job.yaml" "$out_dir/base/"
  jq -n --arg namespace "$client_namespace" --arg run "$run_id" --arg job "$job" '[
    {op:"replace",path:"/metadata/name",value:$job},
    {op:"replace",path:"/metadata/namespace",value:$namespace},
    {op:"replace",path:"/metadata/labels/lumen.axiom.dev~1gke-acceptance-run",value:$run},
    {op:"replace",path:"/metadata/labels/lumen.axiom.dev~1gke-acceptance-job",value:$job},
    {op:"replace",path:"/spec/template/metadata/labels/lumen.axiom.dev~1gke-acceptance-run",value:$run},
    {op:"replace",path:"/spec/template/metadata/labels/lumen.axiom.dev~1gke-acceptance-job",value:$job},
    {op:"replace",path:"/spec/template/spec/serviceAccountName",value:"default"}
  ]' >"$out_dir/tooling.patch.json"
  validate_patch tooling none "$out_dir/tooling.patch.json"
  write_kustomization tooling.patch.json $'      group: batch\n      version: v1\n      kind: Job\n      name: INVALID_TOOLING_JOB\n      namespace: INVALID_CLIENT_NAMESPACE'
}

render_api() {
  local request_b64 required_value rejected_value automount
  cp "$HARNESS_ROOT/jobs/api/kustomization.yaml" "$HARNESS_ROOT/jobs/api/job.yaml" "$out_dir/base/"
  request_b64="$(base64 <"$request_file" | tr -d '\r\n')"
  required_value="$required_id"
  rejected_value="$rejected_id"
  [[ "$required_value" == none ]] && required_value=''
  [[ "$rejected_value" == none ]] && rejected_value=''
  automount=false
  [[ "$token_mode" == default ]] && automount=true
  jq -n \
    --arg namespace "$client_namespace" --arg run "$run_id" --arg job "$job" \
    --arg account "$account" --arg mode "$token_mode" --arg runtime "$runtime_namespace" \
    --arg service "$service" --arg method "$method" --arg path_value "$request_path" \
    --arg body "$request_b64" --arg expected "$expected_status" --arg required "$required_value" \
    --arg rejected "$rejected_value" --arg row "$row_label" --argjson automount "$automount" '[
      {op:"replace",path:"/metadata/name",value:$job},
      {op:"replace",path:"/metadata/namespace",value:$namespace},
      {op:"replace",path:"/metadata/labels/lumen.axiom.dev~1gke-acceptance-run",value:$run},
      {op:"replace",path:"/metadata/labels/lumen.axiom.dev~1gke-acceptance-job",value:$job},
      {op:"replace",path:"/spec/template/metadata/labels/lumen.axiom.dev~1gke-acceptance-run",value:$run},
      {op:"replace",path:"/spec/template/metadata/labels/lumen.axiom.dev~1gke-acceptance-job",value:$job},
      {op:"replace",path:"/spec/template/spec/serviceAccountName",value:$account},
      {op:"replace",path:"/spec/template/spec/automountServiceAccountToken",value:$automount},
      {op:"replace",path:"/spec/template/spec/containers/0/env/0/value",value:$mode},
      {op:"replace",path:"/spec/template/spec/containers/0/env/1/value",value:$runtime},
      {op:"replace",path:"/spec/template/spec/containers/0/env/2/value",value:$service},
      {op:"replace",path:"/spec/template/spec/containers/0/env/3/value",value:$method},
      {op:"replace",path:"/spec/template/spec/containers/0/env/4/value",value:$path_value},
      {op:"replace",path:"/spec/template/spec/containers/0/env/5/value",value:$body},
      {op:"replace",path:"/spec/template/spec/containers/0/env/6/value",value:$expected},
      {op:"replace",path:"/spec/template/spec/containers/0/env/7/value",value:$required},
      {op:"replace",path:"/spec/template/spec/containers/0/env/8/value",value:$rejected},
      {op:"replace",path:"/spec/template/spec/containers/0/env/9/value",value:$row}
    ]' >"$out_dir/api.patch.json"
  if [[ "$token_mode" == projected ]]; then
    jq '. + [
      {op:"add",path:"/spec/template/spec/volumes/-",value:{name:"projected",projected:{sources:[{serviceAccountToken:{path:"token",audience:"lumen.axiom.dev",expirationSeconds:600}}]}}},
      {op:"add",path:"/spec/template/spec/containers/0/volumeMounts/-",value:{name:"projected",mountPath:"/run/lumen/projected",readOnly:true}}
    ]' "$out_dir/api.patch.json" >"$out_dir/api.patch.tmp"
    mv "$out_dir/api.patch.tmp" "$out_dir/api.patch.json"
  fi
  validate_patch api "$token_mode" "$out_dir/api.patch.json"
  write_kustomization api.patch.json $'      group: batch\n      version: v1\n      kind: Job\n      name: INVALID_API_JOB\n      namespace: INVALID_CLIENT_NAMESPACE'
}

render_metrics() {
  cp "$HARNESS_ROOT/jobs/metrics/kustomization.yaml" "$HARNESS_ROOT/jobs/metrics/job.yaml" "$out_dir/base/"
  jq -n --arg namespace "$client_namespace" --arg run "$run_id" --arg job "$job" \
    --arg runtime "$runtime_namespace" --arg service "$service" --arg row "$row_label" '[
      {op:"replace",path:"/metadata/name",value:$job},
      {op:"replace",path:"/metadata/namespace",value:$namespace},
      {op:"replace",path:"/metadata/labels/lumen.axiom.dev~1gke-acceptance-run",value:$run},
      {op:"replace",path:"/metadata/labels/lumen.axiom.dev~1gke-acceptance-job",value:$job},
      {op:"replace",path:"/spec/template/metadata/labels/lumen.axiom.dev~1gke-acceptance-run",value:$run},
      {op:"replace",path:"/spec/template/metadata/labels/lumen.axiom.dev~1gke-acceptance-job",value:$job},
      {op:"replace",path:"/spec/template/spec/containers/0/env/0/value",value:$runtime},
      {op:"replace",path:"/spec/template/spec/containers/0/env/1/value",value:$service},
      {op:"replace",path:"/spec/template/spec/containers/0/env/2/value",value:$row}
    ]' >"$out_dir/metrics.patch.json"
  validate_patch metrics none "$out_dir/metrics.patch.json"
  write_kustomization metrics.patch.json $'      group: batch\n      version: v1\n      kind: Job\n      name: INVALID_METRICS_JOB\n      namespace: INVALID_CLIENT_NAMESPACE'
}

validate_render() {
  local args
  args=("$component" --file "$out_dir/rendered.yaml" --client-namespace "$client_namespace" --run-id "$run_id")
  case "$component" in
    tooling)
      args+=(--job "$job")
      ;;
    api)
      args+=(--job "$job" --account "$account" --token-mode "$token_mode" \
        --runtime-namespace "$runtime_namespace" --service "$service" --method "$method" \
        --path "$request_path" --request-file "$request_file" --expected-status "$expected_status" \
        --required-id "$required_id" --rejected-id "$rejected_id" --row-label "$row_label")
      ;;
    metrics)
      args+=(--job "$job" --runtime-namespace "$runtime_namespace" --service "$service" --row-label "$row_label")
      ;;
  esac
  ruby "$VALIDATOR" "${args[@]}"
}

main() {
  component="${1:-}"
  [[ -n "$component" ]] || die "component is required"
  shift || true
  case "$component" in client|tooling|api|metrics) ;; *) die "unknown component: $component" ;; esac

  seen=''
  out_dir=''
  client_namespace=''
  runtime_namespace=''
  service=''
  run_id=''
  job=''
  account=''
  token_mode=''
  method=''
  request_path=''
  request_file=''
  expected_status=''
  required_id=''
  rejected_id=''
  row_label=''

  while (($#)); do
    need_value "$@"
    case "$1" in
      --out-dir) mark_seen out-dir; out_dir="$2" ;;
      --client-namespace) mark_seen client-namespace; client_namespace="$2" ;;
      --runtime-namespace) mark_seen runtime-namespace; runtime_namespace="$2" ;;
      --service) mark_seen service; service="$2" ;;
      --run-id) mark_seen run-id; run_id="$2" ;;
      --job) mark_seen job; job="$2" ;;
      --account) mark_seen account; account="$2" ;;
      --token-mode) mark_seen token-mode; token_mode="$2" ;;
      --method) mark_seen method; method="$2" ;;
      --path) mark_seen path; request_path="$2" ;;
      --request-file) mark_seen request-file; request_file="$2" ;;
      --expected-status) mark_seen expected-status; expected_status="$2" ;;
      --required-id) mark_seen required-id; required_id="$2" ;;
      --rejected-id) mark_seen rejected-id; rejected_id="$2" ;;
      --row-label) mark_seen row-label; row_label="$2" ;;
      *) die "unknown option: $1" ;;
    esac
    shift 2
  done

  case "$component" in
    client)
      require_flags out-dir client-namespace run-id
      allow_only_flags out-dir client-namespace run-id
      ;;
    tooling)
      require_flags out-dir client-namespace run-id job
      allow_only_flags out-dir client-namespace run-id job
      ;;
    api)
      require_flags out-dir client-namespace runtime-namespace service run-id job account token-mode method path request-file expected-status required-id rejected-id row-label
      allow_only_flags out-dir client-namespace runtime-namespace service run-id job account token-mode method path request-file expected-status required-id rejected-id row-label
      ;;
    metrics)
      require_flags out-dir client-namespace runtime-namespace service run-id job row-label
      allow_only_flags out-dir client-namespace runtime-namespace service run-id job row-label
      ;;
  esac

  valid_dns_label "$client_namespace" || die "invalid client namespace"
  valid_dns_label "$run_id" || die "invalid run id"
  if [[ "$component" != client ]]; then valid_dns_label "$job" || die "invalid job name"; fi
  if [[ "$component" == api || "$component" == metrics ]]; then
    valid_dns_label "$runtime_namespace" || die "invalid runtime namespace"
    valid_dns_label "$service" || die "invalid service name"
    valid_row_label "$row_label" || die "invalid row label"
  fi
  if [[ "$component" == api ]]; then
    case "$method" in GET|PUT|POST) ;; *) die "invalid method" ;; esac
    [[ "$request_path" == /* && "$request_path" != *$'\n'* && ${#request_path} -le 512 ]] || die "invalid request path"
    [[ -f "$request_file" && ! -L "$request_file" && -r "$request_file" ]] || die "invalid request file"
    (($(wc -c <"$request_file") <= 1048576)) || die "request file is too large"
    case "$expected_status" in 2xx|401|403) ;; *) die "invalid expected status" ;; esac
    [[ "$required_id" == none || "$required_id" =~ ^[a-z0-9]([-a-z0-9]{0,126}[a-z0-9])?$ ]] || die "invalid required id"
    [[ "$rejected_id" == none || "$rejected_id" =~ ^[a-z0-9]([-a-z0-9]{0,126}[a-z0-9])?$ ]] || die "invalid rejected id"
    case "$token_mode:$account" in
      default:app|default:unlisted|projected:app|projected:unlisted|missing:default|bad:unlisted) ;;
      *) die "invalid token/account row" ;;
    esac
  fi

  command -v jq >/dev/null || die "jq is required"
  command -v kubectl >/dev/null || die "kubectl is required"
  command -v ruby >/dev/null || die "ruby is required"
  [[ -x "$VALIDATOR" ]] || die "validator is not executable"
  prepare_output

  case "$component" in
    client) render_client ;;
    tooling) render_tooling ;;
    api) render_api ;;
    metrics) render_metrics ;;
  esac

  env -u KUBERNETES_MASTER KUBECONFIG=/dev/null \
    kubectl kustomize "$out_dir" --load-restrictor=LoadRestrictionsRootOnly >"$out_dir/rendered.yaml"
  if grep -q 'INVALID_' "$out_dir/rendered.yaml"; then
    die "unresolved sentinel in rendered resources"
  fi
  validate_render
}

if [[ "${BASH_SOURCE[0]}" == "$0" ]]; then
  main "$@"
fi
