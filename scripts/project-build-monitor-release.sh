#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF' >&2
Usage: scripts/project-build-monitor-release.sh <project> <tag>

Waits for .github/workflows/<project>-release.yml to run for <tag> when present,
then verifies the GitHub release exists and prints GH_RELEASE_URL.
EOF
}

PROJECT="${1:-}"
TAG="${2:-}"
if [[ -z "$PROJECT" || -z "$TAG" || "${3:-}" == "-h" || "${3:-}" == "--help" ]]; then
  usage
  exit 2
fi

command -v gh >/dev/null || {
  echo "error: gh CLI is required to monitor GitHub release CI" >&2
  exit 4
}

WORKFLOW=".github/workflows/${PROJECT}-release.yml"
if [[ -f "$WORKFLOW" ]]; then
  echo "Waiting for ${WORKFLOW} run for ${TAG}..."
  RUN_ID=""
  for _ in {1..30}; do
    RUN_ID="$(
      gh run list \
        --workflow "$(basename "$WORKFLOW")" \
        --event push \
        --limit 30 \
        --json databaseId,headBranch \
        --jq '.[] | select(.headBranch == "'"${TAG}"'") | .databaseId' \
        | head -n 1
    )"
    if [[ -n "$RUN_ID" ]]; then
      break
    fi
    sleep 10
  done

  if [[ -z "$RUN_ID" ]]; then
    echo "error: no ${PROJECT} release workflow run appeared for ${TAG}" >&2
    exit 5
  fi

  echo "Monitoring GitHub Actions run ${RUN_ID}..."
  gh run watch "$RUN_ID" --exit-status --interval 10

  RUN_URL="$(
    gh run view "$RUN_ID" \
      --json url \
      --jq '.url'
  )"
  echo "GH_RUN_URL=${RUN_URL}"
else
  echo "warning: no release workflow found at ${WORKFLOW}; polling GitHub release ${TAG} directly." >&2
fi

echo "Waiting for GitHub release ${TAG}..."
RELEASE_URL=""
for _ in {1..18}; do
  RELEASE_URL="$(
    gh release view "$TAG" \
      --json url \
      --jq '.url' 2>/dev/null || true
  )"
  if [[ -n "$RELEASE_URL" ]]; then
    break
  fi
  sleep 10
done

if [[ -z "$RELEASE_URL" ]]; then
  echo "error: GitHub release ${TAG} was not visible after monitoring" >&2
  exit 6
fi

gh release view "$TAG" --json tagName,name,url,isDraft,isPrerelease,publishedAt,assets
echo "GH_RELEASE_URL=${RELEASE_URL}"
