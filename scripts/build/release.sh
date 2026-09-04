#!/usr/bin/env bash
# GKE acceptance leg of build-release for keep / defer / relay / loom:
# dispatch the gke-acceptance workflow for one app at the pushed HEAD, watch
# it to a terminal state, download the evidence bundle, and report whether
# the node pool was parked. With --image the workflow deploys that
# digest-pinned prebuilt image instead of building one: the release
# candidate's root digest for keep / relay / defer (build-release step 5);
# loom has no candidate and stays acceptance-only. The workflow owns build → terraform + kustomize deploy →
# e2e verify → park; this script only drives it through gh and reads back.
#
# usage: scripts/build/release.sh <app> [--image <ref>] [--rerun] [--out <dir>]
#   app      one of: keep defer relay loom
#   --image  ghcr.io/<owner>/<app>@sha256:<64 hex>; skips the app's build job
#   --rerun  dispatch even when this sha+app already has a successful run
#   --out    evidence download directory
#            (default ${TMPDIR:-/tmp}/gke-acceptance-<run-id>)
# exit: 0 run concluded success AND the park step concluded success;
#   1 run red, or park step not success (evidence downloaded when the run
#   produced any); 2 refused — uncovered app, dirty tree, unpushed HEAD,
#   workflow not on the default branch, duplicate run without --rerun,
#   malformed --image.
# The watch takes 15-40 minutes; run it in the background from a session.
set -euo pipefail
GIT=(git -c core.fsmonitor=false)
WORKFLOW=gke-acceptance.yml
COVERED="keep defer relay loom"
ACCEPTANCE_JOB='deploy + verify on GKE'
PARK_STEP='Park node pool (belt and suspenders)'

app='' rerun=false out='' image=''
while [ $# -gt 0 ]; do
  case "$1" in
    --image) image=${2:?--image needs a digest reference}; shift 2 ;;
    --rerun) rerun=true; shift ;;
    --out) out=${2:?--out needs a directory}; shift 2 ;;
    -*) echo "refused: unknown flag $1" >&2; exit 2 ;;
    *)
      [ -z "$app" ] || { echo "refused: one app per run, got $app and $1" >&2; exit 2; }
      app=$1; shift ;;
  esac
done
[ -n "$app" ] || { echo "refused: name one app (covered: $COVERED)" >&2; exit 2; }
case " $COVERED " in
  *" $app "*) ;;
  *) echo "refused: release route not wired for $app (covered: $COVERED)" >&2; exit 2 ;;
esac
command -v jq >/dev/null || { echo "refused: jq is required" >&2; exit 2; }
[ -z "$image" ] || [[ "$image" =~ ^ghcr\.io/[^/]+/${app}@sha256:[0-9a-f]{64}$ ]] || {
  echo "refused: --image must be ghcr.io/<owner>/$app@sha256:<64 hex>, got $image" >&2
  exit 2
}

dirty=$("${GIT[@]}" status --porcelain)
[ -z "$dirty" ] || {
  echo "refused: dirty tree — the run tests git, not the working tree; commit or discard first" >&2
  printf '%s\n' "$dirty" >&2
  exit 2
}
branch=$("${GIT[@]}" branch --show-current)
[ -n "$branch" ] || { echo "refused: detached HEAD; dispatch needs a pushed branch" >&2; exit 2; }
sha=$("${GIT[@]}" rev-parse HEAD)
remote_sha=$("${GIT[@]}" ls-remote origin "refs/heads/$branch" | cut -f1)
[ "$remote_sha" = "$sha" ] || {
  echo "refused: HEAD $sha is not pushed as origin/$branch (remote: ${remote_sha:-absent})" >&2
  exit 2
}

# gh resolves workflows from the default branch only, so a workflow file
# that has not landed on main cannot be dispatched from any branch.
gh workflow view "$WORKFLOW" --yaml >/dev/null 2>&1 || {
  echo "refused: $WORKFLOW is not on the default branch yet — land it on main before dispatching" >&2
  exit 2
}

# gh run list does not expose workflow_dispatch inputs; the workflow's
# run-name carries the app list (and the prebuilt image, when one was
# given), so displayTitle is the only place to tell a keep run from a loom
# run — or a candidate-image run from a test-image run — at the same sha.
prior_json=$(gh run list --workflow "$WORKFLOW" -L 50 \
  --json databaseId,headSha,event,conclusion,displayTitle,url)
if [ "$rerun" = false ]; then
  dup=$(jq -r --arg sha "$sha" --arg app " $app " --arg image "$image" \
    '[.[] | select(.headSha == $sha and .conclusion == "success"
                   and (.displayTitle | contains($app))
                   and (if $image == "" then (.displayTitle | contains(" image=") | not)
                        else (.displayTitle | endswith(" image=" + $image)) end))]
     | .[0].url // empty' \
    <<<"$prior_json")
  [ -z "$dup" ] || {
    echo "refused: $sha already has a successful $app run: $dup — pass --rerun and name what changed" >&2
    exit 2
  }
fi
before_ids=$(jq -c --arg sha "$sha" \
  '[.[] | select(.headSha == $sha and .event == "workflow_dispatch") | .databaseId]' \
  <<<"$prior_json")

gh workflow run "$WORKFLOW" --ref "$branch" -f apps="$app" -f ref="$sha" -f image="$image"

# gh workflow run prints no run id; the new run is the one for this sha
# that was not in the list before dispatch.
run_id=''
for _ in $(seq 1 18); do
  sleep 5
  run_id=$(gh run list --workflow "$WORKFLOW" --branch "$branch" -L 20 \
      --json databaseId,headSha,event,createdAt \
    | jq -r --arg sha "$sha" --argjson before "$before_ids" \
        '[.[] | select(.headSha == $sha and .event == "workflow_dispatch"
                       and ((.databaseId as $id | $before | index($id)) == null))]
         | sort_by(.createdAt) | last | .databaseId // empty')
  [ -n "$run_id" ] && break
done
[ -n "$run_id" ] || {
  echo "error: dispatched, but no new run for $sha appeared within 90s; inspect: gh run list --workflow $WORKFLOW" >&2
  exit 1
}
url=$(gh run view "$run_id" --json url --jq .url)
echo "run: $url"

# The gate is the read-back below, not the watch's exit code.
gh run watch "$run_id" --interval 30 --exit-status || true

view=$(gh run view "$run_id" --json conclusion,attempt,jobs)
conclusion=$(jq -r '.conclusion' <<<"$view")
attempt=$(jq -r '.attempt' <<<"$view")
acceptance=$(jq -r --arg job "$ACCEPTANCE_JOB" \
  '[.jobs[] | select(.name == $job) | .conclusion] | .[0] // "absent"' <<<"$view")
park=$(jq -r --arg job "$ACCEPTANCE_JOB" --arg step "$PARK_STEP" \
  '[.jobs[] | select(.name == $job) | .steps[] | select(.name == $step) | .conclusion]
   | .[0] // "absent"' <<<"$view")

out=${out:-${TMPDIR:-/tmp}/gke-acceptance-$run_id}
mkdir -p "$out"
if gh run download "$run_id" -n "gke-acceptance-evidence-${run_id}-${attempt}" -D "$out" 2>/dev/null; then
  evidence=$out
else
  evidence="none (the run uploaded no evidence artifact)"
fi

echo "conclusion: $conclusion"
echo "acceptance job: $acceptance"
echo "park step: $park"
echo "evidence: $evidence"

if [ "$conclusion" = success ] && [ "$park" = success ]; then
  exit 0
fi
# A skipped acceptance job never woke the pool (a selected build failed
# upstream), so an absent park step is not a leak there.
if [ "$acceptance" = skipped ]; then
  echo "error: acceptance job skipped — a selected image build failed; the pool was never woken" >&2
  exit 1
fi
if [ "$park" != success ]; then
  echo "error: park step concluded '$park' — the node pool may still be running; run acceptance/gke-harness/scripts/park.sh" >&2
fi
exit 1
