#!/usr/bin/env bash
# Rebase the current branch onto <branch>, or onto a freshly fetched
# origin/main when no branch is given. Does not commit, stash, or push.
#
# On conflict it stops and lists the conflicted files; the caller resolves
# them, stages, runs `git -c core.fsmonitor=false rebase --continue`
# (repeating until the rebase completes), then reruns this script — a
# completed rebase makes the rerun a no-op that just prints the report.
#
# usage: scripts/git/rebase.sh [branch]
# exit: 0 rebased or already up to date; 2 refused (dirty tree, in-progress
#   operation, unresolvable target, target == current branch); 3 stopped on
#   conflicts
set -euo pipefail
GIT=(git -c core.fsmonitor=false)

gitdir=$("${GIT[@]}" rev-parse --git-dir)
for d in rebase-merge rebase-apply; do
  [ -d "$gitdir/$d" ] && { echo "refused: rebase already in progress" >&2; exit 2; }
done
for f in MERGE_HEAD CHERRY_PICK_HEAD REVERT_HEAD; do
  [ -f "$gitdir/$f" ] && { echo "refused: ${f%_HEAD} in progress" >&2; exit 2; }
done
[ -z "$("${GIT[@]}" status --porcelain)" ] \
  || { echo "refused: dirty tree; commit or set the work aside first" >&2; exit 2; }

current=$("${GIT[@]}" branch --show-current)
[ -n "$current" ] || { echo "refused: detached HEAD" >&2; exit 2; }

if [ $# -eq 0 ]; then
  "${GIT[@]}" fetch origin main
  target=origin/main
else
  target=$1
  if ! "${GIT[@]}" rev-parse --verify --quiet "$target^{commit}" >/dev/null; then
    "${GIT[@]}" fetch origin "$target"
    target=origin/$1
    "${GIT[@]}" rev-parse --verify --quiet "$target^{commit}" >/dev/null \
      || { echo "refused: cannot resolve $1 locally or as origin/$1" >&2; exit 2; }
  fi
fi
[ "$target" != "$current" ] \
  || { echo "refused: target $target is the current branch" >&2; exit 2; }

if ! "${GIT[@]}" rebase "$target"; then
  echo
  echo "stopped: conflicts. Conflicted files:"
  "${GIT[@]}" diff --name-only --diff-filter=U
  echo "Resolve each file, stage it, run:"
  echo "  git -c core.fsmonitor=false rebase --continue"
  echo "then rerun this script for the report."
  exit 3
fi

"${GIT[@]}" status --short
echo "HEAD...$target: $("${GIT[@]}" rev-list --left-right --count "HEAD...$target")"
if upstream=$("${GIT[@]}" rev-parse --abbrev-ref --symbolic-full-name '@{u}' 2>/dev/null); then
  echo "HEAD...$upstream: $("${GIT[@]}" rev-list --left-right --count 'HEAD...@{u}')"
fi
"${GIT[@]}" log --oneline -5
