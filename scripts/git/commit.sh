#!/usr/bin/env bash
# Stage and commit the working tree as one commit, then report it.
# The caller inspects the diff and authors the message; this script refuses
# to run without one.
#
# usage: scripts/git/commit.sh (-m <message> | -F <file>) [--] [path...]
#   paths given -> stage only those; none -> stage everything (git add -A)
# exit: 0 committed; 2 refused (in-progress operation, nothing to commit,
#   or no message)
set -euo pipefail
GIT=(git -c core.fsmonitor=false)

msg='' msg_file=''
while [ $# -gt 0 ]; do
  case "$1" in
    -m) msg=${2:?-m needs a message}; shift 2 ;;
    -F) msg_file=${2:?-F needs a file}; shift 2 ;;
    --) shift; break ;;
    -*) echo "refused: unknown flag $1" >&2; exit 2 ;;
    *) break ;;
  esac
done

gitdir=$("${GIT[@]}" rev-parse --git-dir)
for d in rebase-merge rebase-apply; do
  [ -d "$gitdir/$d" ] && { echo "refused: rebase in progress" >&2; exit 2; }
done
for f in MERGE_HEAD CHERRY_PICK_HEAD REVERT_HEAD; do
  [ -f "$gitdir/$f" ] && { echo "refused: ${f%_HEAD} in progress" >&2; exit 2; }
done

[ -n "$("${GIT[@]}" status --porcelain)" ] \
  || { echo "refused: nothing to commit" >&2; exit 2; }
[ -n "$msg$msg_file" ] \
  || { echo "refused: no message; inspect the diff, then pass -m or -F" >&2; exit 2; }

if [ $# -gt 0 ]; then
  "${GIT[@]}" add -- "$@"
else
  "${GIT[@]}" add -A
fi

if [ -n "$msg_file" ]; then
  "${GIT[@]}" commit -F "$msg_file"
else
  "${GIT[@]}" commit -m "$msg"
fi

"${GIT[@]}" log -1 --stat
"${GIT[@]}" status --short
