---
name: codex:dispatch
description: Safely dispatch one bounded ticketed or one-shot task to headless Codex using a round-private CODEX_HOME whose execpolicy rules and write allowlist are generated from the task contract, then independently verify its report. Use whenever Claude delegates authoring, audits, measurements, investigation, or tightly scoped implementation to Codex; ticketed tasks resume one thread per live issue, unticketed tasks run once, implementation requires frozen design inputs, and acceptance remains controller-only.
user-invocable: true
---

# Codex Dispatch

## What you do

Three things. Everything else is a verb.

1. **Set the question** — the oracle: what this round claims, and the rows that
   decide it.
2. **Brief the worker** — the injection.
3. **Accept** — only you can.

**How to write those two documents is not here.** `scaffold` emits them as forms
whose every slot carries its own rule as a `<!-- fill -->` comment, and `lint`
refuses a form still holding one. Read the form when you author; the comments are
stripped before the worker sees anything.

**What each verb refuses is not here either.** Every refusal names its own
recovery at the moment it fires. Do what it says.

This skill dispatches *a task*, not a kind of task. Nothing in the script knows
what a work item, an audit, or a migration is. A round is a profile, an oracle,
and an injection; if you find yourself wanting the script to special-case what
you are dispatching, the special case belongs in those three documents.

## How to do it

```
make_profile → worktree → rules → doctor → scaffold → (fill, capture) → lint
             → snapshot → dispatch → verify → review → adjudicate
             → prove ×2 → sweep → accept | discard
```

```bash
S=.claude/skills/codex-dispatch/scripts

python3 $S/make_profile.py --root "$PWD" --repo owner/name \
  --scope <dir> --issue <N> --design-input <path> \
  --write <path>:<line-budget> --gate '<the one command>' \
  --read-commands \
  --out /tmp/codex-dispatch/<N>.json

python3 $S/codex_dispatch.py worktree  $P <N>      # derive the worker checkout
python3 $S/codex_dispatch.py rules     $P          # build the round's CODEX_HOME
python3 $S/codex_dispatch.py doctor    $P          # dispatch_ready=true or findings
python3 $S/codex_dispatch.py scaffold  $P <N>      # emit both forms
python3 $S/codex_dispatch.py capture   $P <N> '<cmd>'   # record a quotable transcript
python3 $S/codex_dispatch.py lint      $P <N>
python3 $S/codex_dispatch.py snapshot  $P <N>
python3 $S/codex_dispatch.py dispatch  $P <N>      # long; run_in_background
python3 $S/codex_dispatch.py verify    $P <N>
python3 $S/codex_dispatch.py review    $P <N>
python3 $S/codex_dispatch.py adjudicate $P <N> admit|reject '<finding>'
python3 $S/codex_dispatch.py prove     $P <N> mutant     # product at baseline
python3 $S/codex_dispatch.py prove     $P <N> candidate  # product restored
python3 $S/codex_dispatch.py sweep     $P <N> <script.py>
python3 $S/codex_dispatch.py accept    $P <N>
python3 $S/codex_dispatch.py discard   $P <N>
python3 $S/codex_dispatch.py status    $P
```

Exit codes: `0` clean, `1` VOID (the evidence is untrustworthy; the round is
over), `2` findings you must adjudicate.

`--gate` and `--allow` are audited byte-for-byte; `--allow-prefix` and
`--read-commands` are not, so keep them read-only. You need one of the latter on
almost every round: Codex reads a file by running a command, so a round with only
a gate hands the worker a task it can only guess at, and the guesses come back
formatted as observations. `doctor` refuses a bounded-write round that authorizes
no prefix.

`resume` continues the round's Codex thread with the injection you have since
edited. Use it when the worker stopped short of the contract, not when it
misunderstood it — a misunderstanding is cheaper to `discard` and re-brief.

## How to verify

Each verb answers one question, and none of them answers another's.

| verb | answers |
|---|---|
| `doctor` | is the declared surface real — does the round's rule file allow every gate command *and refuse the control command* |
| `verify` | were the rules kept: byte-exact commands, untouched protected artifacts, writes inside the allowlist — through git *and* through the write tool's own path log, which can name a path git never sees — and a report that parsed |
| `review` | what did it actually change — the diff, not the claim |
| `prove` | does the gate discriminate: red with the product at baseline, green with it restored |
| `sweep` | does the same defect exist elsewhere in the scope |

`prove` reverts nothing. Restoring the product to baseline while keeping the
round's new checks, and restoring the candidate afterwards, is yours. Restore by
writing the file's bytes, never by `cp -p` or `shutil.copy2` — a preserved mtime
lets the build skip the rebuild, which is a false kill followed by a false green.

A round that changed no product code has no `mutant` proof to make. Say so; do
not record a proof of a mutation you did not perform.

## What not to do

The addressee here is you, the controller. The worker's constraints are
generated into its prompt from the profile and are not restated here.

1. **Never derive a profile from the last round's.** The frozen complement is a
   hash of a tree that has since moved. Re-run `make_profile.py`.
2. **Never hand-edit the round's rule file.** It is generated from
   `task_commands`; an edit survives until the next `rules` and then vanishes,
   and `verify` VOIDs the round when the file it dispatched under no longer
   matches.
3. **Never point the round at your own `~/.codex`.** The isolation is the
   product: this machine's ambient config rewrites every shell call through a
   wrapper and its accumulated rules already allow `git commit`, `git push`, and
   `gh issue close`. Byte-exact auditing is impossible against it.
4. **Never accept on a green report.** The report is a claim about the diff.
   `review` reads the diff; `prove` shows the gate can be red.
5. **Never `nohup ... &` the dispatch.** Use the Bash tool's `run_in_background`
   so the run's exit is tracked and its log is the one `verify` reads.
6. **Never let a scope finding pass by adjudicating it `admit` to move on.**
   `admit` means you read the diff and the write belongs in the round.
7. **Never leave a round undiscarded.** The worktree and its branch persist;
   `discard` removes both. The round's `CODEX_HOME` lives under `state_dir` and
   changed nothing outside it.
8. **Never let the worker close, comment on, or label a ticket.** Acceptance is
   controller-only, and a worker that can mutate the tracker can close the
   ticket that would have recorded its own failure.

## References

- `references/codex-adapter.md` — what `codex exec` gives this skill and where it
  differs from the AGY adapter.
- `.claude/skills/agy-dispatch/references/lifecycle.md` — the runtime-agnostic
  dispatch lifecycle both skills implement. Read it once; it is not duplicated
  here.
