#!/bin/sh
# Gate for the DX contract payload relocation (#3707).
#
# Two kinds of row.  PROGRESS rows are red at the change's base and green when
# it lands.  GUARD rows are green at the base and must stay green: they hold
# the emitted protocol document byte-identical across the move, masking only
# the one string the change is supposed to alter.
#
# The guard rows are the reason this is a script rather than a build.  A plain
# `cargo build` accepts a payload that was reflowed, re-indented, or had a key
# reordered during the move -- `serde_yaml` parses all three happily -- and the
# emitted document would silently differ.
set -e
cd "$(dirname "$0")/../../.."

PAYLOAD=apps/lumen/src/dx-contract.yaml
PAYLOAD_SHA=9e2b79829587f1a03030fea901c2a628c55451faf29171ab35419c8ed8986efc
OLD_REF='apps/lumen/tech-design/interfaces/dx/lumen-dx-contract.md#dx-contract'
BIN=./target/debug/lumen
fail=0
note() { echo "$1 $2"; [ "$1" = FAIL ] && fail=1; return 0; }

cargo build -p lumen --bin lumen >/dev/null 2>&1 \
  && note PASS "build" || { note FAIL "build"; echo "GATE FAIL"; exit 1; }

# PROGRESS: the payload lives in its own file, byte for byte.
if [ ! -f "$PAYLOAD" ]; then
  note FAIL "payload absent: $PAYLOAD"
else
  got=$(shasum -a 256 "$PAYLOAD" | cut -d' ' -f1)
  [ "$got" = "$PAYLOAD_SHA" ] && note PASS "payload sha" \
    || note FAIL "payload sha: $got"
fi

# GUARD: every topic's rendered document, with the ref masked.
while read -r topic want; do
  got=$("$BIN" llm --topic "$topic" --format json 2>/dev/null \
        | sed -e "s|$OLD_REF|@REF|g" -e "s|$PAYLOAD|@REF|g" \
        | shasum -a 256 | cut -d' ' -f1)
  [ "$got" = "$want" ] && note PASS "topic $topic" \
    || note FAIL "topic $topic: $got"
done <<'TOPICS'
outline 55a98f711afc64eac94ab3f451c6ecf8ccb616e01a0429fff9d8276587d7eb07
local-search dc31b4ce08e91e325fe0d4619e79183b0844462929069b62b36267b295fa3602
model-schema 86463e4b1b12a5045ff9c0bccad183c4232b8c6c930f7483da29803fd6f43771
select-query a6477da6c3b5ddd258bd5d11c6b5ecbb84378cc5bc07ee730ef67cd92e382760
integrate-source-db c320339b3e8035019a03c649a0546aed2e39127ab7a1fa40b513246e71f287f0
authenticate f3a917f535b6778ec786acc535e89fcaa7499005aa848454559c5853ec9b5d61
connect-kubernetes 091b961edf6cc6cafb40f47c9f1b109cf8b6f2e7528fb8b531dfdf397c44d115
deploy-kubernetes 4f4ba41a791676eb18b8d7ccb296977e09ec764c51847aaff8e01b738e312bfb
grant-access c1afa204f49c10390d8d92aa5ac4005ad1c92ed4478ad445261a833b837aaa18
backup-restore 817deb763d125b511b28626046a45269468f80c096284908672078a9dccff9bf
generate-client 3bb0d4dda7de782ff791e5324c731edbcb0e9b0166eadea6be742014dc863204
diagnose 53cf13ed23570c96e31e963cc6d5376e0125a2f55d50f31a158247926f20bb06
TOPICS

# PROGRESS: the emitted ref moved wholesale.  44 = 11 in `outline` + 3 each
# in the other eleven; it is the only tech-design path the CLI emits at all.
o=0; n=0
for t in outline local-search model-schema select-query integrate-source-db \
         authenticate connect-kubernetes deploy-kubernetes grant-access \
         backup-restore generate-client diagnose; do
  doc=$("$BIN" llm --topic "$t" --format json 2>/dev/null)
  o=$((o + $(printf '%s' "$doc" | grep -o "$OLD_REF" | wc -l | tr -d ' ')))
  n=$((n + $(printf '%s' "$doc" | grep -o "$PAYLOAD" | wc -l | tr -d ' ')))
done
[ "$o" -eq 0 ]  && note PASS "old ref occurrences 0"  || note FAIL "old ref occurrences $o (want 0)"
[ "$n" -eq 44 ] && note PASS "new ref occurrences 44" || note FAIL "new ref occurrences $n (want 44)"

# PROGRESS: the Markdown is no longer compiled in, and is still on disk.
#
# Without this row the gate has a hole: repointing DX_CONTRACT_REF at the new
# path while `include_str!` still names the .md turns every row above green
# and leaves #3708 blocked exactly as it was.  `embed` is the column that
# says the compile-time dependency actually went away; `md=102` says the
# file itself is still here, because deleting it belongs to #3708, not here.
# `embed` targets 1, not 0.  `apps/lumen` compiles in TWO Markdown files under
# the tree: this contract, and the TD that
# `apps/lumen/tests/capability_stateful_workload_linkage.rs:5` embeds.  Only
# the first is a product input and only the first moves here; the test's
# assertions are about the TD document itself and are retired with it by #3708.
# Round 1 of this issue asserted `embed=0` and passed, because the probe then
# scanned `include_` line by line and that embed puts the macro and its string
# on separate lines.
#
# `other` targets 19, from a base of 21: the change repoints both `dx.rs:18`
# and `dx.rs:19`.  This script is itself a .sh file naming the old path, so the
# probe counts it -- correctly -- and it stays standing until #3708 retires it.
want='md=102 lock=1 py=51 hdr=589 files=105 other=19 embed=1'
got=$(python3 scripts/td-retire-probe.py apps/lumen/tech-design)
[ "$got" = "$want" ] && note PASS "probe" || note FAIL "probe: $got"

[ "$fail" -eq 0 ] && echo "GATE PASS" || echo "GATE FAIL"
exit "$fail"
