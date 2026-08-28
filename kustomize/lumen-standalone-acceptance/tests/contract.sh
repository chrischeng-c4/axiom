#!/usr/bin/env bash
# shellcheck disable=SC1090,SC2312
set -euo pipefail
umask 077

root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)
renderer="$root/scripts/render.sh"
validator="$root/scripts/validate.rb"
fixture_source="$root/tests/fixtures/request.json"
tmp="$(mktemp -d "${TMPDIR:-/tmp}/lumen-kustomize-contract.XXXXXX")"
fixture="$tmp/request.json"
owned_files=()
owned_dirs=("$tmp")
cleanup() {
  local status=$? cleanup_status=0 file index dir
  trap - EXIT
  for file in "${owned_files[@]-}"; do
    if [[ -e "$file" || -L "$file" ]]; then
      rm -f -- "$file" || cleanup_status=1
    fi
  done
  for ((index=${#owned_dirs[@]}-1; index>=0; index--)); do
    dir=${owned_dirs[$index]}
    if [[ -d "$dir" ]]; then rmdir -- "$dir" 2>/dev/null || cleanup_status=1; fi
  done
  if [[ "$status" -eq 0 && "$cleanup_status" -ne 0 ]]; then
    echo 'kustomize contract: task-local cleanup was incomplete' >&2
    status=1
  fi
  exit "$status"
}
trap cleanup EXIT

command -v jq >/dev/null
command -v ruby >/dev/null
command -v kubectl >/dev/null
[[ -x "$renderer" && -x "$validator" ]] || { echo 'acceptance tools are not executable' >&2; exit 1; }
[[ -f "$fixture_source" && ! -L "$fixture_source" ]] || { echo "missing checked-in fixture: $fixture_source" >&2; exit 1; }

sha256() {
  if command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$1" | awk '{print $1}'
  else
    sha256sum "$1" | awk '{print $1}'
  fi
}
base_files=(
  "$root/client/kustomization.yaml" "$root/client/namespace.yaml" "$root/client/serviceaccounts.yaml"
  "$root/jobs/tooling/kustomization.yaml" "$root/jobs/tooling/job.yaml"
  "$root/jobs/api/kustomization.yaml" "$root/jobs/api/job.yaml"
  "$root/jobs/metrics/kustomization.yaml" "$root/jobs/metrics/job.yaml"
)
before=()
for file in "${base_files[@]}"; do before+=("$(sha256 "$file")"); done
fixture_source_before="$(sha256 "$fixture_source")"
[[ "$fixture_source_before" == 3c5a852ddd7b3f0b1e718435b22a03fdb4d6b71395665a3697beebb8e849b7a9 ]] || {
  echo 'checked-in request fixture bytes changed' >&2
  exit 1
}
cp "$fixture_source" "$fixture"
printf '\n' >>"$fixture"
owned_files+=("$fixture")
[[ "$(sha256 "$fixture")" == f430728441dec341edc81c69dbd199da7795a994695ddc194e34dd80ece94ea0 ]] || {
  echo 'runtime request fixture does not end in two exact LF bytes' >&2
  exit 1
}

fail() { echo "kustomize contract: $*" >&2; exit 1; }
expect_reject() {
  local description=$1
  shift
  if "$@" >/dev/null 2>&1; then fail "accepted forbidden mutation: $description"; fi
}
changed() {
  if cmp -s "$1" "$2"; then fail "mutation did not change bytes: $3"; fi
}
decode_base64() {
  if base64 --help 2>&1 | grep -q -- '--decode'; then base64 --decode; else base64 -D; fi
}

register_render_paths() {
  local out=$1 component=$2
  owned_dirs+=("$out" "$out/base")
  owned_files+=("$out/base/kustomization.yaml" "$out/kustomization.yaml" "$out/rendered.yaml")
  case "$component" in
    client) owned_files+=("$out/base/namespace.yaml" "$out/base/serviceaccounts.yaml" "$out/namespace.patch.json" "$out/app.patch.json" "$out/unlisted.patch.json") ;;
    tooling) owned_files+=("$out/base/kustomization.yaml" "$out/base/job.yaml" "$out/tooling.patch.json") ;;
    api) owned_files+=("$out/base/kustomization.yaml" "$out/base/job.yaml" "$out/api.patch.json" "$out/api.patch.tmp") ;;
    metrics) owned_files+=("$out/base/kustomization.yaml" "$out/base/job.yaml" "$out/metrics.patch.json") ;;
  esac
}
run_render() {
  local component=$1 out=$2 mode=${3:-} account=${4:-} job=$5 row=$6
  local args=(--out-dir "$out" --client-namespace contract-client --run-id contract-run)
  case "$component" in
    client) ;;
    tooling) args+=(--job "$job") ;;
    metrics) args+=(--job "$job" --runtime-namespace lumen --service lumen --row-label "$row") ;;
    api) args+=(--job "$job" --account "$account" --token-mode "$mode" --runtime-namespace lumen --service lumen --method POST --path /collections/demo/docs --request-file "$fixture" --expected-status 2xx --required-id fixture-id --rejected-id none --row-label "$row") ;;
  esac
  register_render_paths "$out" "$component"
  "$renderer" "$component" "${args[@]}"
}

for file in "${base_files[@]}"; do [[ -f "$file" ]] || fail "missing base $file"; done
for base in client jobs/tooling jobs/api jobs/metrics; do
  base_output="$tmp/base-${base##*/}.yaml"
  owned_files+=("$base_output")
  kubectl kustomize "$root/$base" --load-restrictor=LoadRestrictionsRootOnly >"$base_output"
  grep -q 'INVALID_' "$base_output" || fail "base lost sentinel: $base"
done

client="$tmp/client"; tooling="$tmp/tooling"; metrics="$tmp/metrics"
run_render client "$client" '' '' client client
run_render tooling "$tooling" '' '' tooling client-tools
run_render metrics "$metrics" '' '' metrics metrics

rows=("default app" "default unlisted" "projected app" "projected unlisted" "missing default" "bad unlisted")
api_files=()
for pair in "${rows[@]}"; do
  read -r mode account <<<"$pair"
  row="$mode-$account"
  out="$tmp/api-$row"
  run_render api "$out" "$mode" "$account" "api-$row" "$row"
  api_files+=("$out/rendered.yaml")
  ruby "$validator" api --file "$out/rendered.yaml" --client-namespace contract-client --run-id contract-run --job "api-$row" --account "$account" --token-mode "$mode" --runtime-namespace lumen --service lumen --method POST --path /collections/demo/docs --request-file "$fixture" --expected-status 2xx --required-id fixture-id --rejected-id none --row-label "$row" --emit-json >"$tmp/$row.json"
  owned_files+=("$tmp/$row.json")
  body=$(jq -r '.[] | select(.kind=="Job") | .spec.template.spec.containers[0].env[] | select(.name=="LUMEN_REQUEST_BODY_B64") | .value' "$tmp/$row.json")
  printf '%s' "$body" | decode_base64 >"$tmp/$row.body"
  owned_files+=("$tmp/$row.body")
  cmp -s "$fixture" "$tmp/$row.body" || fail "request fixture bytes changed for $row"
done

for pair in "${rows[@]}"; do
  read -r mode account <<<"$pair"
  row="$mode-$account"
  bundle="$tmp/bundle-$row.yaml"
  owned_files+=("$bundle")
  { cat "$client/rendered.yaml"; printf '%s\n' '---'; cat "$tooling/rendered.yaml"; printf '%s\n' '---'; cat "$tmp/api-$row/rendered.yaml"; printf '%s\n' '---'; cat "$metrics/rendered.yaml"; } >"$bundle"
  ruby "$validator" bundle --file "$bundle" --client-namespace contract-client --run-id contract-run --tooling-job tooling --api-job "api-$row" --metrics-job metrics --metrics-row-label metrics --account "$account" --token-mode "$mode" --runtime-namespace lumen --service lumen --method POST --path /collections/demo/docs --request-file "$fixture" --expected-status 2xx --required-id fixture-id --rejected-id none --row-label "$row"
done

mutate_bundle_and_reject() {
  local source=$1 name=$2 filter=$3 target="$tmp/bundle-mutation-$2.yaml"
  jq "$filter" "$source" >"$target"
  owned_files+=("$target")
  changed "$source" "$target" "$name"
  expect_reject "$name" ruby "$validator" bundle --file "$target" --client-namespace contract-client --run-id contract-run --tooling-job tooling --api-job api-default-app --metrics-job metrics --metrics-row-label metrics --account app --token-mode default --runtime-namespace lumen --service lumen --method POST --path /collections/demo/docs --request-file "$fixture" --expected-status 2xx --required-id fixture-id --rejected-id none --row-label default-app
}
valid_bundle="$tmp/bundle-default-app.yaml"
valid_bundle_json="$tmp/bundle-default-app.json"
owned_files+=("$valid_bundle_json")
ruby "$validator" bundle --file "$valid_bundle" --client-namespace contract-client --run-id contract-run --tooling-job tooling --api-job api-default-app --metrics-job metrics --metrics-row-label metrics --account app --token-mode default --runtime-namespace lumen --service lumen --method POST --path /collections/demo/docs --request-file "$fixture" --expected-status 2xx --required-id fixture-id --rejected-id none --row-label default-app --emit-json >"$valid_bundle_json"
mutate_bundle_and_reject "$valid_bundle_json" privileged 'map(if .kind == "Job" and .metadata.name == "tooling" then .spec.template.spec.securityContext.runAsNonRoot = false else . end)'
mutate_bundle_and_reject "$valid_bundle_json" candidate-image 'map(if .kind == "Job" and .metadata.name == "api-default-app" then .spec.template.spec.containers[0].image = "curl:candidate" else . end)'
mutate_bundle_and_reject "$valid_bundle_json" wrong-name-sa 'map(if .kind == "Job" and .metadata.name == "api-default-app" then .metadata.name = "wrong" | .spec.template.spec.serviceAccountName = "wrong" else . end)'
mutate_bundle_and_reject "$valid_bundle_json" api-env 'map(if .kind == "Job" and .metadata.name == "api-default-app" then (.spec.template.spec.containers[0].env[] | select(.name == "LUMEN_PATH").value) = "/wrong" else . end)'
mutate_bundle_and_reject "$valid_bundle_json" metrics-env 'map(if .kind == "Job" and .metadata.name == "metrics" then (.spec.template.spec.containers[0].env[] | select(.name == "LUMEN_SERVICE").value) = "wrong" else . end)'
mutate_bundle_and_reject "$valid_bundle_json" service-account-field 'map(if .kind == "ServiceAccount" and .metadata.name == "app" then .automountServiceAccountToken = false else . end)'

mutate_and_reject() {
  local source=$1 name=$2 filter=$3 mode=$4 account=$5 job=$6 row=$7 target="$tmp/mutation-$2.json"
  jq "$filter" "$source" >"$target"
  owned_files+=("$target")
  changed "$source" "$target" "$name"
  expect_reject "$name" ruby "$validator" api --file "$target" --client-namespace contract-client --run-id contract-run --job "$job" --account "$account" --token-mode "$mode" --runtime-namespace lumen --service lumen --method POST --path /collections/demo/docs --request-file "$fixture" --expected-status 2xx --required-id fixture-id --rejected-id none --row-label "$row"
}
mutate_row_and_reject() {
  local source=$1 name=$2 filter=$3 mode=$4 account=$5 job=$6 row=$7 target="$tmp/mutation-$2.json"
  jq "$filter" "$source" >"$target"
  owned_files+=("$target")
  changed "$source" "$target" "$name"
  expect_reject "$name" ruby "$validator" api --file "$target" --client-namespace contract-client --run-id contract-run --job "$job" --account "$account" --token-mode "$mode" --runtime-namespace lumen --service lumen --method POST --path /collections/demo/docs --request-file "$fixture" --expected-status 2xx --required-id fixture-id --rejected-id none --row-label "$row"
}
api_default="$tmp/default-app.json"
mutate_and_reject "$api_default" image '.[] |= if .kind=="Job" then .spec.template.spec.containers[0].image = "curl:latest" | . else . end' default app api-default-app default-app
mutate_and_reject "$api_default" candidate-image '.[] |= if .kind=="Job" then .spec.template.spec.containers[0].image = "docker.io/curlimages/curl:candidate" | . else . end' default app api-default-app default-app
mutate_and_reject "$api_default" uid '.[] |= if .kind=="Job" then .spec.template.spec.securityContext.runAsUser = 0 | . else . end' default app api-default-app default-app
mutate_and_reject "$api_default" gid '.[] |= if .kind=="Job" then .spec.template.spec.securityContext.runAsGroup = 0 | . else . end' default app api-default-app default-app
mutate_and_reject "$api_default" nonroot '.[] |= if .kind=="Job" then .spec.template.spec.securityContext.runAsNonRoot = false | . else . end' default app api-default-app default-app
mutate_and_reject "$api_default" seccomp '.[] |= if .kind=="Job" then .spec.template.spec.securityContext.seccompProfile.type = "Unconfined" | . else . end' default app api-default-app default-app
mutate_and_reject "$api_default" command '.[] |= if .kind=="Job" then .spec.template.spec.containers[0].command = ["sh"] | . else . end' default app api-default-app default-app
mutate_and_reject "$api_default" privilege '.[] |= if .kind=="Job" then .spec.template.spec.containers[0].securityContext.allowPrivilegeEscalation = true | . else . end' default app api-default-app default-app
mutate_and_reject "$api_default" rootfs '.[] |= if .kind=="Job" then .spec.template.spec.containers[0].securityContext.readOnlyRootFilesystem = false | . else . end' default app api-default-app default-app
mutate_and_reject "$api_default" capabilities '.[] |= if .kind=="Job" then .spec.template.spec.containers[0].securityContext.capabilities.drop = [] | . else . end' default app api-default-app default-app
mutate_and_reject "$api_default" second-container '.[] |= if .kind=="Job" then .spec.template.spec.containers += [{name:"extra",image:"bad"}] | . else . end' default app api-default-app default-app
mutate_and_reject "$api_default" forbidden-kind '.[] |= if .kind=="Job" then .kind = "Deployment" | . else . end' default app api-default-app default-app

for pair in "default app" "default unlisted" "projected app" "projected unlisted" "missing default" "bad unlisted"; do
  read -r mode account <<<"$pair"
  row="$mode-$account"
  source_file="$tmp/$row.json"
  mutate_row_and_reject "$source_file" "$row-wrong-ksa" '.[] |= if .kind=="Job" then .spec.template.spec.serviceAccountName = "wrong" else . end' "$mode" "$account" "api-$row" "$row"
  automount=false
  [[ "$mode" == default ]] && automount=true
  toggle=true
  [[ "$automount" == true ]] && toggle=false
  mutate_row_and_reject "$source_file" "$row-wrong-automount" ".[] |= if .kind==\"Job\" then .spec.template.spec.automountServiceAccountToken = $toggle else . end" "$mode" "$account" "api-$row" "$row"
done
for pair in "default app" "default unlisted" "missing default" "bad unlisted"; do
  read -r mode account <<<"$pair"
  row="$mode-$account"
  source_file="$tmp/$row.json"
  mutate_row_and_reject "$source_file" "$row-extra-projected" '.[] |= if .kind=="Job" then .spec.template.spec.volumes += [{name:"projected",projected:{sources:[]}}] | .spec.template.spec.containers[0].volumeMounts += [{name:"projected",mountPath:"/run/lumen/projected",readOnly:true}] | . else . end' "$mode" "$account" "api-$row" "$row"
done

projected="$tmp/projected-app.json"
ruby "$validator" api --file "$tmp/api-projected-app/rendered.yaml" --client-namespace contract-client --run-id contract-run --job api-projected-app --account app --token-mode projected --runtime-namespace lumen --service lumen --method POST --path /collections/demo/docs --request-file "$fixture" --expected-status 2xx --required-id fixture-id --rejected-id none --row-label projected-app --emit-json >"$projected"
owned_files+=("$projected")
for mutation in audience expiration path name mountPath readOnly; do
  case "$mutation" in
    audience) filter='.[] |= if .kind=="Job" then (.spec.template.spec.volumes[] | select(.name=="projected") | .projected.sources[0].serviceAccountToken.audience) = "wrong" else . end' ;;
    expiration) filter='.[] |= if .kind=="Job" then (.spec.template.spec.volumes[] | select(.name=="projected") | .projected.sources[0].serviceAccountToken.expirationSeconds) = 601 else . end' ;;
    path) filter='.[] |= if .kind=="Job" then (.spec.template.spec.volumes[] | select(.name=="projected") | .projected.sources[0].serviceAccountToken.path) = "other" else . end' ;;
    name) filter='.[] |= if .kind=="Job" then (.spec.template.spec.volumes[] | select(.name=="projected")).name = "other" else . end' ;;
    mountPath) filter='.[] |= if .kind=="Job" then (.spec.template.spec.containers[0].volumeMounts[] | select(.name=="projected")).mountPath = "/tmp/token" else . end' ;;
    readOnly) filter='.[] |= if .kind=="Job" then (.spec.template.spec.containers[0].volumeMounts[] | select(.name=="projected")).readOnly = false else . end' ;;
  esac
  mutate_and_reject "$projected" "projected-$mutation" "$filter" projected app api-projected-app projected-app
done
mutate_and_reject "$projected" projected-missing-volume '.[] |= if .kind=="Job" then .spec.template.spec.volumes |= map(select(.name != "projected")) else . end' projected app api-projected-app projected-app
mutate_and_reject "$projected" projected-missing-mount '.[] |= if .kind=="Job" then .spec.template.spec.containers[0].volumeMounts = [{name:"memory",mountPath:"/run/lumen"}] | . else . end' projected app api-projected-app projected-app

source "$renderer"
patch="$tmp/patch.json"; valid_patch="$tmp/api-default.patch.json"
cp "$tmp/api-default-app/api.patch.json" "$valid_patch"
owned_files+=("$valid_patch" "$patch")
jq '.[0].path = "/spec/template/spec/containers/0/image"' "$valid_patch" >"$patch"
changed "$valid_patch" "$patch" forbidden-image
expect_reject forbidden-image validate_patch api default "$patch"
for op in remove move copy test; do jq --arg op "$op" '.[0].op = $op' "$valid_patch" >"$patch"; changed "$valid_patch" "$patch" "$op-operation"; expect_reject "$op operation" validate_patch api default "$patch"; done
jq '.[0] as $op | . + [$op]' "$valid_patch" >"$patch"
changed "$valid_patch" "$patch" duplicate-path
expect_reject duplicate-path validate_patch api default "$patch"
valid_projected_patch="$tmp/api-projected.patch.json"
cp "$tmp/api-projected-app/api.patch.json" "$valid_projected_patch"
owned_files+=("$valid_projected_patch")
jq '.[-2].value.projected.sources[0].serviceAccountToken.audience = "wrong"' "$valid_projected_patch" >"$patch"
changed "$valid_projected_patch" "$patch" bad-projected-literal
expect_reject bad-projected-literal validate_patch api projected "$patch"

repo_output="$root/.contract-forbidden-output"
[[ ! -e "$repo_output" && ! -L "$repo_output" ]] || fail 'repo output negative path already exists'
expect_reject repo-output "$renderer" client --out-dir "$repo_output" --client-namespace x --run-id x
[[ ! -e "$repo_output" && ! -L "$repo_output" ]] || fail 'renderer created forbidden repo output'
existing="$tmp/existing"; mkdir "$existing"; owned_dirs+=("$existing"); expect_reject existing-output "$renderer" client --out-dir "$existing" --client-namespace x --run-id x
link="$tmp/link"; ln -s "$tmp/missing" "$link"; owned_files+=("$link"); expect_reject symlink-output "$renderer" client --out-dir "$link" --client-namespace x --run-id x
expect_reject unknown-flag "$renderer" client --out-dir "$tmp/unknown" --client-namespace x --run-id x --unknown
expect_reject duplicate-flag "$renderer" client --out-dir "$tmp/duplicate" --client-namespace x --run-id x --run-id y
expect_reject missing-required "$renderer" client --out-dir "$tmp/missing-required" --client-namespace x
expect_reject missing-value "$renderer" client --out-dir "$tmp/missing-value" --client-namespace x --run-id
expect_reject invalid-component "$renderer" nope --out-dir "$tmp/invalid-component" --client-namespace x --run-id x
expect_reject forbidden-token-account "$renderer" api --out-dir "$tmp/forbidden-pair" --client-namespace x --run-id x --job api --account default --token-mode default --runtime-namespace lumen --service lumen --method POST --path /x --request-file "$fixture" --expected-status 2xx --required-id id --rejected-id none --row-label row

for index in "${!base_files[@]}"; do [[ "$(sha256 "${base_files[$index]}")" == "${before[$index]}" ]] || fail 'checked-in base changed'; done
[[ "$(sha256 "$fixture_source")" == "$fixture_source_before" ]] || fail 'checked-in request fixture changed'
[[ "$(sha256 "$fixture")" == f430728441dec341edc81c69dbd199da7795a994695ddc194e34dd80ece94ea0 ]] || fail 'runtime request fixture changed'
echo 'kustomize standalone acceptance contract: passed'
