#!/usr/bin/env bash
# HANDWRITE-BEGIN gap="missing-generator:hand-written:fc907b9a" tracker="pending-tracker" reason="Integration test fixture for R7 (Hook 2 PreToolUse). Creates a synthetic pending-apply lock file, then fires hook2-pre-apply-guard.sh as PreToolUse against a second `--apply` Bash call. Asserts: (a) the hook exits 2 with reason 'unvalidated apply pending' when the lock exists, (b) the hook exits 0 and writes the lock file when no prior lock is present. Pass: both assertions exit as expected; no stale lock files remain after cleanup."
#
# Integration test fixture — Hook 2 PreToolUse guard.
#
# @spec projects/agentic-workflow/tech-design/surface/specs/score-mainthread-only-execution.md#changes (R7)

set -uo pipefail

HOOK="${HOOK:-$(git rev-parse --show-toplevel)/.claude/hooks/hook2-pre-apply-guard.sh}"
SLUG="test-fixture-slug-r7"
LOCK="/tmp/aw-apply-lock-${SLUG}"
trap 'rm -f "$LOCK"' EXIT

run_hook() {
  local cmd="$1"
  local input
  input="$(jq -n --arg cmd "$cmd" '{tool_name:"Bash", tool_input:{command:$cmd}}')"
  bash "$HOOK" <<< "$input"
}

# Assertion (a): lock present → block (exit 2).
: > "$LOCK"
err="$(run_hook "aw wi fill-section --apply --slug $SLUG --section scope" 2>&1)"
rc=$?
if [[ $rc -ne 2 ]]; then
  echo "FAIL (a): expected exit 2 when lock present, got $rc"; exit 1
fi
echo "$err" | grep -qi "unvalidated apply pending" \
  || { echo "FAIL (a): missing reason text. stderr: $err"; exit 1; }

# Assertion (b): no lock → allow (exit 0) + write lock.
rm -f "$LOCK"
run_hook "aw td create --apply --slug $SLUG" >/dev/null 2>&1
rc=$?
if [[ $rc -ne 0 ]]; then
  echo "FAIL (b): expected exit 0 when no lock, got $rc"; exit 1
fi
[[ -e "$LOCK" ]] || { echo "FAIL (b): lock not written"; exit 1; }

echo "PASS: hook2 fixture (2/2 assertions + lock written)"
# HANDWRITE-END
