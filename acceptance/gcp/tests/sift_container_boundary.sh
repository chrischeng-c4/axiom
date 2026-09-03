#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ACCEPTANCE_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
source "$ACCEPTANCE_ROOT/scripts/sift-candidate.sh"
source "$ACCEPTANCE_ROOT/scripts/sift-container-boundary.sh"

test_root="$(mktemp -d "${TMPDIR:-/tmp}/sift-container-boundary.XXXXXX")"
candidate_dir="$test_root/candidate"
fake_bin="$test_root/bin"
fake_gcloud_config="$test_root/gcloud-source"
docker_log="$test_root/docker.log"
cleanup_test() {
  if [[ "${SIFT_KEEP_TEST_TMP:-0}" == "1" ]]; then
    echo "preserved test root: $test_root" >&2
    return
  fi
  find "$test_root" -depth -delete >/dev/null 2>&1 || true
}
trap cleanup_test EXIT INT TERM
mkdir -p "$candidate_dir" "$fake_bin" "$fake_gcloud_config"

git_sha="0123456789abcdef0123456789abcdef01234567"
source_sha="aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
registry="asia-east1-docker.pkg.dev/axiom-test/courier"
source_prefix="gs://axiom-test_cloudbuild/source/axiom-gcp-operator-boundary1"
acquisition_id="11111111111111111111111111111111"
image_tag="${git_sha}-boundary1-${acquisition_id}"
reservation_uri="$source_prefix/candidate-reservation.json"
sift_image="$registry/sift@sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
rig_image="$registry/rig@sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
controller_image="$registry/sift-acceptance-runner@sha256:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"

jq -n --arg git_sha "$git_sha" --arg source_bundle_sha256 "$source_sha" \
  '{git_sha:$git_sha,source_archive:("git-archive:" + $git_sha),source_bundle_sha256:$source_bundle_sha256,source_bundle_bytes:123}' \
  > "$candidate_dir/candidate-source.json"
jq -n --arg git_sha "$git_sha" --arg source_bundle_sha256 "$source_sha" \
  '{schema:"axiom.gcp.sift.candidate-gate.v1",git_sha:$git_sha,source_bundle_sha256:$source_bundle_sha256,entrypoint:"apps/sift/test.sh --candidate",completed_at:"2026-09-03T00:00:00Z",status:"passed"}' \
  > "$candidate_dir/candidate-gate.json"
printf 'candidate passed\n' > "$candidate_dir/candidate-gate.log"
jq -n --arg prefix "$source_prefix" \
  '{schema:"axiom.gcp.operator.source-prefix.v1",project_id:"axiom-test",run_id:"boundary1",bucket:"axiom-test_cloudbuild",prefix:$prefix}' \
  > "$candidate_dir/source-prefix.json"
jq -n --arg git_sha "$git_sha" --arg source_sha "$source_sha" \
  --arg registry "$registry" --arg image_tag "$image_tag" \
  --arg acquisition_id "$acquisition_id" '
  {
    id:"build-1",
    source:{storageSource:{bucket:"axiom-test_cloudbuild",object:"source/axiom-gcp-operator-boundary1/source.tgz"}},
    substitutions:{
      _GIT_SHA:$git_sha,
      _RUN_ID:"boundary1",
      _SOURCE_BUNDLE_SHA256:$source_sha,
      _REGISTRY:$registry,
      _TAG:$image_tag,
      _CANDIDATE_ACQUISITION_ID:$acquisition_id
    }
  }
  ' > "$candidate_dir/cloud-build-submit.json"
jq -n \
  --arg git_sha "$git_sha" --arg source_sha "$source_sha" \
  --arg sift_name "$registry/sift:${image_tag}" \
  --arg sift_digest "${sift_image##*@}" \
  --arg rig_name "$registry/rig:${image_tag}" \
  --arg rig_digest "${rig_image##*@}" \
  --arg runner_name "$registry/sift-acceptance-runner:${image_tag}" \
  --arg runner_digest "${controller_image##*@}" \
  --arg image_tag "$image_tag" \
  --arg acquisition_id "$acquisition_id" '
    {
      id:"build-1",status:"SUCCESS",
      source:{storageSource:{bucket:"axiom-test_cloudbuild",object:"source/axiom-gcp-operator-boundary1/source.tgz"}},
      substitutions:{
        _GIT_SHA:$git_sha,
        _RUN_ID:"boundary1",
        _SOURCE_BUNDLE_SHA256:$source_sha,
        _REGISTRY:"asia-east1-docker.pkg.dev/axiom-test/courier",
        _TAG:$image_tag,
        _CANDIDATE_ACQUISITION_ID:$acquisition_id
      },
      tags:[
        "sift-mvp",
        "axiom-run-boundary1",
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
jq -n '{bucket:"axiom-test_cloudbuild",name:"source/axiom-gcp-operator-boundary1/source.tgz"}' \
  > "$candidate_dir/cloud-build-source-object.json"
jq -n --arg git_sha "$git_sha" --arg source_bundle_sha256 "$source_sha" \
  --arg source_uri "$source_prefix/source.tgz" \
  '{build_id:"build-1",git_sha:$git_sha,source_uri:$source_uri,source_bundle_sha256:$source_bundle_sha256,staged_source_sha256:$source_bundle_sha256}' \
  > "$candidate_dir/cloud-build-source-binding.json"
jq -n --arg sift "$sift_image" --arg rig "$rig_image" \
  --arg acceptance_runner "$controller_image" \
  '{sift:$sift,rig:$rig,acceptance_runner:$acceptance_runner}' \
  > "$candidate_dir/images.json"
jq -n '{}' > "$candidate_dir/preexisting-artifact-registry.json"
jq -n '{}' > "$candidate_dir/preexisting-cloud-build-source-bucket.json"
: > "$candidate_dir/preexisting-cloud-build-source-objects.txt"
for name in sift rig sift-acceptance-runner; do
  printf '[]\n' > "$candidate_dir/preexisting-${name}-images.json"
done
jq -n \
  --arg git_sha "$git_sha" --arg source_sha "$source_sha" \
  --arg acquisition_id "$acquisition_id" --arg registry "$registry" \
  --arg image_tag "$image_tag" --arg source_prefix "$source_prefix" \
  --arg reservation_uri "$reservation_uri" '
    {
      schema:"axiom.gcp.sift.candidate-reservation.v1",
      project_id:"axiom-test",
      region:"asia-east1",
      artifact_registry_repository:"courier",
      run_id:"boundary1",
      git_sha:$git_sha,
      source_bundle_sha256:$source_sha,
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
  --arg git_sha "$git_sha" --arg source_sha "$source_sha" \
  --arg acquisition_id "$acquisition_id" --arg registry "$registry" \
  --arg image_tag "$image_tag" --arg source_prefix "$source_prefix" '
    {
      schema:"axiom.gcp.sift.candidate-submit-intent.v1",
      project_id:"axiom-test",
      region:"asia-east1",
      run_id:"boundary1",
      git_sha:$git_sha,
      source_bundle_sha256:$source_sha,
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
  --arg git_sha "$git_sha" --arg registry "$registry" \
  --arg image_tag "$image_tag" \
  --arg source_prefix "$source_prefix" --arg source_bundle_sha256 "$source_sha" \
  --arg acquisition_id "$acquisition_id" --arg reservation_uri "$reservation_uri" \
  --arg sift_image "$sift_image" --arg rig_image "$rig_image" \
  --arg acceptance_runner_image "$controller_image" \
  --argjson file_sha256 "$file_hashes" '
    {
      schema:"axiom.gcp.sift.candidate.v1",project_id:"axiom-test",region:"asia-east1",
      artifact_registry_repository:"courier",registry:$registry,run_id:"boundary1",
      git_sha:$git_sha,image_tag:$image_tag,source_prefix:$source_prefix,
      acquisition_id:$acquisition_id,reservation_uri:$reservation_uri,
      source_bundle_sha256:$source_bundle_sha256,source_object_uri:($source_prefix + "/source.tgz"),
      cloud_build_id:"build-1",sift_image:$sift_image,rig_image:$rig_image,
      acceptance_runner_image:$acceptance_runner_image,completed_at:"2026-09-03T00:00:00Z",
      file_sha256:$file_sha256
    }
  ' > "$candidate_dir/candidate.json"
verify_sift_candidate_directory "$candidate_dir"

cat > "$fake_bin/gcloud" <<'EOF'
#!/usr/bin/env bash
if [[ "${1:-}" == "info" ]]; then
  printf '%s\n' "${SIFT_FAKE_GCLOUD_CONFIG:?}"
  exit 0
fi
if [[ "${1:-}" == "auth" && "${2:-}" == "print-access-token" ]]; then
  printf 'test-token\n'
  exit 0
fi
if [[ "${1:-}" == "services" && "${2:-}" == "list" ]]; then
  exit 0
fi
exit 2
EOF
cat > "$fake_bin/docker" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
command="${1:-}"
shift || true
printf '%s\t%s\n' "$command" "$*" >> "${SIFT_FAKE_DOCKER_LOG:?}"
run_id="aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
cleanup_id="bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
case "$command" in
  create)
    count_file="${SIFT_FAKE_DOCKER_LOG}.creates"
    count=0
    [[ ! -f "$count_file" ]] || count="$(<"$count_file")"
    count=$((count + 1))
    printf '%s\n' "$count" > "$count_file"
    if [[ "$count" == "1" ]]; then
      printf '%s\n' "$run_id"
    else
      [[ -f "${SIFT_FAKE_CONTAINMENT:?}/container-stopped.json" ]] || exit 31
      printf '%s\n' "$cleanup_id"
    fi
    ;;
  start)
    last=""
    for argument in "$@"; do last="$argument"; done
    if [[ "$last" == "$run_id" ]]; then
      if find "${SIFT_FAKE_EVIDENCE:?}" -mindepth 1 -print -quit \
          | grep -q .; then
        echo "run evidence was not empty before the contained controller started" >&2
        exit 41
      fi
      if [[ "${SIFT_FAKE_PREFLIGHT_EXIT:-0}" == "1" ]]; then
        jq -n '
          {
            schema:"axiom.gcp.sift.contained-run-exit.v1",
            exit_code:17,
            completed:false,
            cleanup_armed:false,
            local_claim_released:true,
            finished_at:"2026-09-03T00:01:00Z"
          }
        ' > "${SIFT_FAKE_EVIDENCE}/run-exit.json"
        printf 'contained preflight failed\n'
        exit 17
      fi
      if [[ "${SIFT_FAKE_RUNNING:-0}" == "1" ]]; then
        printf '{}\n' > "${SIFT_FAKE_EVIDENCE}/acceptance-lock-intent.json"
      else
        jq -n '
          {
            schema:"axiom.gcp.sift.contained-run-exit.v1",
            exit_code:0,
            completed:true,
            cleanup_armed:true,
            local_claim_released:false,
            finished_at:"2026-09-03T00:01:00Z"
          }
        ' > "${SIFT_FAKE_EVIDENCE}/run-exit.json"
      fi
      exit "${SIFT_FAKE_RUN_STATUS:-0}"
    fi
    exit 0
    ;;
  inspect)
    if [[ "${1:-}" == "--format" ]]; then
      [[ "${SIFT_FAKE_RUNNING:-0}" == "1" ]] && printf 'true\n' || printf 'false\n'
      exit 0
    fi
    id="${1:?}"
    jq -n --arg id "$id" --arg image "${SIFT_FAKE_CONTROLLER_IMAGE:?}" \
      '[{Id:$id,Config:{Image:$image},State:{Running:false,Status:"exited",ExitCode:0,FinishedAt:"2026-09-03T00:01:00Z"}}]'
    ;;
  stop|kill|rm) ;;
  *) exit 3 ;;
esac
EOF
chmod +x "$fake_bin/gcloud" "$fake_bin/docker"

# The private containment directory must not be equal to, inside, or above a
# directory that the run container can write. Otherwise the run container can
# read or replace the cleanup nonce before the host starts cleanup.
overlap_state="$test_root/overlap-state"
overlap_evidence="$test_root/overlap-evidence"
overlap_containment="$overlap_state/containment"
overlap_claims="$test_root/overlap-claims"
overlap_log="$test_root/overlap-docker.log"
overlap_status=0
: > "$overlap_log"
PATH="$fake_bin:$PATH" \
SIFT_FAKE_GCLOUD_CONFIG="$fake_gcloud_config" \
SIFT_FAKE_DOCKER_LOG="$overlap_log" \
SIFT_FAKE_CONTAINMENT="$overlap_containment" \
SIFT_FAKE_CONTROLLER_IMAGE="$controller_image" \
SIFT_FAKE_EVIDENCE="$overlap_evidence" \
PROJECT_ID=axiom-test REGION=asia-east1 \
SIFT_CANDIDATE_DIR="$candidate_dir" \
STATE_DIR="$overlap_state" EVIDENCE_DIR="$overlap_evidence" \
CONTAINMENT_DIR="$overlap_containment" \
ACCEPTANCE_LOCAL_CLAIM_ROOT="$overlap_claims" \
  "$ACCEPTANCE_ROOT/scripts/run-sift-contained.sh" \
  > "$test_root/overlap.stdout" 2> "$test_root/overlap.stderr" \
  || overlap_status=$?
[[ "$overlap_status" != "0" ]] || {
  echo "contained run accepted a containment directory below writable state" >&2
  exit 1
}
grep -F 'state, evidence, and containment directories must not overlap' \
  "$test_root/overlap.stderr" >/dev/null
[[ ! -s "$overlap_log" \
  && ! -e "$overlap_containment/nonce/cleanup-nonce" ]] || {
  echo "contained run created a container or secret before rejecting overlap" >&2
  exit 1
}

# The immutable candidate must not also enter through a writable bind mount.
# This exact alias would make /candidate read-only but the same bytes writable
# through /claims.
candidate_alias_state="$test_root/candidate-alias-state"
candidate_alias_evidence="$test_root/candidate-alias-evidence"
candidate_alias_containment="$test_root/candidate-alias-containment"
candidate_alias_log="$test_root/candidate-alias-docker.log"
candidate_alias_status=0
: > "$candidate_alias_log"
PATH="$fake_bin:$PATH" \
SIFT_FAKE_GCLOUD_CONFIG="$fake_gcloud_config" \
SIFT_FAKE_DOCKER_LOG="$candidate_alias_log" \
SIFT_FAKE_CONTAINMENT="$candidate_alias_containment" \
SIFT_FAKE_CONTROLLER_IMAGE="$controller_image" \
SIFT_FAKE_EVIDENCE="$candidate_alias_evidence" \
PROJECT_ID=axiom-test REGION=asia-east1 \
SIFT_CANDIDATE_DIR="$candidate_dir" \
STATE_DIR="$candidate_alias_state" EVIDENCE_DIR="$candidate_alias_evidence" \
CONTAINMENT_DIR="$candidate_alias_containment" \
ACCEPTANCE_LOCAL_CLAIM_ROOT="$candidate_dir" \
  "$ACCEPTANCE_ROOT/scripts/run-sift-contained.sh" \
  > "$test_root/candidate-alias.stdout" \
  2> "$test_root/candidate-alias.stderr" \
  || candidate_alias_status=$?
[[ "$candidate_alias_status" != "0" ]] || {
  echo "contained run accepted a writable alias of the candidate" >&2
  exit 1
}
grep -F 'the candidate directory must not overlap a writable run directory' \
  "$test_root/candidate-alias.stderr" >/dev/null
[[ ! -s "$candidate_alias_log" ]] || {
  echo "contained run created a container before rejecting candidate alias" >&2
  exit 1
}

# A prospective writable child must be rejected before mkdir can modify the
# immutable candidate directory.
candidate_child="$candidate_dir/must-not-be-created"
candidate_child_evidence="$test_root/candidate-child-evidence"
candidate_child_containment="$test_root/candidate-child-containment"
candidate_child_claims="$test_root/candidate-child-claims"
candidate_child_log="$test_root/candidate-child-docker.log"
candidate_child_status=0
: > "$candidate_child_log"
PATH="$fake_bin:$PATH" \
SIFT_FAKE_GCLOUD_CONFIG="$fake_gcloud_config" \
SIFT_FAKE_DOCKER_LOG="$candidate_child_log" \
SIFT_FAKE_CONTAINMENT="$candidate_child_containment" \
SIFT_FAKE_CONTROLLER_IMAGE="$controller_image" \
SIFT_FAKE_EVIDENCE="$candidate_child_evidence" \
PROJECT_ID=axiom-test REGION=asia-east1 \
SIFT_CANDIDATE_DIR="$candidate_dir" \
STATE_DIR="$candidate_child" EVIDENCE_DIR="$candidate_child_evidence" \
CONTAINMENT_DIR="$candidate_child_containment" \
ACCEPTANCE_LOCAL_CLAIM_ROOT="$candidate_child_claims" \
  "$ACCEPTANCE_ROOT/scripts/run-sift-contained.sh" \
  > "$test_root/candidate-child.stdout" \
  2> "$test_root/candidate-child.stderr" \
  || candidate_child_status=$?
[[ "$candidate_child_status" != "0" ]] || {
  echo "contained run accepted a writable child of the candidate" >&2
  exit 1
}
grep -F 'the candidate directory must not overlap a writable run directory' \
  "$test_root/candidate-child.stderr" >/dev/null
[[ ! -e "$candidate_child" && ! -s "$candidate_child_log" ]] || {
  echo "contained run modified the candidate before rejecting its writable child" >&2
  exit 1
}

# Execute the real contained run entrypoint through its first preflight
# failure. The evidence directory starts empty. The inner process must copy
# the candidate, release its local claim, and publish cleanup_armed=false.
inner_state="$test_root/inner-state"
inner_evidence="$test_root/inner-evidence"
inner_claims="$test_root/inner-claims"
inner_ready="$test_root/inner-ready"
mkdir -p "$inner_ready"
inner_status=0
PATH="$fake_bin:$PATH" \
AXIOM_GCP_ACCEPTANCE_CONTAINER_ROLE=run \
AXIOM_GCP_ACCEPTANCE_ISOLATED_SESSION=1 \
AXIOM_GCP_ACCEPTANCE_SUPERVISOR_READY_PATH="$inner_ready/ready.txt" \
AXIOM_GCP_ACCEPTANCE_SUPERVISOR_READY_TOKEN="$source_sha" \
ACCEPTANCE_CONTAINER_HANDOFF_DIGEST="$source_sha" \
ACCEPTANCE_APPS=sift PROJECT_ID=axiom-test REGION=asia-east1 \
GKE_ZONE=asia-east1-a RUN_ID=boundary1 \
SIFT_CANDIDATE_DIR="$candidate_dir" \
STATE_DIR="$inner_state" EVIDENCE_DIR="$inner_evidence" \
ACCEPTANCE_LOCAL_CLAIM_ROOT="$inner_claims" \
  python3 -c '
import os
import sys

os.setsid()
os.execve(sys.argv[1], [sys.argv[1]], os.environ)
' "$ACCEPTANCE_ROOT/scripts/run.sh" \
  > "$test_root/inner.stdout" 2> "$test_root/inner.stderr" || inner_status=$?
[[ "$inner_status" != "0" ]] || {
  echo "contained preflight unexpectedly passed" >&2
  exit 1
}
grep -F 'required API is not already enabled' "$test_root/inner.stderr" >/dev/null
jq -e '
  .schema == "axiom.gcp.sift.contained-run-exit.v1"
  and .cleanup_armed == false
  and .local_claim_released == true
  and .completed == false
' "$inner_evidence/run-exit.json" >/dev/null
[[ ! -e "$(find "$inner_claims" -type f -print -quit)" ]] || {
  echo "contained preflight retained its local run claim" >&2
  exit 1
}

state_dir="$test_root/state"
evidence_dir="$test_root/evidence"
containment_dir="$test_root/containment"
claims_dir="$test_root/claims"
PATH="$fake_bin:$PATH" \
SIFT_FAKE_GCLOUD_CONFIG="$fake_gcloud_config" \
SIFT_FAKE_DOCKER_LOG="$docker_log" \
SIFT_FAKE_CONTAINMENT="$containment_dir" \
SIFT_FAKE_CONTROLLER_IMAGE="$controller_image" \
SIFT_FAKE_EVIDENCE="$evidence_dir" \
PROJECT_ID=axiom-test REGION=asia-east1 \
SIFT_CANDIDATE_DIR="$candidate_dir" \
STATE_DIR="$state_dir" EVIDENCE_DIR="$evidence_dir" \
CONTAINMENT_DIR="$containment_dir" ACCEPTANCE_LOCAL_CLAIM_ROOT="$claims_dir" \
  "$ACCEPTANCE_ROOT/scripts/run-sift-contained.sh"

[[ -f "$containment_dir/container-owner.json" \
  && -f "$containment_dir/container-stopped.json" ]] || {
  echo "the host did not persist the exact stopped-container receipts" >&2
  exit 1
}
first_create="$(rg -n '^create' "$docker_log" | sed -n '1p' | cut -d: -f1)"
second_create="$(rg -n '^create' "$docker_log" | sed -n '2p' | cut -d: -f1)"
inspect_line="$(rg -n '^inspect' "$docker_log" | sed -n '1p' | cut -d: -f1)"
[[ "$first_create" =~ ^[0-9]+$ && "$second_create" =~ ^[0-9]+$ \
  && "$inspect_line" =~ ^[0-9]+$ \
  && "$first_create" -lt "$inspect_line" \
  && "$inspect_line" -lt "$second_create" ]] || {
  echo "cleanup container started before the exact run container stop proof" >&2
  exit 1
}
if rg -F '/var/run/docker.sock' "$docker_log" >/dev/null; then
  echo "the contained controller received the Docker socket" >&2
  exit 1
fi
rg -F -- '--read-only' "$docker_log" >/dev/null
rg -F -- '--cap-drop=ALL' "$docker_log" >/dev/null
rg -F -- '--security-opt=no-new-privileges' "$docker_log" >/dev/null
[[ ! -e "$containment_dir/gcloud" && ! -e "$containment_dir/nonce" ]] || {
  echo "the successful wrapper retained its private credential copy" >&2
  exit 1
}
[[ ! -e "$containment_dir/cleanup-container-owner.json" ]] || {
  echo "the successful wrapper retained a stale cleanup-container receipt" >&2
  exit 1
}

# A failure before the shared Lease is armed must publish the host-side log,
# release its local claim, and skip the cloud cleanup container.
preflight_state="$test_root/preflight-state"
preflight_evidence="$test_root/preflight-evidence"
preflight_containment="$test_root/preflight-containment"
preflight_claims="$test_root/preflight-claims"
preflight_log="$test_root/preflight-docker.log"
preflight_status=0
PATH="$fake_bin:$PATH" \
SIFT_FAKE_GCLOUD_CONFIG="$fake_gcloud_config" \
SIFT_FAKE_DOCKER_LOG="$preflight_log" \
SIFT_FAKE_CONTAINMENT="$preflight_containment" \
SIFT_FAKE_CONTROLLER_IMAGE="$controller_image" \
SIFT_FAKE_EVIDENCE="$preflight_evidence" \
SIFT_FAKE_PREFLIGHT_EXIT=1 \
PROJECT_ID=axiom-test REGION=asia-east1 \
SIFT_CANDIDATE_DIR="$candidate_dir" \
STATE_DIR="$preflight_state" EVIDENCE_DIR="$preflight_evidence" \
CONTAINMENT_DIR="$preflight_containment" \
ACCEPTANCE_LOCAL_CLAIM_ROOT="$preflight_claims" \
  "$ACCEPTANCE_ROOT/scripts/run-sift-contained.sh" \
  >/dev/null 2>&1 || preflight_status=$?
[[ "$preflight_status" == "17" ]] || {
  echo "contained preflight did not preserve its run status" >&2
  exit 1
}
[[ "$(rg -c '^create' "$preflight_log")" == "1" ]] || {
  echo "contained preflight started a cloud cleanup container" >&2
  exit 1
}
jq -e '
  .cleanup_armed == false
  and .local_claim_released == true
' "$preflight_evidence/run-exit.json" >/dev/null
grep -Fx 'contained preflight failed' "$preflight_evidence/run.log" >/dev/null
[[ ! -e "$preflight_containment/gcloud" \
  && ! -e "$preflight_containment/nonce" ]] || {
  echo "contained preflight retained private credentials after a safe exit" >&2
  exit 1
}

# Cleanup authorization needs both the exact stopped container receipt and the
# nonce that the run container never received.
auth_dir="$test_root/auth"
mkdir -p "$auth_dir"
auth_nonce="eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"
auth_digest="$(sift_container_nonce_digest "$auth_nonce")"
auth_id="ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
write_sift_container_owner \
  "$auth_dir/owner.json" "$auth_id" "$controller_image" \
  /state /evidence "$auth_digest"
jq -n --arg id "$auth_id" --arg image "$controller_image" \
  '{Id:$id,Config:{Image:$image},State:{Running:false,Status:"exited",ExitCode:7,FinishedAt:"2026-09-03T00:01:00Z"}}' \
  > "$auth_dir/inspect.json"
write_sift_container_stopped_receipt \
  "$auth_dir/stopped.json" "$auth_dir/inspect.json" \
  "$auth_id" "$controller_image" "$auth_digest"
authorize_sift_container_cleanup \
  "$auth_dir/owner.json" "$auth_dir/stopped.json" "$auth_nonce" \
  "$controller_image" /state /evidence
if authorize_sift_container_cleanup \
    "$auth_dir/owner.json" "$auth_dir/stopped.json" \
    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" \
    "$controller_image" /state /evidence; then
  echo "contained cleanup accepted the wrong handoff nonce" >&2
  exit 1
fi
jq '.running = true | .status = "running"' \
  "$auth_dir/stopped.json" > "$auth_dir/running.json"
if authorize_sift_container_cleanup \
    "$auth_dir/owner.json" "$auth_dir/running.json" "$auth_nonce" \
    "$controller_image" /state /evidence; then
  echo "contained cleanup accepted a running container" >&2
  exit 1
fi

# A host observation that still says Running=true must not start the cleanup
# container, even if Docker stop itself returned success.
running_state="$test_root/running-state"
running_evidence="$test_root/running-evidence"
running_containment="$test_root/running-containment"
running_claims="$test_root/running-claims"
running_log="$test_root/running-docker.log"
running_status=0
PATH="$fake_bin:$PATH" \
SIFT_FAKE_GCLOUD_CONFIG="$fake_gcloud_config" \
SIFT_FAKE_DOCKER_LOG="$running_log" \
SIFT_FAKE_CONTAINMENT="$running_containment" \
SIFT_FAKE_CONTROLLER_IMAGE="$controller_image" \
SIFT_FAKE_EVIDENCE="$running_evidence" \
SIFT_FAKE_RUNNING=1 \
PROJECT_ID=axiom-test REGION=asia-east1 \
SIFT_CANDIDATE_DIR="$candidate_dir" \
STATE_DIR="$running_state" EVIDENCE_DIR="$running_evidence" \
CONTAINMENT_DIR="$running_containment" \
ACCEPTANCE_LOCAL_CLAIM_ROOT="$running_claims" \
  "$ACCEPTANCE_ROOT/scripts/run-sift-contained.sh" \
  >/dev/null 2>&1 || running_status=$?
[[ "$running_status" != "0" ]] || {
  echo "the wrapper accepted a run container that was still running" >&2
  exit 1
}
[[ "$(rg -c '^create' "$running_log")" == "1" ]] || {
  echo "the wrapper started cleanup without a stopped-container proof" >&2
  exit 1
}
[[ -f "$running_containment/container-owner.json" \
  && -f "$running_containment/nonce/cleanup-nonce" ]] || {
  echo "the failed wrapper did not retain bounded recovery evidence" >&2
  exit 1
}

# Recovery uses the same exact run-container receipt and private nonce. It can
# stop that container and start cleanup only after it writes the stop proof.
recovery_status=0
PATH="$fake_bin:$PATH" \
SIFT_FAKE_GCLOUD_CONFIG="$fake_gcloud_config" \
SIFT_FAKE_DOCKER_LOG="$running_log" \
SIFT_FAKE_CONTAINMENT="$running_containment" \
SIFT_FAKE_CONTROLLER_IMAGE="$controller_image" \
SIFT_FAKE_EVIDENCE="$running_evidence" \
PROJECT_ID=axiom-test REGION=asia-east1 \
SIFT_CANDIDATE_DIR="$candidate_dir" \
STATE_DIR="$running_state" EVIDENCE_DIR="$running_evidence" \
CONTAINMENT_DIR="$running_containment" \
ACCEPTANCE_LOCAL_CLAIM_ROOT="$running_claims" \
  "$ACCEPTANCE_ROOT/scripts/run-sift-contained.sh" --recover \
  >/dev/null 2>&1 || recovery_status=$?
[[ "$recovery_status" == "125" ]] || {
  echo "contained recovery did not preserve the unknown run status" >&2
  exit 1
}
[[ "$(rg -c '^create' "$running_log")" == "2" ]] || {
  echo "contained recovery did not start exactly one cleanup container" >&2
  exit 1
}
[[ ! -e "$running_containment/gcloud" \
  && ! -e "$running_containment/nonce" \
  && ! -e "$running_containment/cleanup-container-owner.json" ]] || {
  echo "successful contained recovery retained private recovery material" >&2
  exit 1
}

echo "Sift container boundary E2E: ok"
