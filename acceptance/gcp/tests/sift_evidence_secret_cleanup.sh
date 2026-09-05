#!/usr/bin/env bash
set -euo pipefail

test_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
acceptance_root="$(cd "$test_dir/.." && pwd)"
helper="$acceptance_root/scripts/sift-evidence-secrets.sh"
verifier="$acceptance_root/scripts/verify-sift-mvp.sh"
cleanup_script="$acceptance_root/scripts/cleanup.sh"
test_root="$(mktemp -d "${TMPDIR:-/tmp}/sift-evidence-secrets.XXXXXX")"
cleanup_test() {
  local status="$1"
  trap - EXIT INT TERM
  set +e
  find "$test_root" -type l -delete
  find "$test_root" -type f -delete
  find "$test_root" -depth -type d -empty -delete
  exit "$status"
}
trap 'cleanup_test "$?"' EXIT
trap 'exit 130' INT TERM

[[ -f "$helper" && ! -L "$helper" ]] || {
  echo "the Sift evidence secret helper is missing or unsafe" >&2
  exit 1
}
source "$helper"

regular="$test_root/regular"
mkdir -p "$regular/kubernetes"
printf 'secret-token\n' > "$regular/kubernetes/sift-rig.token"
sift_remove_ephemeral_evidence_secrets "$regular"
[[ ! -e "$regular/kubernetes/sift-rig.token" \
  && ! -L "$regular/kubernetes/sift-rig.token" ]] || {
  echo "the ephemeral Sift token remained in evidence" >&2
  exit 1
}

protected="$test_root/protected"
linked_token="$test_root/linked-token"
mkdir -p "$protected/kubernetes" "$linked_token/kubernetes"
printf 'keep-me\n' > "$protected/token"
ln -s "$protected/token" "$linked_token/kubernetes/sift-rig.token"
sift_remove_ephemeral_evidence_secrets "$linked_token"
[[ -f "$protected/token" \
  && ! -e "$linked_token/kubernetes/sift-rig.token" \
  && ! -L "$linked_token/kubernetes/sift-rig.token" ]] || {
  echo "token cleanup followed an unsafe final symlink" >&2
  exit 1
}

linked_parent="$test_root/linked-parent"
mkdir -p "$linked_parent"
ln -s "$protected/kubernetes" "$linked_parent/kubernetes"
printf 'keep-parent-target\n' > "$protected/kubernetes/sift-rig.token"
if sift_remove_ephemeral_evidence_secrets "$linked_parent"; then
  echo "token cleanup accepted a symlinked Kubernetes evidence directory" >&2
  exit 1
fi
[[ -f "$protected/kubernetes/sift-rig.token" ]] || {
  echo "token cleanup changed a symlinked parent target" >&2
  exit 1
}

rg -F -- 'source "$SCRIPT_DIR/sift-evidence-secrets.sh"' "$verifier" >/dev/null
rg -F -- 'sift_remove_ephemeral_evidence_secrets "$EVIDENCE_DIR"' \
  "$verifier" >/dev/null
rg -F -- 'source "$ACCEPTANCE_ROOT/scripts/sift-evidence-secrets.sh"' \
  "$cleanup_script" >/dev/null
rg -F -- 'sift_remove_ephemeral_evidence_secrets "$EVIDENCE_DIR"' \
  "$cleanup_script" >/dev/null

echo "Sift evidence secret cleanup: ok"
