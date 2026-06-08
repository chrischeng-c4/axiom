#!/usr/bin/env bash
set -euo pipefail

# /aw:build:debug dispatcher
#
# Resolves <project> against [[projects]] in .aw/config.toml (matches `name`
# or any entry in `aliases`) and execs <project_path>/build.sh debug.
#
# Usage:
#   build.sh <project>       explicit project name or alias
#   build.sh                 infer from current branch (project-<name>)

MODE="debug"
ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

usage() {
  cat <<'EOF' >&2
Usage: /aw:build:debug [<project>]

  <project>  Project name or alias declared in .aw/config.toml [[projects]].
             Omit on a `project-<name>` branch to infer the project.
EOF
}

PROJECT="${1:-}"
if [[ -z "$PROJECT" ]]; then
  branch="$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "")"
  if [[ "$branch" == project-* && "$branch" != "project-" ]]; then
    PROJECT="${branch#project-}"
  else
    echo "error: no project argument and current branch ($branch) is not project-<name>" >&2
    usage
    exit 2
  fi
fi

PROJECT_PATH="$(awk -v target="$PROJECT" '
  function flush() {
    if (already_printed) return
    if (in_block && matched && path != "") {
      print path
      already_printed=1
      exit 0
    }
    in_block=0; matched=0; name=""; path=""
  }
  /^\[\[projects\]\]/ { flush(); in_block=1; next }
  /^\[/ { flush(); next }
  in_block && /^[[:space:]]*name[[:space:]]*=[[:space:]]*"/ {
    match($0, /"[^"]*"/); name=substr($0, RSTART+1, RLENGTH-2)
    if (name == target) matched=1
  }
  in_block && /^[[:space:]]*path[[:space:]]*=[[:space:]]*"/ {
    match($0, /"[^"]*"/); path=substr($0, RSTART+1, RLENGTH-2)
  }
  in_block && /^[[:space:]]*aliases[[:space:]]*=/ {
    if (index($0, "\"" target "\"")) matched=1
  }
  END { flush(); if (!already_printed) exit 1 }
' .aw/config.toml)" || {
  echo "error: project '$PROJECT' not found in .aw/config.toml" >&2
  exit 3
}

BUILD="$PROJECT_PATH/build.sh"
if [[ ! -x "$BUILD" ]]; then
  echo "error: $BUILD not found or not executable" >&2
  echo "       $PROJECT_PATH must provide an executable build.sh accepting '$MODE'" >&2
  exit 4
fi

exec "$BUILD" "$MODE"
