#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ACCEPTANCE_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
source "$ACCEPTANCE_ROOT/scripts/sift-candidate.sh"

test_root="$(mktemp -d "${TMPDIR:-/tmp}/sift-candidate-receipt.XXXXXX")"
candidate_dir="$test_root/candidate"
copied="$test_root/copied"
cleanup_test() {
  find "$test_root" -depth -delete >/dev/null 2>&1 || true
}
trap cleanup_test EXIT INT TERM
mkdir -p "$candidate_dir"

git_sha="0123456789abcdef0123456789abcdef01234567"
source_sha="aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
sift_digest="bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
rig_digest="cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
runner_digest="dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"
registry="asia-east1-docker.pkg.dev/axiom-test/courier"
source_prefix="gs://axiom-test_cloudbuild/source/axiom-gcp-operator-receipt1"
acquisition_id="11111111111111111111111111111111"
image_tag="${git_sha}-receipt1-${acquisition_id}"
reservation_uri="$source_prefix/candidate-reservation.json"

jq -n --arg git_sha "$git_sha" --arg source_bundle_sha256 "$source_sha" \
  '{git_sha:$git_sha,source_archive:("git-archive:" + $git_sha),source_bundle_sha256:$source_bundle_sha256,source_bundle_bytes:123}' \
  > "$candidate_dir/candidate-source.json"
jq -n --arg git_sha "$git_sha" --arg source_bundle_sha256 "$source_sha" \
  '{schema:"axiom.gcp.sift.candidate-gate.v1",git_sha:$git_sha,source_bundle_sha256:$source_bundle_sha256,entrypoint:"apps/sift/test.sh --candidate",completed_at:"2026-09-03T00:00:00Z",status:"passed"}' \
  > "$candidate_dir/candidate-gate.json"
printf 'candidate passed\n' > "$candidate_dir/candidate-gate.log"
jq -n --arg prefix "$source_prefix" \
  '{schema:"axiom.gcp.operator.source-prefix.v1",project_id:"axiom-test",run_id:"receipt1",bucket:"axiom-test_cloudbuild",prefix:$prefix}' \
  > "$candidate_dir/source-prefix.json"
jq -n --arg git_sha "$git_sha" --arg source_sha "$source_sha" \
  --arg registry "$registry" --arg image_tag "$image_tag" \
  --arg acquisition_id "$acquisition_id" '
  {
    id:"build-1",
    source:{storageSource:{bucket:"axiom-test_cloudbuild",object:"source/axiom-gcp-operator-receipt1/source.tgz"}},
    substitutions:{
      _GIT_SHA:$git_sha,
      _RUN_ID:"receipt1",
      _SOURCE_BUNDLE_SHA256:$source_sha,
      _REGISTRY:$registry,
      _TAG:$image_tag,
      _CANDIDATE_ACQUISITION_ID:$acquisition_id
    }
  }
  ' \
  > "$candidate_dir/cloud-build-submit.json"
jq -n \
  --arg git_sha "$git_sha" \
  --arg source_sha "$source_sha" \
  --arg sift_name "$registry/sift:${image_tag}" \
  --arg sift_digest "sha256:$sift_digest" \
  --arg rig_name "$registry/rig:${image_tag}" \
  --arg rig_digest "sha256:$rig_digest" \
  --arg runner_name "$registry/sift-acceptance-runner:${image_tag}" \
  --arg runner_digest "sha256:$runner_digest" \
  --arg image_tag "$image_tag" \
  --arg acquisition_id "$acquisition_id" '
    {
      id:"build-1",
      status:"SUCCESS",
      source:{storageSource:{bucket:"axiom-test_cloudbuild",object:"source/axiom-gcp-operator-receipt1/source.tgz"}},
      substitutions:{
        _GIT_SHA:$git_sha,
        _RUN_ID:"receipt1",
        _SOURCE_BUNDLE_SHA256:$source_sha,
        _REGISTRY:"asia-east1-docker.pkg.dev/axiom-test/courier",
        _TAG:$image_tag,
        _CANDIDATE_ACQUISITION_ID:$acquisition_id
      },
      tags:[
        "sift-mvp",
        "axiom-run-receipt1",
        ("axiom-source-" + $source_sha),
        ("axiom-acquisition-" + $acquisition_id)
      ],
      results:{images:[
        {name:$sift_name,digest:$sift_digest},
        {name:$rig_name,digest:$rig_digest},
        {name:$runner_name,digest:$runner_digest}
      ]}
    }
  ' > "$candidate_dir/cloud-build-final.json"
jq -n '{bucket:"axiom-test_cloudbuild",name:"source/axiom-gcp-operator-receipt1/source.tgz"}' \
  > "$candidate_dir/cloud-build-source-object.json"
jq -n --arg git_sha "$git_sha" --arg source_bundle_sha256 "$source_sha" \
  --arg source_uri "$source_prefix/source.tgz" \
  '{build_id:"build-1",git_sha:$git_sha,source_uri:$source_uri,source_bundle_sha256:$source_bundle_sha256,staged_source_sha256:$source_bundle_sha256}' \
  > "$candidate_dir/cloud-build-source-binding.json"
jq -n \
  --arg sift "$registry/sift@sha256:$sift_digest" \
  --arg rig "$registry/rig@sha256:$rig_digest" \
  --arg acceptance_runner "$registry/sift-acceptance-runner@sha256:$runner_digest" \
  '{sift:$sift,rig:$rig,acceptance_runner:$acceptance_runner}' \
  > "$candidate_dir/images.json"
jq -n '{}' > "$candidate_dir/preexisting-artifact-registry.json"
jq -n '{}' > "$candidate_dir/preexisting-cloud-build-source-bucket.json"
: > "$candidate_dir/preexisting-cloud-build-source-objects.txt"
printf '[]\n' > "$candidate_dir/preexisting-sift-images.json"
printf '[]\n' > "$candidate_dir/preexisting-rig-images.json"
printf '[]\n' > "$candidate_dir/preexisting-sift-acceptance-runner-images.json"
jq -n \
  --arg git_sha "$git_sha" \
  --arg source_bundle_sha256 "$source_sha" \
  --arg acquisition_id "$acquisition_id" \
  --arg registry "$registry" \
  --arg image_tag "$image_tag" \
  --arg source_prefix "$source_prefix" \
  --arg reservation_uri "$reservation_uri" '
    {
      schema:"axiom.gcp.sift.candidate-reservation.v1",
      project_id:"axiom-test",
      region:"asia-east1",
      artifact_registry_repository:"courier",
      run_id:"receipt1",
      git_sha:$git_sha,
      source_bundle_sha256:$source_bundle_sha256,
      acquisition_id:$acquisition_id,
      registry:$registry,
      image_tag:$image_tag,
      source_prefix:$source_prefix,
      reservation_uri:$reservation_uri,
      created_at:"2026-09-03T00:00:00Z",
      preexisting_images:{sift:[],rig:[],sift_acceptance_runner:[]}
    }
  ' > "$candidate_dir/candidate-reservation.json"
jq -n \
  --arg git_sha "$git_sha" \
  --arg source_bundle_sha256 "$source_sha" \
  --arg acquisition_id "$acquisition_id" \
  --arg registry "$registry" \
  --arg image_tag "$image_tag" \
  --arg source_prefix "$source_prefix" '
    {
      schema:"axiom.gcp.sift.candidate-submit-intent.v1",
      project_id:"axiom-test",
      region:"asia-east1",
      run_id:"receipt1",
      git_sha:$git_sha,
      source_bundle_sha256:$source_bundle_sha256,
      acquisition_id:$acquisition_id,
      registry:$registry,
      image_tag:$image_tag,
      source_prefix:$source_prefix,
      submitted_at:"2026-09-03T00:00:01Z"
    }
  ' > "$candidate_dir/candidate-submit-intent.json"

file_hashes='{}'
while IFS= read -r name; do
  digest="$(sift_candidate_file_sha256 "$candidate_dir/$name")"
  file_hashes="$(jq -c --arg name "$name" --arg digest "$digest" \
    '. + {($name):$digest}' <<<"$file_hashes")"
done < <(sift_candidate_required_files)
jq -n \
  --arg git_sha "$git_sha" \
  --arg image_tag "$image_tag" \
  --arg registry "$registry" \
  --arg source_prefix "$source_prefix" \
  --arg acquisition_id "$acquisition_id" \
  --arg reservation_uri "$reservation_uri" \
  --arg source_bundle_sha256 "$source_sha" \
  --arg source_object_uri "$source_prefix/source.tgz" \
  --arg sift_image "$registry/sift@sha256:$sift_digest" \
  --arg rig_image "$registry/rig@sha256:$rig_digest" \
  --arg acceptance_runner_image "$registry/sift-acceptance-runner@sha256:$runner_digest" \
  --argjson file_sha256 "$file_hashes" '
    {
      schema:"axiom.gcp.sift.candidate.v1",
      project_id:"axiom-test",
      region:"asia-east1",
      artifact_registry_repository:"courier",
      registry:$registry,
      run_id:"receipt1",
      git_sha:$git_sha,
      image_tag:$image_tag,
      acquisition_id:$acquisition_id,
      reservation_uri:$reservation_uri,
      source_prefix:$source_prefix,
      source_bundle_sha256:$source_bundle_sha256,
      source_object_uri:$source_object_uri,
      cloud_build_id:"build-1",
      sift_image:$sift_image,
      rig_image:$rig_image,
      acceptance_runner_image:$acceptance_runner_image,
      completed_at:"2026-09-03T00:00:00Z",
      file_sha256:$file_sha256
    }
  ' > "$candidate_dir/candidate.json"

verify_sift_candidate_directory "$candidate_dir"
if [[ -n "${SIFT_CANDIDATE_FIXTURE_OUT:-}" ]]; then
  [[ "$SIFT_CANDIDATE_FIXTURE_OUT" == /* \
    && ! -e "$SIFT_CANDIDATE_FIXTURE_OUT" \
    && ! -L "$SIFT_CANDIDATE_FIXTURE_OUT" ]] || {
    echo "SIFT_CANDIDATE_FIXTURE_OUT must be a new absolute path" >&2
    exit 1
  }
  mkdir -p "$(dirname "$SIFT_CANDIDATE_FIXTURE_OUT")"
  copy_sift_candidate_evidence "$candidate_dir" "$SIFT_CANDIDATE_FIXTURE_OUT"
fi
copy_sift_candidate_evidence "$candidate_dir" "$copied"
verify_sift_candidate_directory "$copied"

printf 'tampered\n' >> "$candidate_dir/candidate-gate.log"
if verify_sift_candidate_directory "$candidate_dir"; then
  echo "candidate validation accepted a changed component file" >&2
  exit 1
fi
printf 'candidate passed\n' > "$candidate_dir/candidate-gate.log"
gate_log_digest="$(sift_candidate_file_sha256 "$candidate_dir/candidate-gate.log")"
jq --arg digest "$gate_log_digest" \
  '.file_sha256["candidate-gate.log"] = $digest' \
  "$candidate_dir/candidate.json" > "$candidate_dir/candidate-updated.json"
mv "$candidate_dir/candidate-updated.json" "$candidate_dir/candidate.json"

jq '.status = "FAILURE"' "$candidate_dir/cloud-build-final.json" \
  > "$candidate_dir/cloud-build-final-bad.json"
mv "$candidate_dir/cloud-build-final-bad.json" \
  "$candidate_dir/cloud-build-final.json"
bad_build_digest="$(sift_candidate_file_sha256 \
  "$candidate_dir/cloud-build-final.json")"
jq --arg digest "$bad_build_digest" \
  '.file_sha256["cloud-build-final.json"] = $digest' \
  "$candidate_dir/candidate.json" > "$candidate_dir/candidate-updated.json"
mv "$candidate_dir/candidate-updated.json" "$candidate_dir/candidate.json"
if verify_sift_candidate_directory "$candidate_dir"; then
  echo "candidate validation accepted a semantically failed Cloud Build" >&2
  exit 1
fi
jq '.status = "SUCCESS"' "$candidate_dir/cloud-build-final.json" \
  > "$candidate_dir/cloud-build-final-good.json"
mv "$candidate_dir/cloud-build-final-good.json" \
  "$candidate_dir/cloud-build-final.json"
good_build_digest="$(sift_candidate_file_sha256 \
  "$candidate_dir/cloud-build-final.json")"
jq --arg digest "$good_build_digest" \
  '.file_sha256["cloud-build-final.json"] = $digest' \
  "$candidate_dir/candidate.json" > "$candidate_dir/candidate-updated.json"
mv "$candidate_dir/candidate-updated.json" "$candidate_dir/candidate.json"
verify_sift_candidate_directory "$candidate_dir"

ln -s "$candidate_dir/images.json" "$test_root/images-link.json"
mv "$candidate_dir/images.json" "$candidate_dir/images-real.json"
mv "$test_root/images-link.json" "$candidate_dir/images.json"
if verify_sift_candidate_directory "$candidate_dir"; then
  echo "candidate validation accepted a symlinked component" >&2
  exit 1
fi

echo "Sift candidate receipt E2E: ok"
