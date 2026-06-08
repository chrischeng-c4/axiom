#!/usr/bin/env bash
# HANDWRITE-BEGIN gap="missing-generator:hand-written:0db8f9a1" tracker="pending-tracker" reason="Integration test fixture for R6 (Hook 1 PostToolUse). Simulates a `aw wi fill-section --apply` Bash call, then fires hook1-post-apply-validate.sh as PostToolUse. Asserts: (a) aw validate is invoked with the correct slug, (b) a DispatchEnvelope is emitted on stdout when validate exits 0, (c) an ErrorEnvelope is emitted when validate exits non-zero. Pass: all three assertions exit 0; the lock file is removed after each run."
#
# Integration test fixture — Hook 1 PostToolUse auto-validate.
#
# Exercised by `cargo test --test hook1_post_apply_validate` via a
# wrapper that shells out here. Uses a stub `aw` shim on PATH to
# capture the invocation and assert envelope shape.
#
# @spec projects/agentic-workflow/tech-design/surface/specs/score-mainthread-only-execution.md#changes (R6)

set -euo pipefail

HOOK="${HOOK:-$(git rev-parse --show-toplevel)/.claude/hooks/hook1-post-apply-validate.sh}"
SLUG="test-fixture-slug-r6"
LOCK="/tmp/aw-apply-lock-${SLUG}"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP" "$LOCK"' EXIT

mk_stub() {
  local exit_code="$1" envelope="$2"
  cat > "$TMP/aw" <<EOF
#!/usr/bin/env bash
echo "STUB-CALLED: \$*" >> "$TMP/calls.log"
case "\$*" in
  *validate*) echo '$envelope'; exit $exit_code ;;
  *)          exit 0 ;;
esac
EOF
  chmod +x "$TMP/aw"
}

run_hook() {
  local cmd="$1"
  local input
  input="$(jq -n --arg cmd "$cmd" '{tool_name:"Bash", tool_input:{command:$cmd}}')"
  PATH="$TMP:$PATH" bash "$HOOK" <<< "$input"
}

# Assertion (a): validate invoked with the correct slug.
mk_stub 0 "{\"action\":\"dispatch\",\"slug\":\"$SLUG\",\"invoke\":{\"command\":\"aw wi review\"}}"
out="$(run_hook "aw wi fill-section --apply --slug $SLUG --section requirements")"
grep -q "STUB-CALLED: wi validate $SLUG" "$TMP/calls.log" \
  || { echo "FAIL (a): validate not invoked with slug $SLUG"; cat "$TMP/calls.log"; exit 1; }

# Assertion (b): DispatchEnvelope on validate exit 0.
echo "$out" | jq -e '.action == "dispatch"' >/dev/null \
  || { echo "FAIL (b): expected dispatch envelope, got: $out"; exit 1; }

# Assertion (c): ErrorEnvelope on validate non-zero.
mk_stub 1 'validation rejected: bad payload'
out2="$(run_hook "aw td create --apply --slug $SLUG")"
echo "$out2" | jq -e '.action == "error"' >/dev/null \
  || { echo "FAIL (c): expected error envelope, got: $out2"; exit 1; }

# Lock cleanup: hook1 must remove the lock after validate.
[[ ! -e "$LOCK" ]] || { echo "FAIL: lock $LOCK not cleaned up"; exit 1; }

echo "PASS: hook1 fixture (3/3 assertions + lock cleanup)"
# HANDWRITE-END
