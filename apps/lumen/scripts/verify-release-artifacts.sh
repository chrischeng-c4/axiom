#!/usr/bin/env bash
# Verify a Lumen promotion. This script never creates or changes a tag, image,
# release, signature, provenance, or attestation.
set -euo pipefail

usage() {
  cat >&2 <<'EOF'
Usage: verify-release-artifacts.sh \
  --repo <owner/repo> --tag <lumen@semver> --commit <40-hex> \
  --candidate-run-id <id> --mode <candidate|fixture|public> \
  [--candidate-receipt-dir <dir> --release-assets-dir <dir> --output <path> \
   --standalone-gke-receipt <path> --standalone-gke-receipt-sidecar <path>]

candidate: prove the immutable tag, tag ruleset, candidate run, receipt, image,
and existing attestations. fixture: compare supplied candidate and release bytes
without network access. public: also verify the published GitHub Release.
EOF
  exit 2
}

fail() { printf '%s\n' "$*" >&2; exit 1; }
require_file() { [[ -f "$1" && ! -L "$1" ]] || fail "required regular file is absent: $1"; }
sha256_file() { if command -v sha256sum >/dev/null 2>&1; then sha256sum "$1" | awk '{print $1}'; else shasum -a 256 "$1" | awk '{print $1}'; fi; }
sha256_stdin() { if command -v sha256sum >/dev/null 2>&1; then sha256sum | awk '{print $1}'; else shasum -a 256 | awk '{print $1}'; fi; }

image_digest_or_absent() {
  local ref="$1" error manifest status
  error="$(mktemp)"
  set +e
  manifest="$(docker buildx imagetools inspect "$ref" --format '{{json .Manifest}}' 2>"$error")"
  status=$?
  set -e
  if [[ "$status" == 0 ]]; then
    rm -f "$error"
    jq -er '.digest' <<<"$manifest"
    return 0
  fi
  if grep -Eqi 'manifest unknown|not found' "$error"; then
    rm -f "$error"
    return 0
  fi
  cat "$error" >&2
  rm -f "$error"
  fail "cannot prove GHCR image state: $ref"
}

targets() {
  cat <<'EOF'
aarch64-apple-darwin
x86_64-unknown-linux-gnu
aarch64-unknown-linux-gnu
x86_64-unknown-linux-musl
aarch64-unknown-linux-musl
EOF
}

host_target() {
  case "$(uname -s):$(uname -m)" in
    Darwin:arm64|Darwin:aarch64) printf '%s\n' aarch64-apple-darwin ;;
    Linux:x86_64|Linux:amd64) printf '%s\n' x86_64-unknown-linux-gnu ;;
    Linux:aarch64|Linux:arm64) printf '%s\n' aarch64-unknown-linux-gnu ;;
    *) fail "unsupported verifier host: $(uname -s) $(uname -m)" ;;
  esac
}

validate_receipt() {
  local manifest="$1" sidecar="$2" receipt_dir="$3" actual expected name extra target archive sidecar_name
  require_file "$manifest"; require_file "$sidecar"
  actual="$(sha256_file "$manifest")"
  read -r expected name extra <"$sidecar" || fail "cannot read final receipt sidecar"
  [[ "$expected" == "$actual" && "$name" == "${manifest##*/}" && -z "${extra:-}" ]] || fail "final receipt sidecar does not bind exact bytes"
  jq -e --arg repo "$REPO" --arg tag "$TAG" --arg commit "$COMMIT" --arg run "$CANDIDATE_RUN_ID" '
    (keys | sort) == ["artifacts","candidate_tag","commit","image","jobs","pr","repository","run_attempt","run_id","run_url","sboms","schema","source_ref","tag","version","workflow_id","workflow_path","workflow_ref"] and
    .schema == "cclab.lumen.candidate-manifest.v3" and .repository == $repo and .tag == $tag and .commit == $commit and
    .run_id == $run and .source_ref == "refs/heads/main" and .workflow_path == ".github/workflows/lumen-release-candidate.yml" and
    .workflow_ref == ($repo + "/.github/workflows/lumen-release-candidate.yml@refs/heads/main") and
    .run_url == ("https://github.com/" + $repo + "/actions/runs/" + .run_id + "/attempts/" + .run_attempt) and
    .jobs == {identity:"success",build:"success",manifest:"success","ghcr-image-and-attest":"success","verify-candidate":"success","verify-libraries":"success","kind-amd64":"success","kind-arm64":"success",result:"success"} and
    (.image | (
      (keys | sort) == ["amd64_digest","arm64_digest","repository","root_digest"] and
      .repository == "ghcr.io/chrischeng-c4/lumen" and
      ([.root_digest,.amd64_digest,.arm64_digest] | all(test("^sha256:[0-9a-f]{64}$")))
    )) and
    (.artifacts | type == "array" and length == 5 and all(.[]; .archive == ("lumen-" + .target + ".tar.gz") and .sidecar == (.archive + ".sha256") and (.archive_sha256 | test("^[0-9a-f]{64}$")) and (.sidecar_sha256 | test("^[0-9a-f]{64}$")))) and
    (.sboms.amd64.file == "spdx-amd64.json" and (.sboms.amd64.sha256 | test("^[0-9a-f]{64}$")) and .sboms.arm64.file == "spdx-arm64.json" and (.sboms.arm64.sha256 | test("^[0-9a-f]{64}$")))
  ' "$manifest" >/dev/null || fail "candidate final receipt contract changed"
  while IFS= read -r target; do
    archive="$(jq -er --arg t "$target" '.artifacts[] | select(.target == $t) | .archive' "$manifest")"
    sidecar_name="$(jq -er --arg t "$target" '.artifacts[] | select(.target == $t) | .sidecar' "$manifest")"
    require_file "$receipt_dir/$archive"; require_file "$receipt_dir/$sidecar_name"
    [[ "$(sha256_file "$receipt_dir/$archive")" == "$(jq -er --arg t "$target" '.artifacts[] | select(.target == $t) | .archive_sha256' "$manifest")" ]] || fail "candidate archive hash mismatch: $archive"
    [[ "$(sha256_file "$receipt_dir/$sidecar_name")" == "$(jq -er --arg t "$target" '.artifacts[] | select(.target == $t) | .sidecar_sha256' "$manifest")" ]] || fail "candidate checksum-sidecar hash mismatch: $sidecar_name"
  done < <(targets)
  for target in amd64 arm64; do
    require_file "$receipt_dir/spdx-${target}.json"
    [[ "$(sha256_file "$receipt_dir/spdx-${target}.json")" == "$(jq -er --arg t "$target" '.sboms[$t].sha256' "$manifest")" ]] || fail "candidate SPDX hash mismatch: $target"
    jq -e '.spdxVersion == "SPDX-2.3"' "$receipt_dir/spdx-${target}.json" >/dev/null || fail "candidate SPDX is invalid: $target"
  done
}

verify_archive_pair() {
  local dir="$1" target="$2" archive="lumen-${2}.tar.gz" sidecar="lumen-${2}.tar.gz.sha256" expected listed extra actual members mode
  require_file "$dir/$archive"; require_file "$dir/$sidecar"
  read -r expected listed extra <"$dir/$sidecar" || fail "cannot read release checksum: $sidecar"
  actual="$(sha256_file "$dir/$archive")"
  [[ "$expected" == "$actual" && "$listed" == "$archive" && -z "${extra:-}" ]] || fail "release checksum mismatch: $archive"
  members="$(tar -tzf "$dir/$archive" | LC_ALL=C sort)" || fail "cannot list release archive: $archive"
  [[ "$members" == "$(printf '%s\n' "lumen-${target}/" "lumen-${target}/README.md" "lumen-${target}/lumen" | LC_ALL=C sort)" ]] || fail "release archive members changed: $archive"
  mode="$(tar -tvzf "$dir/$archive" | awk -v path="lumen-${target}/lumen" '$NF == path { print $1 }')" || fail "cannot read release binary mode: $archive"
  [[ "$mode" =~ ^-.{2}x ]] || fail "release binary is not executable: $archive"
}

verify_release_assets_against_receipt() {
  local receipt_dir release_dir manifest expected actual target archive sidecar host private_home unpack binary version
  receipt_dir="$1"; release_dir="$2"; manifest="$receipt_dir/final-candidate-manifest.json"
  expected="$({ while IFS= read -r target; do printf 'lumen-%s.tar.gz\nlumen-%s.tar.gz.sha256\n' "$target" "$target"; done < <(targets); printf 'spdx-amd64.json\nspdx-arm64.json\n'; if [[ "$TAG" == "lumen@0.4.29" ]]; then printf 'lumen-standalone-gke-receipt.json\nlumen-standalone-gke-receipt.json.sha256\n'; fi; } | LC_ALL=C sort)"
  actual="$(find "$release_dir" -maxdepth 1 -type f -exec basename {} \; | LC_ALL=C sort)"
  [[ "$actual" == "$expected" ]] || fail "public release assets are not the exact receipt asset set"
  while IFS= read -r target; do
    archive="lumen-${target}.tar.gz"
    sidecar="${archive}.sha256"
    verify_archive_pair "$release_dir" "$target"
    [[ "$(sha256_file "$release_dir/$archive")" == "$(jq -er --arg t "$target" '.artifacts[] | select(.target == $t) | .archive_sha256' "$manifest")" ]] || fail "public archive hash differs from final candidate receipt: $archive"
    [[ "$(sha256_file "$release_dir/$sidecar")" == "$(jq -er --arg t "$target" '.artifacts[] | select(.target == $t) | .sidecar_sha256' "$manifest")" ]] || fail "public checksum-sidecar hash differs from final candidate receipt: $sidecar"
  done < <(targets)
  for target in amd64 arm64; do
    jq -en --slurpfile candidate "$receipt_dir/spdx-${target}.json" --slurpfile release "$release_dir/spdx-${target}.json" '$candidate[0] == $release[0]' >/dev/null || fail "published SPDX bytes differ from candidate: $target"
  done
  verify_public_standalone_gke_receipt "$release_dir"
  host="$(host_target)"; private_home="$(mktemp -d)"; unpack="$(mktemp -d)"
  trap 'rm -rf "${private_home:-}" "${unpack:-}"' RETURN
  tar -xzf "$release_dir/lumen-${host}.tar.gz" -C "$unpack"
  binary="$unpack/lumen-${host}/lumen"
  [[ -x "$binary" ]] || fail "host binary is missing from public asset"
  version="$(env -i HOME="$private_home" PATH="$PATH" TMPDIR="$private_home" "$binary" --version)" || fail "public host binary cannot report version"
  [[ "$version" == "lumen ${TAG#lumen@}" ]] || fail "public host binary version mismatch: $version"
  [[ ! -e "$private_home/.aws" && ! -e "$private_home/.config/gcloud" ]] || fail "public host binary wrote credential state"
  rm -rf "$private_home" "$unpack"; trap - RETURN
}

verify_annotated_tag_and_ruleset() {
  local encoded_tag ref tag_object rulesets details id
  encoded_tag="${TAG/@/%40}"
  ref="$(gh api "repos/${REPO}/git/ref/tags/${encoded_tag}")"
  [[ "$(jq -r '.object.type' <<<"$ref")" == tag ]] || fail "release tag is not annotated"
  tag_object="$(gh api "repos/${REPO}/git/tags/$(jq -er '.object.sha' <<<"$ref")")"
  [[ "$(jq -r '.object.type' <<<"$tag_object")" == commit && "$(jq -r '.object.sha' <<<"$tag_object")" == "$COMMIT" ]] || fail "annotated tag does not peel to the exact candidate commit"
  rulesets='[]'
  while IFS= read -r id; do
    details="$(gh api "repos/${REPO}/rulesets/${id}")"
    rulesets="$(jq -c --argjson detail "$details" '. + [$detail]' <<<"$rulesets")"
  done < <(gh api --paginate "repos/${REPO}/rulesets?per_page=100" | jq -r '.[] | .id')
  jq -e '
    any(.[];
      .target == "tag" and .enforcement == "active" and
      (.conditions.ref_name.include == ["refs/tags/lumen@*"]) and
      ((.conditions.ref_name.exclude // []) == []) and
      ([.rules[].type] | sort == ["deletion","update"]) and
      ((.bypass_actors // []) | length == 0))
  ' <<<"$rulesets" >/dev/null || fail "exact active immutable lumen tag ruleset is absent"
}

flatten_paginated_jobs() { jq -cs '[.[] | .jobs[]]'; }
flatten_paginated_artifacts() { jq -cs '[.[] | .artifacts[]]'; }
validate_candidate_job_inventory() {
  jq -e '
    length == 13 and all(.[]; .status == "completed" and .conclusion == "success") and
    ([.[].name] | sort == ["bind candidate inputs","build (aarch64-apple-darwin)","build (aarch64-unknown-linux-gnu)","build (aarch64-unknown-linux-musl)","build (x86_64-unknown-linux-gnu)","build (x86_64-unknown-linux-musl)","build candidate image and attest","candidate identity","final candidate receipt","kind e2e (amd64)","kind e2e (arm64)","verify exact candidate gates","verify service and Raft library gates"])
  ' >/dev/null || fail "candidate attempt does not contain the exact successful execution set"
}

fetch_candidate_receipt() {
  local run attempt candidate_workflow_id jobs artifact_name artifacts artifact_id zip
  run="$(gh api "repos/${REPO}/actions/runs/${CANDIDATE_RUN_ID}")"
  attempt="$(jq -er '.run_attempt' <<<"$run")"
  candidate_workflow_id="$(gh api "repos/${REPO}/actions/workflows/lumen-release-candidate.yml" --jq '.id')"
  jq -e --arg commit "$COMMIT" --argjson workflow "$candidate_workflow_id" '
    .event == "workflow_dispatch" and .status == "completed" and .conclusion == "success" and
    .head_branch == "main" and .head_sha == $commit and .workflow_id == $workflow and
    .head_repository.full_name == "chrischeng-c4/axiom"
  ' <<<"$run" >/dev/null || fail "candidate run identity or conclusion changed"
  jobs="$(gh api --paginate "repos/${REPO}/actions/runs/${CANDIDATE_RUN_ID}/attempts/${attempt}/jobs?filter=latest&per_page=100" | flatten_paginated_jobs)"
  validate_candidate_job_inventory <<<"$jobs"
  artifact_name="lumen-release-candidate-${CANDIDATE_RUN_ID}-${attempt}"
  artifacts="$(gh api --paginate "repos/${REPO}/actions/runs/${CANDIDATE_RUN_ID}/artifacts?per_page=100" | flatten_paginated_artifacts)"
  artifact_id="$(jq -er --arg name "$artifact_name" '[.[] | select(.name == $name and .expired == false)] | if length == 1 then .[0].id else error("exact receipt artifact absent") end' <<<"$artifacts")"
  zip="$(mktemp)"
  gh api -H 'Accept: application/vnd.github+json' "repos/${REPO}/actions/artifacts/${artifact_id}/zip" >"$zip"
  mkdir -p "$CANDIDATE_RECEIPT_DIR"
  unzip -q "$zip" -d "$CANDIDATE_RECEIPT_DIR"
  rm -f "$zip"
  CANDIDATE_ATTEMPT="$attempt"
  validate_receipt "$CANDIDATE_RECEIPT_DIR/final-candidate-manifest.json" "$CANDIDATE_RECEIPT_DIR/final-candidate-manifest.json.sha256" "$CANDIDATE_RECEIPT_DIR"
}

verify_candidate_supply_chain() {
  local manifest="$CANDIDATE_RECEIPT_DIR/final-candidate-manifest.json" root amd64 arm64 candidate_tag
  root="$(jq -er '.image.root_digest' "$manifest")"; amd64="$(jq -er '.image.amd64_digest' "$manifest")"; arm64="$(jq -er '.image.arm64_digest' "$manifest")"; candidate_tag="$(jq -er '.candidate_tag' "$manifest")"
  apps/lumen/scripts/verify-release-candidate.sh \
    --repo "$REPO" --version "${TAG#lumen@}" --commit "$COMMIT" --run-id "$CANDIDATE_RUN_ID" --run-attempt "$CANDIDATE_ATTEMPT" \
    --manifest "$manifest" --manifest-sidecar "$CANDIDATE_RECEIPT_DIR/final-candidate-manifest.json.sha256" --artifacts-dir "$CANDIDATE_RECEIPT_DIR" \
    --image "ghcr.io/chrischeng-c4/lumen@${root}" --candidate-tag "$candidate_tag" --amd64-digest "$amd64" --arm64-digest "$arm64" --mode full
}

validate_standalone_gke_receipt() {
  local receipt="$STANDALONE_GKE_RECEIPT" sidecar="$STANDALONE_GKE_RECEIPT_SIDECAR" manifest actual bytes target archive cli_sha root amd64 arm64
  if [[ "$TAG" != "lumen@0.4.29" ]]; then
    [[ -z "$receipt" && -z "$sidecar" ]] || fail "standalone GKE receipt is only valid for lumen@0.4.29"
    STANDALONE_GKE_RECEIPT_SHA256=""
    return 0
  fi
  [[ -n "$receipt" && -n "$sidecar" ]] || fail "lumen@0.4.29 requires the standalone GKE receipt and sidecar"
  require_file "$receipt"; require_file "$sidecar"
  bytes="$(wc -c <"$receipt" | tr -d '[:space:]')"
  [[ "$bytes" =~ ^[0-9]+$ && "$bytes" -gt 0 && "$bytes" -le 16384 ]] || fail "standalone GKE receipt size is invalid"
  actual="$(sha256_file "$receipt")"
  [[ "$actual" =~ ^[0-9a-f]{64}$ ]] || fail "standalone GKE receipt hash is invalid"
  cmp -s "$sidecar" <(printf '%s  lumen-standalone-gke-receipt.json\n' "$actual") || fail "standalone GKE receipt sidecar is not exact"
  manifest="$CANDIDATE_RECEIPT_DIR/final-candidate-manifest.json"
  root="$(jq -er '.image.root_digest' "$manifest")"
  amd64="$(jq -er '.image.amd64_digest' "$manifest")"
  arm64="$(jq -er '.image.arm64_digest' "$manifest")"
  jq -e --arg manifest_sha "$(sha256_file "$manifest")" --arg commit "$COMMIT" --arg run "$CANDIDATE_RUN_ID" --arg attempt "$CANDIDATE_ATTEMPT" --arg root "$root" --arg amd64 "$amd64" --arg arm64 "$arm64" '
    . as $receipt |
    (keys | sort) == ["candidate","complete","matrix","redaction","schema","stage"] and
    .schema == "lumen.standalone-gke-receipt/v2" and .stage == "slice-b-live" and .complete == true and
    (.candidate | (keys | sort) == ["amd64_digest","arm64_digest","commit","controller_cli","manifest_sha256","repository","root_digest","run_attempt","run_id","version","workflow_ref"]) and
    .candidate.repository == "chrischeng-c4/axiom" and .candidate.version == "0.4.29" and .candidate.commit == $commit and
    .candidate.workflow_ref == "chrischeng-c4/axiom/.github/workflows/lumen-release-candidate.yml@refs/heads/main" and
    .candidate.run_id == $run and .candidate.run_attempt == $attempt and .candidate.manifest_sha256 == $manifest_sha and
    .candidate.root_digest == $root and .candidate.amd64_digest == $amd64 and .candidate.arm64_digest == $arm64 and
    (.candidate.controller_cli | (keys | sort) == ["sha256","target"] and (.sha256 | type == "string" and test("^[0-9a-f]{64}$")) and
      (.target == "aarch64-apple-darwin" or .target == "x86_64-unknown-linux-gnu" or .target == "aarch64-unknown-linux-gnu" or .target == "x86_64-unknown-linux-musl" or .target == "aarch64-unknown-linux-musl")) and
    ($receipt.matrix as $matrix |
      ($matrix | keys | sort) == ["admin_backup_restore","allowed_ksa","application_admin_403","bad_token","cleanup","clusterip_only","missing_token","network_policy","pod_replacement","pvc_recovery","required_continuity","subjectaccessreview","tokenreview","unlisted_ksa","vertical_resize"] and
      ($matrix | del(.required_continuity) | to_entries | all(.[]; (.value | type == "string" and . == "passed")))) and
    ($receipt.matrix.required_continuity as $continuity |
      ($continuity | keys | sort) == ["allowed_delta","audience","denied_delta","observed_runtime_image_digest","profile","projected_allowed_2xx","projected_unlisted_403","same_ksa_default_token_401","scheduled_node_arch","scheduled_runtime_child_digest","subjectaccessreview_delta","tokenreview_delta"] and
      $continuity.profile == "LUMEN_AUTH=required" and $continuity.audience == "lumen.axiom.dev" and
      $continuity.observed_runtime_image_digest == $root and
      (($continuity.scheduled_node_arch == "amd64" and $continuity.scheduled_runtime_child_digest == $amd64) or ($continuity.scheduled_node_arch == "arm64" and $continuity.scheduled_runtime_child_digest == $arm64)) and
      $continuity.projected_allowed_2xx == "passed" and $continuity.same_ksa_default_token_401 == "passed" and $continuity.projected_unlisted_403 == "passed" and
      ([$continuity.tokenreview_delta,$continuity.subjectaccessreview_delta,$continuity.allowed_delta,$continuity.denied_delta] | all(.[]; type == "number" and floor == . and . > 0))) and
    $receipt.redaction == {kubeconfig_retained:false,token_retained:false,authorization_retained:false,secret_retained:false,cluster_identity_retained:false,command_output_retained:false,canary_scan:true}
  ' "$receipt" >/dev/null || fail "standalone GKE receipt contract changed"
  target="$(jq -er '.candidate.controller_cli.target' "$receipt")"
  archive="$(jq -er --arg target "$target" '.artifacts[] | select(.target == $target) | .archive' "$manifest")"
  cli_sha="$(tar -xOzf "$CANDIDATE_RECEIPT_DIR/$archive" "lumen-${target}/lumen" | sha256_stdin)"
  [[ "$cli_sha" == "$(jq -er '.candidate.controller_cli.sha256' "$receipt")" ]] || fail "standalone GKE controller CLI hash does not bind candidate archive"
  STANDALONE_GKE_RECEIPT_SHA256="$actual"
}

write_identity() {
  local manifest="$CANDIDATE_RECEIPT_DIR/final-candidate-manifest.json" identity
  identity="$(jq -nc --arg repo "$REPO" --arg tag "$TAG" --arg commit "$COMMIT" --arg candidate_run_id "$CANDIDATE_RUN_ID" --arg candidate_attempt "$CANDIDATE_ATTEMPT" --arg root "$(jq -er '.image.root_digest' "$manifest")" --arg amd64 "$(jq -er '.image.amd64_digest' "$manifest")" --arg arm64 "$(jq -er '.image.arm64_digest' "$manifest")" --arg pr "$(jq -er '.pr.url' "$manifest")" --arg candidate_url "$(jq -er '.run_url' "$manifest")" '{repository:$repo,tag:$tag,commit:$commit,candidate_run_id:$candidate_run_id,candidate_attempt:$candidate_attempt,root_digest:$root,amd64_digest:$amd64,arm64_digest:$arm64,pr_url:$pr,candidate_url:$candidate_url}')"
  if [[ "$TAG" == "lumen@0.4.29" ]]; then
    jq -nc --argjson identity "$identity" --arg receipt "$STANDALONE_GKE_RECEIPT_SHA256" '$identity + {standalone_gke_receipt_sha256:$receipt}' >"$OUTPUT"
  else
    printf '%s\n' "$identity" >"$OUTPUT"
  fi
}

verify_public_standalone_gke_receipt() {
  local release_dir="$1" public_receipt public_sidecar
  public_receipt="$release_dir/lumen-standalone-gke-receipt.json"
  public_sidecar="$release_dir/lumen-standalone-gke-receipt.json.sha256"
  [[ "$TAG" == "lumen@0.4.29" ]] || return 0
  require_file "$public_receipt"; require_file "$public_sidecar"
  cmp -s "$STANDALONE_GKE_RECEIPT" "$public_receipt" || fail "public standalone GKE receipt bytes differ from verified receipt"
  cmp -s "$STANDALONE_GKE_RECEIPT_SIDECAR" "$public_sidecar" || fail "public standalone GKE receipt sidecar differs from verified receipt"
}

verify_latest_is_safe() {
  local root="$1" image_repo="ghcr.io/chrischeng-c4/lumen" latest releases tag version digest
  latest="$(image_digest_or_absent "${image_repo}:latest")"
  [[ -n "$latest" ]] || fail "public latest image tag is absent"
  [[ "$latest" == "$root" ]] && return 0
  releases="$(gh release list --repo "$REPO" --limit 100 --json tagName,isDraft)"
  while IFS= read -r tag; do
    version="${tag#lumen@}"
    [[ "$tag" =~ ^lumen@[0-9]+\.[0-9]+\.[0-9]+$ ]] || continue
    [[ "$(printf '%s\n%s\n' "${TAG#lumen@}" "$version" | sort -V | tail -n1)" == "$version" && "$version" != "${TAG#lumen@}" ]] || continue
    digest="$(image_digest_or_absent "${image_repo}:${version}")"
    [[ -n "$digest" ]] || fail "newer published semver image tag is absent: $version"
    [[ "$digest" == "$latest" ]] && return 0
  done < <(jq -r '.[] | select(.isDraft == false) | .tagName' <<<"$releases")
  fail "latest points to neither this root nor a newer published semver root"
}

verify_public_release_notes() {
  local receipt_sha256="${1:-}" release_json
  release_json="$(cat)"
  jq -e '
    (.body | if type == "string" then split("\n") else error("release body is not a string") end) as $lines |
    ([ $lines[] | select(. == "- Compatibility: shipped Docker images default to durable segment storage at /var/lib/lumen/data; bare lumen serve stays ephemeral without --data-dir or LUMEN_DATA_DIR. A 0.4.28 segment volume upgrades one way on first 0.4.29 start; in-place downgrade is unsupported.") ] | length) == 1 and
    ([ $lines[] | select(. == "- Compatibility: no API, CRD, or runtime-default migration.") ] | length) == 0 and
    ([ $lines[] | select(contains("0.4.28") and . != "- Compatibility: shipped Docker images default to durable segment storage at /var/lib/lumen/data; bare lumen serve stays ephemeral without --data-dir or LUMEN_DATA_DIR. A 0.4.28 segment volume upgrades one way on first 0.4.29 start; in-place downgrade is unsupported.") ] | length) == 0 and
    ([$lines[] | sub("^\\s+"; "")] | all(.[]; (startswith(">") | not) and (startswith("```") | not) and (startswith("~~~") | not) and (contains("<!--") | not)))
  ' <<<"$release_json" >/dev/null || fail "public GitHub Release notes do not bind exact compatibility semantics"
  [[ -z "$receipt_sha256" ]] || jq -e --arg receipt "$receipt_sha256" '
    (.body | if type == "string" then split("\n") else error("release body is not a string") end) as $lines |
    ([ $lines[] | select(. == ("- Standalone GKE receipt SHA-256: " + $receipt)) ] | length) == 1 and
    ([$lines[] | sub("^\\s+"; "")] | all(.[]; (startswith(">") | not) and (startswith("```") | not) and (startswith("~~~") | not) and (contains("<!--") | not)))
  ' <<<"$release_json" >/dev/null || fail "public GitHub Release notes do not bind exact standalone GKE receipt hash"
}

verify_public_release() {
  local release_json release_dir manifest root amd64 arm64 pr_url candidate_url semver image_repo expected_assets actual_assets
  release_json="$(gh release view "$TAG" --repo "$REPO" --json assets,isDraft,tagName,targetCommitish,url,body)"
  jq -e --arg tag "$TAG" --arg commit "$COMMIT" '.tagName == $tag and .isDraft == false and .targetCommitish == $commit' <<<"$release_json" >/dev/null || fail "public GitHub Release identity changed"
  if [[ "$TAG" == "lumen@0.4.29" ]]; then
    verify_public_release_notes "$STANDALONE_GKE_RECEIPT_SHA256" <<<"$release_json"
  else
    verify_public_release_notes <<<"$release_json"
  fi
  manifest="$CANDIDATE_RECEIPT_DIR/final-candidate-manifest.json"
  root="$(jq -er '.image.root_digest' "$manifest")"; amd64="$(jq -er '.image.amd64_digest' "$manifest")"; arm64="$(jq -er '.image.arm64_digest' "$manifest")"; pr_url="$(jq -er '.pr.url' "$manifest")"; candidate_url="$(jq -er '.run_url' "$manifest")"
  jq -e --arg repo "$REPO" --arg commit "$COMMIT" --arg pr_url "$pr_url" --arg candidate_url "$candidate_url" --arg root "$root" --arg amd64 "$amd64" --arg arm64 "$arm64" --arg receipt_sha256 "$STANDALONE_GKE_RECEIPT_SHA256" '
    (.body | if type == "string" then split("\n") else error("release body is not a string") end) as $lines |
    ($lines | index("- Source commit: " + $commit) != null) and
    ($lines | index("- Pull request: " + $pr_url) != null) and
    ($lines | index("- Candidate run: " + $candidate_url) != null) and
    ($lines | index("- Root index digest: " + $root) != null) and
    ($lines | index("- linux/amd64 digest: " + $amd64) != null) and
    ($lines | index("- linux/arm64 digest: " + $arm64) != null) and
    (if $tag == "lumen@0.4.29" then ($lines | index("- Standalone GKE receipt SHA-256: " + $receipt_sha256) != null) else true end) and
    ($lines | index("- Release path: landed main -> untagged candidate verification -> protected annotated tag -> promotion of the same candidate digest.") != null) and
    ($lines | index("- Placement path: a non-empty nodeSelector with the default initialMachineType skips the legacy capacity catalog.") != null) and
    ($lines | index("- Legacy placement path: an empty selector, tolerations-only placement, or a non-default initialMachineType still requires lumen-system/lumen-capacity-catalog.") != null) and
    ($lines | index("- Compatibility: shipped Docker images default to durable segment storage at /var/lib/lumen/data; bare lumen serve stays ephemeral without --data-dir or LUMEN_DATA_DIR. A 0.4.28 segment volume upgrades one way on first 0.4.29 start; in-place downgrade is unsupported.") != null) and
    any($lines[]; test("^- Promotion run: https://github\\.com/" + $repo + "/actions/runs/[0-9]+/attempts/[0-9]+$"))
  ' <<<"$release_json" >/dev/null || fail "public GitHub Release notes do not bind exact promotion evidence"
  expected_assets="$({ while IFS= read -r target; do printf 'lumen-%s.tar.gz\nlumen-%s.tar.gz.sha256\n' "$target" "$target"; done < <(targets); printf 'spdx-amd64.json\nspdx-arm64.json\n'; if [[ "$TAG" == "lumen@0.4.29" ]]; then printf 'lumen-standalone-gke-receipt.json\nlumen-standalone-gke-receipt.json.sha256\n'; fi; } | LC_ALL=C sort)"
  actual_assets="$(jq -r '.assets[].name' <<<"$release_json" | LC_ALL=C sort)"
  [[ "$actual_assets" == "$expected_assets" ]] || fail "public GitHub Release asset inventory is not exact"
  release_dir="$(mktemp -d)"; trap 'rm -rf "${release_dir:-}"' RETURN
  gh release download "$TAG" --repo "$REPO" --dir "$release_dir" --pattern 'lumen-*.tar.gz' --pattern 'lumen-*.tar.gz.sha256' --pattern 'spdx-*.json' --pattern 'lumen-standalone-gke-receipt.json' --pattern 'lumen-standalone-gke-receipt.json.sha256'
  verify_release_assets_against_receipt "$CANDIDATE_RECEIPT_DIR" "$release_dir"
  semver="${TAG#lumen@}"; image_repo="ghcr.io/chrischeng-c4/lumen"
  [[ "$(docker buildx imagetools inspect "${image_repo}:${semver}" --format '{{json .Manifest}}' | jq -er '.digest')" == "$root" ]] || fail "semver image tag does not bind candidate root"
  verify_latest_is_safe "$root"
  rm -rf "$release_dir"; trap - RETURN
}

if [[ "${BASH_SOURCE[0]}" != "$0" ]]; then return 0; fi

REPO=""; TAG=""; COMMIT=""; CANDIDATE_RUN_ID=""; MODE=""; CANDIDATE_RECEIPT_DIR=""; RELEASE_ASSETS_DIR=""; OUTPUT=""; STANDALONE_GKE_RECEIPT=""; STANDALONE_GKE_RECEIPT_SIDECAR=""; STANDALONE_GKE_RECEIPT_SHA256=""
while [[ $# -gt 0 ]]; do
  [[ $# -ge 2 ]] || usage
  case "$1" in
    --repo) REPO="$2" ;; --tag) TAG="$2" ;; --commit) COMMIT="$2" ;; --candidate-run-id) CANDIDATE_RUN_ID="$2" ;;
    --mode) MODE="$2" ;; --candidate-receipt-dir) CANDIDATE_RECEIPT_DIR="$2" ;; --release-assets-dir) RELEASE_ASSETS_DIR="$2" ;; --output) OUTPUT="$2" ;;
    --standalone-gke-receipt) STANDALONE_GKE_RECEIPT="$2" ;; --standalone-gke-receipt-sidecar) STANDALONE_GKE_RECEIPT_SIDECAR="$2" ;;
    *) usage ;;
  esac
  shift 2
done
[[ "$REPO" == "chrischeng-c4/axiom" && "$TAG" =~ ^lumen@[0-9]+\.[0-9]+\.[0-9]+$ && "$COMMIT" =~ ^[0-9a-f]{40}$ && "$CANDIDATE_RUN_ID" =~ ^[0-9]+$ ]] || fail "invalid promotion identity"
[[ "$MODE" == candidate || "$MODE" == fixture || "$MODE" == public ]] || usage

if [[ "$MODE" == fixture ]]; then
  [[ -n "$CANDIDATE_RECEIPT_DIR" && -n "$RELEASE_ASSETS_DIR" ]] || usage
  validate_receipt "$CANDIDATE_RECEIPT_DIR/final-candidate-manifest.json" "$CANDIDATE_RECEIPT_DIR/final-candidate-manifest.json.sha256" "$CANDIDATE_RECEIPT_DIR"
  CANDIDATE_ATTEMPT="$(jq -er '.run_attempt' "$CANDIDATE_RECEIPT_DIR/final-candidate-manifest.json")"
  validate_standalone_gke_receipt
  verify_release_assets_against_receipt "$CANDIDATE_RECEIPT_DIR" "$RELEASE_ASSETS_DIR"
  printf 'LOCAL FIXTURE ONLY: release bytes verified; this is not public release acceptance.\n'
  exit 0
fi

[[ -n "$OUTPUT" ]] || usage
CANDIDATE_RECEIPT_DIR="$(mktemp -d)"; trap 'rm -rf "${CANDIDATE_RECEIPT_DIR:-}"' EXIT
verify_annotated_tag_and_ruleset
fetch_candidate_receipt
verify_candidate_supply_chain
validate_standalone_gke_receipt
write_identity
if [[ "$MODE" == public ]]; then verify_public_release; fi
printf 'PROMOTION VERIFICATION PASS: %s %s candidate=%s/%s\n' "$TAG" "$COMMIT" "$CANDIDATE_RUN_ID" "$CANDIDATE_ATTEMPT"
