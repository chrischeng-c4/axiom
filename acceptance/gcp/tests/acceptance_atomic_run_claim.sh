#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ACCEPTANCE_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
source "$ACCEPTANCE_ROOT/scripts/acceptance-lock.sh"

test_root="$(mktemp -d "${TMPDIR:-/tmp}/sift-atomic-run-claim.XXXXXX")"
claim_root="$test_root/claims"
final_claim="$(
  acceptance_run_claim_path "$claim_root" "test-project" "claim-red" "sift"
)"
first_candidate="$claim_root/.acceptance-run-aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.json"
second_candidate="$claim_root/.acceptance-run-bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb.json"

cleanup_test() {
  find "$test_root" -type f -delete
  find "$test_root" -depth -type d -empty -delete
}
trap cleanup_test EXIT INT TERM

write_acceptance_run_owner \
  "$first_candidate" "test-project" "claim-red" "sift" \
  "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" \
  "$test_root/state-a" "$test_root/evidence-a" \
  "11111111" "11111111" "first-generation" \
  "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
write_acceptance_run_owner \
  "$second_candidate" "test-project" "claim-red" "sift" \
  "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb" \
  "$test_root/state-b" "$test_root/evidence-b" \
  "22222222" "22222222" "second-generation" \
  "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"

set +e
ln "$first_candidate" "$final_claim" 2>/dev/null &
first_pid=$!
ln "$second_candidate" "$final_claim" 2>/dev/null &
second_pid=$!
wait "$first_pid"
first_status=$?
wait "$second_pid"
second_status=$?
set -e

if [[ "$first_status" == "0" ]]; then
  [[ "$second_status" != "0" ]]
  cmp -s "$first_candidate" "$final_claim"
  verify_acceptance_run_owner \
    "$final_claim" "test-project" "claim-red" "sift" \
    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" \
    "$test_root/state-a" "$test_root/evidence-a" \
    "11111111" "11111111" "first-generation" \
    "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
else
  [[ "$second_status" == "0" ]]
  cmp -s "$second_candidate" "$final_claim"
  verify_acceptance_run_owner \
    "$final_claim" "test-project" "claim-red" "sift" \
    "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb" \
    "$test_root/state-b" "$test_root/evidence-b" \
    "22222222" "22222222" "second-generation" \
    "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"
fi

claim_mode="$(stat -f '%Lp' "$final_claim" 2>/dev/null \
  || stat -c '%a' "$final_claim")"
[[ "$claim_mode" == "600" ]]

# A process that stops before the hard-link install leaves only a random
# candidate. It does not reserve the fixed claim of another run identity.
other_claim="$(
  acceptance_run_claim_path "$claim_root" "test-project" "other-red" "sift"
)"
other_candidate="$claim_root/.acceptance-run-eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee.json"
write_acceptance_run_owner \
  "$other_candidate" "test-project" "other-red" "sift" \
  "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee" \
  "$test_root/state-other" "$test_root/evidence-other" \
  "33333333" "33333333" "other-generation" \
  "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
[[ ! -e "$other_claim" ]]
ln "$other_candidate" "$other_claim"
verify_acceptance_run_owner \
  "$other_claim" "test-project" "other-red" "sift" \
  "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee" \
  "$test_root/state-other" "$test_root/evidence-other" \
  "33333333" "33333333" "other-generation" \
  "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"

echo "atomic local acceptance run claim E2E: ok"
