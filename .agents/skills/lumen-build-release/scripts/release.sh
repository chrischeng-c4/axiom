#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

release_tag_at_head() {
  git tag --points-at HEAD | grep -E '^lumen@[0-9]+\.[0-9]+\.[0-9]+$' | sort -V | tail -1
}

TAG="$(release_tag_at_head || true)"
if [[ -n "$TAG" ]]; then
  echo "Found prepared release tag at HEAD: ${TAG}"
else
  projects/lumen/build.sh release
  TAG="$(release_tag_at_head || true)"
fi

if [[ -z "$TAG" ]]; then
  echo "ERROR: release completed but no lumen@<version> tag points at HEAD" >&2
  exit 1
fi

BRANCH="$(git branch --show-current)"
if [[ -z "$BRANCH" ]]; then
  echo "ERROR: cannot push release source from detached HEAD" >&2
  exit 1
fi

git fetch origin "$BRANCH" --tags

if [[ "$BRANCH" == "main" ]]; then
  git push origin "HEAD:${BRANCH}"
else
  git push --force-with-lease origin "HEAD:${BRANCH}"
fi

git push origin "$TAG"

echo ""
echo "Release pushed."
echo "SOURCE_BRANCH=${BRANCH}"
echo "RELEASE_TAG=${TAG}"
