#!/usr/bin/env bash

# Shared validation for the immutable Sift and Rig candidate bundle.

sift_candidate_required_files() {
  printf '%s\n' \
    candidate-source.json \
    candidate-gate.json \
    candidate-gate.log \
    candidate-reservation.json \
    candidate-submit-intent.json \
    source-prefix.json \
    cloud-build-submit.json \
    cloud-build-final.json \
    cloud-build-source-object.json \
    cloud-build-source-binding.json \
    images.json \
    preexisting-artifact-registry.json \
    preexisting-cloud-build-source-bucket.json \
    preexisting-cloud-build-source-objects.txt \
    preexisting-sift-images.json \
    preexisting-rig-images.json \
    preexisting-sift-acceptance-runner-images.json
}

sift_candidate_file_sha256() {
  local input="$1"
  openssl dgst -sha256 "$input" | awk '{print $NF}'
}

sift_candidate_reservation_uri() {
  local source_prefix="$1"
  printf '%s/candidate-reservation.json\n' "$source_prefix"
}

sift_candidate_submit_intent_uri() {
  local source_prefix="$1"
  printf '%s/candidate-submit-intent.json\n' "$source_prefix"
}

verify_sift_candidate_reservation() {
  local reservation="$1"
  [[ -f "$reservation" && ! -L "$reservation" ]] || return 1
  jq -e '
    . as $r
    | type == "object"
    and keys == [
      "acquisition_id", "artifact_registry_repository", "created_at",
      "git_sha", "image_tag", "preexisting_images", "project_id",
      "region", "registry", "reservation_uri", "run_id", "schema",
      "source_bundle_sha256", "source_prefix"
    ]
    and .schema == "axiom.gcp.sift.candidate-reservation.v1"
    and (.project_id | type) == "string"
    and (.project_id | test("^[a-z][a-z0-9-]{4,62}$"))
    and (.region | type) == "string"
    and (.region | test("^[a-z]+-[a-z]+[0-9]$"))
    and (.artifact_registry_repository | type) == "string"
    and (.artifact_registry_repository | test("^[a-z][a-z0-9._-]{0,62}$"))
    and (.run_id | type) == "string"
    and (.run_id | test("^[a-z0-9][a-z0-9-]{0,17}$"))
    and (.git_sha | type) == "string"
    and (.git_sha | test("^[0-9a-f]{40}$"))
    and (.source_bundle_sha256 | type) == "string"
    and (.source_bundle_sha256 | test("^[0-9a-f]{64}$"))
    and (.acquisition_id | type) == "string"
    and (.acquisition_id | test("^[0-9a-f]{32}$"))
    and .image_tag == (.git_sha + "-" + .run_id + "-" + .acquisition_id)
    and .registry == (.region + "-docker.pkg.dev/" + .project_id + "/" + .artifact_registry_repository)
    and (.source_prefix | type) == "string"
    and (.source_prefix | endswith("/source/axiom-gcp-operator-" + $r.run_id))
    and .reservation_uri == (.source_prefix + "/candidate-reservation.json")
    and (.created_at | type) == "string" and (.created_at | length) > 0
    and (.preexisting_images | type) == "object"
    and (.preexisting_images | keys) == ["rig","sift","sift_acceptance_runner"]
    and (.preexisting_images.rig | type) == "array"
    and (.preexisting_images.sift | type) == "array"
    and (.preexisting_images.sift_acceptance_runner | type) == "array"
  ' "$reservation" >/dev/null
}

verify_sift_candidate_submit_intent() {
  local intent="$1"
  local reservation="$2"
  [[ -f "$intent" && ! -L "$intent" ]] || return 1
  verify_sift_candidate_reservation "$reservation" || return 1
  jq -e --slurpfile reservation "$reservation" '
    ($reservation[0]) as $r
    | type == "object"
    and keys == [
      "acquisition_id", "git_sha", "image_tag", "project_id", "region",
      "registry", "run_id", "schema", "source_bundle_sha256",
      "source_prefix", "submitted_at"
    ]
    and .schema == "axiom.gcp.sift.candidate-submit-intent.v1"
    and .project_id == $r.project_id
    and .region == $r.region
    and .run_id == $r.run_id
    and .git_sha == $r.git_sha
    and .source_bundle_sha256 == $r.source_bundle_sha256
    and .registry == $r.registry
    and .image_tag == $r.image_tag
    and .source_prefix == $r.source_prefix
    and .acquisition_id == $r.acquisition_id
    and (.submitted_at | type) == "string" and (.submitted_at | length) > 0
  ' "$intent" >/dev/null
}

verify_sift_candidate_build_receipt() {
  local candidate_receipt="$1"
  local build_receipt="$2"
  local source_object expected_hash
  [[ -f "$candidate_receipt" && ! -L "$candidate_receipt" \
    && -f "$build_receipt" && ! -L "$build_receipt" ]] || return 1
  source_object="$(dirname "$candidate_receipt")/cloud-build-source-object.json"
  [[ -f "$source_object" && ! -L "$source_object" ]] || return 1
  expected_hash="$(jq -er '.file_sha256["cloud-build-source-object.json"]' "$candidate_receipt")" || return 1
  [[ "$(sift_candidate_file_sha256 "$source_object")" == "$expected_hash" ]] || return 1
  jq -e \
    --slurpfile candidate "$candidate_receipt" \
    --slurpfile object "$source_object" '
      ($candidate[0]) as $c
      | type == "object"
      and .id == $c.cloud_build_id
      and .status == "SUCCESS"
      and .substitutions._GIT_SHA == $c.git_sha
      and .substitutions._RUN_ID == $c.run_id
      and .substitutions._SOURCE_BUNDLE_SHA256 == $c.source_bundle_sha256
      and .substitutions._REGISTRY == $c.registry
      and .substitutions._TAG == $c.image_tag
      and .substitutions._CANDIDATE_ACQUISITION_ID == $c.acquisition_id
      and .source.storageSource.bucket == ($c.source_object_uri
        | sub("^gs://"; "") | split("/")[0])
      and .source.storageSource.object == ($c.source_object_uri
        | sub("^gs://[^/]+/"; ""))
      and ($object[0].generation | type == "string" and test("^[1-9][0-9]*$"))
      and .source.storageSource.generation == $object[0].generation
      and .source.storageSource.bucket == $object[0].bucket
      and .source.storageSource.object == $object[0].name
      and .sourceProvenance.resolvedStorageSource == .source.storageSource
      and ((.tags // []) | index("sift-mvp") != null)
      and ((.tags // []) | index("axiom-run-" + $c.run_id) != null)
      and ((.tags // []) | index("axiom-source-" + $c.source_bundle_sha256) != null)
      and ((.tags // []) | index("axiom-acquisition-" + $c.acquisition_id) != null)
      and any(.results.images[]?;
        .name == ($c.registry + "/sift:" + $c.image_tag)
        and .digest == ($c.sift_image | split("@")[-1]))
      and any(.results.images[]?;
        .name == ($c.registry + "/rig:" + $c.image_tag)
        and .digest == ($c.rig_image | split("@")[-1]))
      and any(.results.images[]?;
        .name == ($c.registry + "/sift-acceptance-runner:" + $c.image_tag)
        and .digest == ($c.acceptance_runner_image | split("@")[-1]))
    ' "$build_receipt" >/dev/null
}

verify_sift_candidate_directory() {
  local candidate_dir="$1"
  local receipt="$candidate_dir/candidate.json"
  local name expected actual

  [[ "$candidate_dir" == /* && -d "$candidate_dir" && ! -L "$candidate_dir" ]] \
    || return 1
  [[ -f "$receipt" && ! -L "$receipt" ]] || return 1
  jq -e '
    . as $candidate
    | type == "object"
    and keys == [
      "acceptance_runner_image",
      "acquisition_id",
      "artifact_registry_repository",
      "cloud_build_id",
      "completed_at",
      "file_sha256",
      "git_sha",
      "image_tag",
      "project_id",
      "region",
      "registry",
      "reservation_uri",
      "rig_image",
      "run_id",
      "schema",
      "sift_image",
      "source_bundle_sha256",
      "source_object_uri",
      "source_prefix"
    ]
    and .schema == "axiom.gcp.sift.candidate.v1"
    and (.project_id | type) == "string"
    and (.project_id | test("^[a-z][a-z0-9-]{4,62}$"))
    and (.region | type) == "string"
    and (.region | test("^[a-z]+-[a-z]+[0-9]$"))
    and (.artifact_registry_repository | type) == "string"
    and (.artifact_registry_repository | test("^[a-z][a-z0-9._-]{0,62}$"))
    and (.run_id | type) == "string"
    and (.run_id | test("^[a-z0-9][a-z0-9-]{0,17}$"))
    and (.git_sha | type) == "string"
    and (.git_sha | test("^[0-9a-f]{40}$"))
    and .image_tag == (.git_sha + "-" + .run_id + "-" + .acquisition_id)
    and .registry == (.region + "-docker.pkg.dev/" + .project_id + "/" + .artifact_registry_repository)
    and (.source_bundle_sha256 | type) == "string"
    and (.source_bundle_sha256 | test("^[0-9a-f]{64}$"))
    and (.acquisition_id | type) == "string"
    and (.acquisition_id | test("^[0-9a-f]{32}$"))
    and (.cloud_build_id | type) == "string"
    and (.cloud_build_id | test("^[A-Za-z0-9-]{1,128}$"))
    and (.source_prefix | type) == "string"
    and (.source_prefix | test("^gs://[A-Za-z0-9._-]+/source/axiom-gcp-operator-[a-z0-9][a-z0-9-]{0,17}$"))
    and (.source_prefix | endswith("/source/axiom-gcp-operator-" + $candidate.run_id))
    and .reservation_uri == (.source_prefix + "/candidate-reservation.json")
    and (.source_object_uri | type) == "string"
    and ($candidate.source_object_uri | startswith($candidate.source_prefix + "/"))
    and $candidate.sift_image == ($candidate.registry + "/sift@" + ($candidate.sift_image | split("@")[-1]))
    and $candidate.rig_image == ($candidate.registry + "/rig@" + ($candidate.rig_image | split("@")[-1]))
    and (.sift_image | test("@sha256:[0-9a-f]{64}$"))
    and (.rig_image | test("@sha256:[0-9a-f]{64}$"))
    and $candidate.acceptance_runner_image == ($candidate.registry + "/sift-acceptance-runner@" + ($candidate.acceptance_runner_image | split("@")[-1]))
    and (.acceptance_runner_image | test("@sha256:[0-9a-f]{64}$"))
    and (.completed_at | type) == "string"
    and (.completed_at | length) > 0
    and (.file_sha256 | type) == "object"
    and (.file_sha256 | keys) == [
      "candidate-gate.json",
      "candidate-gate.log",
      "candidate-reservation.json",
      "candidate-source.json",
      "candidate-submit-intent.json",
      "cloud-build-final.json",
      "cloud-build-source-binding.json",
      "cloud-build-source-object.json",
      "cloud-build-submit.json",
      "images.json",
      "preexisting-artifact-registry.json",
      "preexisting-cloud-build-source-bucket.json",
      "preexisting-cloud-build-source-objects.txt",
      "preexisting-rig-images.json",
      "preexisting-sift-acceptance-runner-images.json",
      "preexisting-sift-images.json",
      "source-prefix.json"
    ]
    and all(.file_sha256[]; type == "string" and test("^[0-9a-f]{64}$"))
  ' "$receipt" >/dev/null || return 1

  while IFS= read -r name; do
    [[ -f "$candidate_dir/$name" && ! -L "$candidate_dir/$name" ]] || return 1
    expected="$(jq -er --arg name "$name" '.file_sha256[$name]' "$receipt")" \
      || return 1
    actual="$(sift_candidate_file_sha256 "$candidate_dir/$name")" || return 1
    [[ "$actual" == "$expected" ]] || return 1
  done < <(sift_candidate_required_files)

  verify_sift_candidate_reservation \
    "$candidate_dir/candidate-reservation.json" || return 1
  verify_sift_candidate_submit_intent \
    "$candidate_dir/candidate-submit-intent.json" \
    "$candidate_dir/candidate-reservation.json" || return 1
  verify_sift_candidate_build_receipt \
    "$receipt" "$candidate_dir/cloud-build-final.json" || return 1

  jq -ne \
    --slurpfile candidate "$receipt" \
    --slurpfile source "$candidate_dir/candidate-source.json" \
    --slurpfile gate "$candidate_dir/candidate-gate.json" \
    --slurpfile reservation "$candidate_dir/candidate-reservation.json" \
    --slurpfile submit_intent "$candidate_dir/candidate-submit-intent.json" \
    --slurpfile prefix "$candidate_dir/source-prefix.json" \
    --slurpfile submit "$candidate_dir/cloud-build-submit.json" \
    --slurpfile object "$candidate_dir/cloud-build-source-object.json" \
    --slurpfile binding "$candidate_dir/cloud-build-source-binding.json" \
    --slurpfile images "$candidate_dir/images.json" \
    --slurpfile artifact_registry "$candidate_dir/preexisting-artifact-registry.json" \
    --slurpfile source_bucket "$candidate_dir/preexisting-cloud-build-source-bucket.json" \
    --slurpfile sift_inventory "$candidate_dir/preexisting-sift-images.json" \
    --slurpfile rig_inventory "$candidate_dir/preexisting-rig-images.json" \
    --slurpfile runner_inventory "$candidate_dir/preexisting-sift-acceptance-runner-images.json" '
      ($candidate[0]) as $c
      | ($source[0] | type) == "object"
      and ($source[0] | keys) == ["git_sha","source_archive","source_bundle_bytes","source_bundle_sha256"]
      and $source[0].git_sha == $c.git_sha
      and $source[0].source_bundle_sha256 == $c.source_bundle_sha256
      and $source[0].source_archive == ("git-archive:" + $c.git_sha)
      and ($source[0].source_bundle_bytes | type) == "number"
      and $source[0].source_bundle_bytes > 0
      and ($gate[0] | type) == "object"
      and ($gate[0] | keys) == ["completed_at","entrypoint","git_sha","schema","source_bundle_sha256","status"]
      and $gate[0].schema == "axiom.gcp.sift.candidate-gate.v1"
      and $gate[0].git_sha == $c.git_sha
      and $gate[0].source_bundle_sha256 == $c.source_bundle_sha256
      and $gate[0].entrypoint == "apps/sift/test.sh --candidate"
      and ($gate[0].completed_at | type) == "string"
      and ($gate[0].completed_at | length) > 0
      and $gate[0].status == "passed"
      and $reservation[0].schema == "axiom.gcp.sift.candidate-reservation.v1"
      and $reservation[0].project_id == $c.project_id
      and $reservation[0].region == $c.region
      and $reservation[0].run_id == $c.run_id
      and $reservation[0].git_sha == $c.git_sha
      and $reservation[0].source_bundle_sha256 == $c.source_bundle_sha256
      and $reservation[0].registry == $c.registry
      and $reservation[0].image_tag == $c.image_tag
      and $reservation[0].source_prefix == $c.source_prefix
      and $reservation[0].reservation_uri == $c.reservation_uri
      and $reservation[0].acquisition_id == $c.acquisition_id
      and $reservation[0].preexisting_images.sift == $sift_inventory[0]
      and $reservation[0].preexisting_images.rig == $rig_inventory[0]
      and $reservation[0].preexisting_images.sift_acceptance_runner == $runner_inventory[0]
      and $submit_intent[0].schema == "axiom.gcp.sift.candidate-submit-intent.v1"
      and $submit_intent[0].acquisition_id == $c.acquisition_id
      and $submit_intent[0].source_prefix == $c.source_prefix
      and ($prefix[0] | type) == "object"
      and ($prefix[0] | keys) == ["bucket","prefix","project_id","run_id","schema"]
      and $prefix[0].schema == "axiom.gcp.operator.source-prefix.v1"
      and $prefix[0].project_id == $c.project_id
      and $prefix[0].run_id == $c.run_id
      and $prefix[0].prefix == $c.source_prefix
      and $prefix[0].bucket == ($c.source_prefix | sub("^gs://"; "") | split("/")[0])
      and ($submit[0] | type) == "object"
      and $submit[0].id == $c.cloud_build_id
      and $submit[0].substitutions._GIT_SHA == $c.git_sha
      and $submit[0].substitutions._RUN_ID == $c.run_id
      and $submit[0].substitutions._SOURCE_BUNDLE_SHA256 == $c.source_bundle_sha256
      and $submit[0].substitutions._REGISTRY == $c.registry
      and $submit[0].substitutions._TAG == $c.image_tag
      and $submit[0].substitutions._CANDIDATE_ACQUISITION_ID == $c.acquisition_id
      and $submit[0].source.storageSource.bucket == ($c.source_object_uri
        | sub("^gs://"; "") | split("/")[0])
      and $submit[0].source.storageSource.object == ($c.source_object_uri
        | sub("^gs://[^/]+/"; ""))
      and $submit[0].source.storageSource.generation == $object[0].generation
      and ($submit[0].sourceProvenance.resolvedStorageSource == null
        or $submit[0].sourceProvenance.resolvedStorageSource == $submit[0].source.storageSource)
      and ($object[0] | type) == "object"
      and $object[0].bucket == ($c.source_object_uri
        | sub("^gs://"; "") | split("/")[0])
      and $object[0].name == ($c.source_object_uri | sub("^gs://[^/]+/"; ""))
      and ($binding[0] | type) == "object"
      and ($binding[0] | keys) == ["build_id","git_sha","source_bundle_sha256","source_uri","staged_source_sha256"]
      and $binding[0].build_id == $c.cloud_build_id
      and $binding[0].git_sha == $c.git_sha
      and $binding[0].source_uri == $c.source_object_uri
      and $binding[0].source_bundle_sha256 == $c.source_bundle_sha256
      and $binding[0].staged_source_sha256 == $c.source_bundle_sha256
      and ($images[0] | type) == "object"
      and ($images[0] | keys) == ["acceptance_runner","rig","sift"]
      and $images[0].sift == $c.sift_image
      and $images[0].rig == $c.rig_image
      and $images[0].acceptance_runner == $c.acceptance_runner_image
      and ($artifact_registry[0] | type) == "object"
      and ($source_bucket[0] | type) == "object"
      and ($sift_inventory[0] | type) == "array"
      and ($rig_inventory[0] | type) == "array"
      and ($runner_inventory[0] | type) == "array"
    ' >/dev/null || return 1
}

copy_sift_candidate_evidence() {
  local candidate_dir="$1"
  local evidence_dir="$2"
  local name temporary
  verify_sift_candidate_directory "$candidate_dir" || return 1
  mkdir -p "$evidence_dir" || return 1
  while IFS= read -r name; do
    temporary="$(mktemp "$evidence_dir/.candidate-copy.XXXXXX")" || return 1
    if ! cp "$candidate_dir/$name" "$temporary" \
        || ! chmod 0600 "$temporary" \
        || ! mv "$temporary" "$evidence_dir/$name"; then
      rm -f "$temporary"
      return 1
    fi
  done < <(sift_candidate_required_files)
  temporary="$(mktemp "$evidence_dir/.candidate-copy.XXXXXX")" || return 1
  if ! cp "$candidate_dir/candidate.json" "$temporary" \
      || ! chmod 0600 "$temporary" \
      || ! mv "$temporary" "$evidence_dir/candidate.json"; then
    rm -f "$temporary"
    return 1
  fi
}
