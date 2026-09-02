#!/usr/bin/env bash
# Push the current branch to origin, choosing the safe variant from the
# upstream divergence:
#   no upstream        -> git push -u origin HEAD
#   ahead only (N 0)   -> git push
#   rewritten (N M>0)  -> git push --force-with-lease
#   in sync (0 0)      -> nothing to push
#   behind only (0 M)  -> refused (nothing local to publish)
# Never bare --force. A force push to a persistent ref (main, examples,
# project-mamba, project-lumen, app/*, lib/*) is refused unless
# --force-persistent-ok is passed, which the caller may only do after
# explicit user confirmation.
#
# usage: scripts/git/push.sh [--force-persistent-ok]
# exit: 0 pushed or nothing to push; 2 refused; 4 rejected by the remote
#   (the remote moved — divergence is reported; do not override it)
set -euo pipefail
GIT=(git -c core.fsmonitor=false)

allow_persistent=false
[ "${1:-}" = "--force-persistent-ok" ] && allow_persistent=true

branch=$("${GIT[@]}" branch --show-current)
[ -n "$branch" ] || { echo "refused: detached HEAD" >&2; exit 2; }

push_or_report() {
  if ! "${GIT[@]}" push "$@"; then
    echo "rejected: the remote moved. Fetching to report the divergence:" >&2
    "${GIT[@]}" fetch origin "$branch" || true
    "${GIT[@]}" rev-list --left-right --count 'HEAD...@{u}' >&2 || true
    exit 4
  fi
}

if upstream=$("${GIT[@]}" rev-parse --abbrev-ref --symbolic-full-name '@{u}' 2>/dev/null); then
  read -r ahead behind <<EOF
$("${GIT[@]}" rev-list --left-right --count 'HEAD...@{u}')
EOF
  if [ "$behind" -gt 0 ] && [ "$ahead" -eq 0 ]; then
    echo "refused: only behind $upstream ($ahead $behind); nothing local to publish" >&2
    exit 2
  elif [ "$behind" -gt 0 ]; then
    case "$branch" in
      main|examples|project-mamba|project-lumen|app/*|lib/*)
        if [ "$allow_persistent" != true ]; then
          echo "refused: force-push to persistent ref $branch needs explicit user" >&2
          echo "confirmation; rerun with --force-persistent-ok only after getting it" >&2
          exit 2
        fi ;;
    esac
    push_or_report --force-with-lease
  elif [ "$ahead" -gt 0 ]; then
    push_or_report
  else
    echo "nothing to push: HEAD...$upstream is 0 0"
  fi
else
  push_or_report -u origin HEAD
fi

echo "HEAD...@{u}: $("${GIT[@]}" rev-list --left-right --count 'HEAD...@{u}')"
