#!/usr/bin/env bash
# Merge one PR after its checks finish, then verify it reached MERGED.
# Pending checks are watched to completion; a failing, timed-out,
# cancelled, or action-required check refuses the merge. Never passes
# --delete-branch unless asked. Does not sync the local branch afterwards
# (scripts/git/rebase.sh + scripts/git/push.sh, or /git:land).
#
# usage: scripts/gh/merge-pr.sh [pr] [--strategy squash|merge|rebase] [--delete-branch]
#   no pr -> the one open PR whose head is the current branch (base main)
# exit: 0 merged and verified; 2 refused (cannot resolve exactly one PR, or
#   failing checks)
set -euo pipefail
GIT=(git -c core.fsmonitor=false)

pr='' strategy=squash delete_branch=false
while [ $# -gt 0 ]; do
  case "$1" in
    --strategy) strategy=${2:?--strategy needs squash|merge|rebase}; shift 2 ;;
    --delete-branch) delete_branch=true; shift ;;
    -*) echo "refused: unknown flag $1" >&2; exit 2 ;;
    *) pr=$1; shift ;;
  esac
done
case "$strategy" in squash|merge|rebase) ;; *)
  echo "refused: strategy must be squash, merge, or rebase" >&2; exit 2 ;;
esac

if [ -z "$pr" ]; then
  branch=$("${GIT[@]}" branch --show-current)
  [ -n "$branch" ] || { echo "refused: detached HEAD and no PR given" >&2; exit 2; }
  count=$(gh pr list --head "$branch" --base main --state open --json number --jq 'length')
  [ "$count" = 1 ] \
    || { echo "refused: $count open PRs from $branch to main; name one" >&2; exit 2; }
  pr=$(gh pr list --head "$branch" --base main --state open --json number --jq '.[0].number')
fi

# gh pr checks exits non-zero on "no checks reported"; the failure gate is
# the rollup query below, so the watch itself is best-effort.
gh pr checks "$pr" --watch --interval 15 || true
failing=$(gh pr view "$pr" --json statusCheckRollup --jq \
  '[.statusCheckRollup[]? | select((.conclusion // "") as $c
    | $c == "FAILURE" or $c == "TIMED_OUT" or $c == "CANCELLED"
      or $c == "ACTION_REQUIRED") | .name] | join(", ")')
[ -z "$failing" ] \
  || { echo "refused: failing checks on PR #$pr: $failing" >&2; exit 2; }

if [ "$delete_branch" = true ]; then
  gh pr merge "$pr" "--$strategy" --delete-branch
else
  gh pr merge "$pr" "--$strategy"
fi

gh pr view "$pr" --json state,mergedAt,mergeCommit,url
state=$(gh pr view "$pr" --json state --jq .state)
[ "$state" = MERGED ] || { echo "error: PR #$pr state is $state, not MERGED" >&2; exit 1; }
echo "note: the local branch now trails the merged base until it is rebased and pushed"
