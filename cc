#!/usr/bin/env bash
set -e
if [ -n "$CCLAB_CLAUDE_CONFIG_DIR" ]; then
  export CLAUDE_CONFIG_DIR="$CCLAB_CLAUDE_CONFIG_DIR"
fi
exec claude "$@"
