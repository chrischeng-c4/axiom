#!/usr/bin/env bash
# agy-wave.sh — headless agy dispatch. One agy process per ticket.
#
#   ./agy-wave.sh lock                 # remove cargo / push allow-rules, add the pinned binary
#   ./agy-wave.sh dispatch 2642        # serial: one ticket
#   ./agy-wave.sh dispatch 2636 2643   # parallel: one agy process each, then wait
#   ./agy-wave.sh status               # per-run triage from the logs
#   ./agy-wave.sh unlock               # restore the pre-wave permission set
#
# `lock` is not optional. permissions.allow ships with command(cargo), and
# headless mode auto-approves anything on that list with no prompt.
#
# Overridable: AGY_MODEL AGY_TIMEOUT AGY_REPO AGY_ROOT AGY_BIN AGY_SHA AGY_LOGDIR
set -u

HERE="$(cd "$(dirname "$0")" && pwd)"
REPO="${AGY_REPO:-chrischeng-c4/axiom}"
ROOT="${AGY_ROOT:-/Users/chrischeng/axiom/project-mamba}"
# The `-cfbeef43138f9938` suffix is cargo's METADATA hash, not a content hash: it
# is stable across source edits, so a rebuild overwrites the artifact in place
# under the same name. A binary inside target/ is therefore never pinned — only
# the sha256 distinguishes the wave's binary from its successor. Default to the
# staged copy OUTSIDE target/; point AGY_BIN back into target/ only to re-pin a
# wave deliberately. Fired for real on 2026-07-27: a concurrent mamba-dev build
# replaced the target/ artifact mid-wave and the sha guard below caught it.
BIN="${AGY_BIN:-/tmp/waveA/pinned/cpython_ported_integration-cfbeef43138f9938}"
SHA="${AGY_SHA:-f815a9a7e4d78190e70570f5eaea66bc24579097dc4b6a93bdf2a4d02dc3eb97}"
MODEL="${AGY_MODEL:-gemini-3.6-flash-high}"
TIMEOUT="${AGY_TIMEOUT:-30m}"
LOGDIR="${AGY_LOGDIR:-$HERE/runs}"
# TWO permission sources are consulted, and both must be handled.
#   SETTINGS  per-CLI  permissions.allow
#   GLOBAL    shared   userSettings.globalPermissionGrants.{allow,deny}
# GLOBAL ships with command(cargo test) and command(cargo check). Removing
# command(cargo) from SETTINGS alone leaves the rebuild path wide open.
SETTINGS="$HOME/.gemini/antigravity-cli/settings.json"
GLOBAL="$HOME/.gemini/config/config.json"
BACKUP="$HERE/.settings.prewave.json"
GBACKUP="$HERE/.config.prewave.json"
TMPL="$HERE/prompt.tmpl"
DENY_MARK="command(cargo)"

lock() {
  [ -f "$BACKUP" ]  || cp "$SETTINGS" "$BACKUP"
  [ -f "$GBACKUP" ] || cp "$GLOBAL" "$GBACKUP"
  BIN="$BIN" SETTINGS="$SETTINGS" GLOBAL="$GLOBAL" python3 - <<'PY'
import json, os
b = os.environ["BIN"]

p = os.environ["SETTINGS"]
s = json.load(open(p))
a = [x for x in s["permissions"]["allow"]
     if x not in ("command(cargo)", "command(cargo check)",
                  "command(git push --force-with-lease)")]
# Every tool named in prompt.tmpl's "complete tool set" must appear here.
# A tool the prompt advertises and settings.json does not grant is silent
# death, and the executor cannot tell that from a bug in its own command.
# #2696 died at step 30 on `rg`, which prompt.tmpl has listed since v1.
# `env` is deliberately absent: `env FOO=1 cargo ...` reopens the rebuild path.
for extra in (f"command({b})", "command(sed)", "command(sort)", "command(uniq)",
              "command(echo)", "command(printf)", "command(awk)", "command(cut)",
              "command(xargs)", "command(rg)", "command(basename)",
              "command(dirname)", "command(realpath)", "command(stat)",
              "command(tee)", "command(tr)", "command(nl)", "command(seq)",
              "command(comm)", "command(paste)", "command(date)",
              "command(file)", "command(fold)", "command(expand)",
              "command(split)", "command(uname)"):
    if extra not in a:
        a.append(extra)
s["permissions"]["allow"] = a
json.dump(s, open(p, "w"), indent=2)

# deny beats allow, and is the only thing that closes globalPermissionGrants
g = os.environ["GLOBAL"]
d = json.load(open(g))
gr = d["userSettings"]["globalPermissionGrants"]
gr["deny"] = list(dict.fromkeys(gr.get("deny", []) +
                  ["command(cargo)", "command(git push)", "command(git commit)"]))
json.dump(d, open(g, "w"), indent=1)
print("locked: cargo/push removed from CLI allow; cargo/push/commit DENIED globally")
PY
}

unlock() {
  ok=1
  if [ -f "$BACKUP" ];  then cp "$BACKUP" "$SETTINGS"; rm -f "$BACKUP";   else ok=0; fi
  if [ -f "$GBACKUP" ]; then cp "$GBACKUP" "$GLOBAL"; rm -f "$GBACKUP"; else ok=0; fi
  [ "$ok" = 1 ] && echo "unlocked: both permission files restored" \
                || echo "unlock incomplete — a backup was missing" >&2
}

prompt_for() {
  sed -e "s|{{ISSUE}}|$1|g" -e "s|{{REPO}}|$REPO|g" -e "s|{{ROOT}}|$ROOT|g" \
      -e "s|{{BIN}}|$BIN|g"  -e "s|{{SHA}}|$SHA|g" "$TMPL"
}

dispatch() {
  [ -f "$TMPL" ] || { echo "missing $TMPL" >&2; exit 1; }
  mkdir -p "$LOGDIR"
  if grep -q '"command(cargo)"' "$SETTINGS"; then
    echo "REFUSING: command(cargo) is still allow-listed. Run './agy-wave.sh lock' first." >&2
    exit 1
  fi
  if ! GLOBAL="$GLOBAL" python3 -c 'import json,os;d=json.load(open(os.environ["GLOBAL"]));import sys;sys.exit(0 if "command(cargo)" in d["userSettings"]["globalPermissionGrants"].get("deny",[]) else 1)'; then
    echo "REFUSING: command(cargo) is not in the GLOBAL deny list — globalPermissionGrants still grants cargo test/check. Run './agy-wave.sh lock' first." >&2
    exit 1
  fi
  if [ -n "$SHA" ] && [ -e "$BIN" ]; then
    pre=$(shasum -a 256 "$BIN" | cut -d' ' -f1)
    [ "$pre" = "$SHA" ] || { echo "REFUSING: binary sha is $pre, expected $SHA" >&2; exit 1; }
  fi

  for n in "$@"; do
    echo "dispatch #$n -> $LOGDIR/$n.log"
    agy -p "$(prompt_for "$n")" --model "$MODEL" --effort high \
        --add-dir "$ROOT" --add-dir /tmp/waveA \
        --print-timeout "$TIMEOUT" --log-file "$LOGDIR/$n.agy.log" \
        > "$LOGDIR/$n.log" 2>&1 &
  done
  wait

  if [ -n "$SHA" ] && [ -e "$BIN" ]; then
    post=$(shasum -a 256 "$BIN" | cut -d' ' -f1)
    echo "---"
    if [ "$post" = "$SHA" ]; then echo "binary sha UNCHANGED — measurements commensurable"
    else echo "*** BINARY REBUILT during the wave: $post — every result in this wave is suspect"; fi
  fi
  status
}

status() {
  for f in "$LOGDIR"/*.log; do
    case "$f" in *.agy.log) continue;; esac
    [ -e "$f" ] || continue
    n=$(basename "$f" .log)
    if   grep -q 'auto-denied' "$f";        then v="DENIED — a command was not allow-listed; see log"
    elif [ ! -s "$f" ];                     then v="EMPTY — timeout or crash; nothing was reported"
    elif grep -q 'issuecomment' "$f";       then v="reported — comment posted, VERIFY IT"
    else                                         v="finished without a comment URL — check the log"; fi
    printf '  #%-6s %s\n' "$n" "$v"
  done
}

# A denial at step N discards the run but NOT the conversation. Recover instead
# of re-dispatching: `convid <issue>` finds the id, `resume <issue> <text>`
# continues it with a corrective instruction.
convid() {
  grep -o 'Streaming conversation [0-9a-f-]*' "$LOGDIR/$1.agy.log" | tail -1 | awk '{print $3}'
}

resume() {
  local n="$1"; shift
  local id; id=$(convid "$n")
  [ -n "$id" ] || { echo "no conversation id in $LOGDIR/$n.agy.log" >&2; exit 1; }
  echo "resuming #$n conversation $id"
  agy --conversation "$id" -p "$*" --model "$MODEL" --effort high \
      --add-dir "$ROOT" --add-dir /tmp/waveA \
      --print-timeout "$TIMEOUT" --log-file "$LOGDIR/$n.resume.agy.log" \
      > "$LOGDIR/$n.log" 2>&1
  status
}

# Why did a run die? The conversation store holds every command it ran.
lastcmds() {
  local id; id=$(convid "$1")
  ID="$id" python3 - <<'PY'
import os, re, sqlite3
p = f"/Users/chrischeng/.gemini/antigravity-cli/conversations/{os.environ['ID']}.db"
con = sqlite3.connect(f"file:{p}?mode=ro", uri=True)
CMD = re.compile(rb'CommandLine[^\x00-\x1f]{0,4}([ -~]{2,400})')
out = []
for (v,) in con.execute("select step_payload from steps"):
    if isinstance(v, (bytes, bytearray)):
        out += [m.group(1).decode("utf-8", "replace") for m in CMD.finditer(v)]
for r in out[-8:]:
    print(" *", r[:240])
PY
}

case "${1:-}" in
  lock)     lock ;;
  unlock)   unlock ;;
  dispatch) shift; dispatch "$@" ;;
  status)   status ;;
  prompt)   prompt_for "${2:-NNNN}" ;;
  convid)   convid "${2:?issue}" ;;
  lastcmds) lastcmds "${2:?issue}" ;;
  resume)   shift; resume "$@" ;;
  *)        sed -n '2,12p' "$0" ;;
esac
