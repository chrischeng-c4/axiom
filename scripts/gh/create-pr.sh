#!/usr/bin/env bash
# Open — or reuse — the one open PR from the current branch to <base>
# (default main), then print its state. Does not push, wait on checks, or
# merge. The branch must already be pushed and in sync with its upstream
# (scripts/git/push.sh first).
#
# usage: scripts/gh/create-pr.sh [base]
# exit: 0 PR open and reported; 2 refused (on base branch, no upstream, or
#   not in sync with upstream)
set -euo pipefail
GIT=(git -c core.fsmonitor=false)

base=${1:-main}
branch=$("${GIT[@]}" branch --show-current)
[ -n "$branch" ] || { echo "refused: detached HEAD" >&2; exit 2; }
[ "$branch" != "$base" ] \
  || { echo "refused: current branch is the base itself" >&2; exit 2; }
"${GIT[@]}" rev-parse --abbrev-ref --symbolic-full-name '@{u}' >/dev/null 2>&1 \
  || { echo "refused: no upstream; run scripts/git/push.sh first" >&2; exit 2; }
div=$("${GIT[@]}" rev-list --left-right --count 'HEAD...@{u}')
[ "$div" = "$(printf '0\t0')" ] \
  || { echo "refused: HEAD...@{u} is $div, not 0 0; run scripts/git/push.sh first" >&2; exit 2; }

pr=$(gh pr list --head "$branch" --base "$base" --state open \
  --json number --jq '.[0].number // empty')
if [ -n "$pr" ]; then
  echo "reusing open PR #$pr"
else
  gh pr create --base "$base" --head "$branch" --fill
  pr=$(gh pr list --head "$branch" --base "$base" --state open \
    --json number --jq '.[0].number // empty')
  [ -n "$pr" ] || { echo "error: created PR not found" >&2; exit 1; }
  echo "created PR #$pr"
fi

gh pr view "$pr" --json number,url,state,mergeable,mergeStateStatus,statusCheckRollup
