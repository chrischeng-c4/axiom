---
name: agy:dispatch
description: Dispatch one bounded ticket to agy in headless mode and verify the result independently. Use whenever work is delegated to AGY instead of pasting a prompt into an interactive agy session — measurement sweeps, audits, transcription, any atomic ticket whose report must be checked before it can be believed. Owns the permission lock, the dispatch prompt, the pre-result oracle, and the closure rule.
user-invocable: true
---

# /agy:dispatch

Headless agy is a **verified-report channel**, not an autonomous worker. The
executor measures; this session decides whether the measurement is real. Every
part of the protocol below exists to make a wrong answer cheap to detect.

## Non-negotiables

| Rule | Operational meaning |
|---|---|
| AGY never closes | The executor may comment. Only this session closes, and only after independently recomputing the claim. `gh issue close` is never in a dispatch prompt. |
| No worktrees | All work happens in place in `/Users/chrischeng/axiom/project-mamba`. No `git worktree`, no branch switch, by anyone. |
| One ticket, one process | One `agy -p` per ticket. Never one prompt carrying two tickets — a merged report cannot be rejected by half. |
| Verify before the next batch | Batch N+1 does not dispatch until batch N is verified. The ramp is 2 → 4 → 8, cap 8 concurrent. Start serial when the channel is new or unstable. |
| The comment is the record | Chat summaries are not evidence. Where a chat summary and the issue comment disagree, the comment governs. |

## Environment invariants

Verified on this machine, 2026-07-26. Re-check after any agy upgrade.

| Fact | Consequence for the prompt |
|---|---|
| cwd is `~/.gemini/antigravity-cli/scratch`, **not** the repo | Absolute paths everywhere; `gh` always needs `-R <owner/repo>`. `--project` takes a project ID, not a path; `--add-dir` does not change cwd. |
| Prompt must be an argument | `agy -p "<text>"`. stdin is ignored → `Error: empty prompt`. |
| `--print-timeout` defaults to 5m | Real tickets run ~10 min. Set 30m. |
| Unlisted command → auto-denied, run aborts, **exit status 0** | Never triage on exit code. Triage on log content. |
| Chains are matched **segment-wise**, with no builtin exemption | `ls X ; echo done` is denied because `echo` is unlisted. One command per step. |
| **Command substitution `$(...)` is refused as a form** | Denied even when every tool inside it is allow-listed. `cat f \| xargs CMD` is the replacement. A ticket phase written with `$(...)` is an unrunnable ticket. |
| Shell variables are matched **unexpanded** | `BIN=<path>` then `$BIN --list` matches no rule and dies. Absolute path, every time. |
| A large inline `python3 -c` payload gets soft-denied | Cause unproven (single-line, multi-line, escaped quotes and `#`-comments all probe clean). Mitigation is unconditional: script file + `python3 /tmp/x.py`. |
| A denial discards the run but **not** the conversation | `./agy-wave.sh lastcmds <issue>` shows what it ran; `./agy-wave.sh resume <issue> "<correction>"` continues with all prior work intact. Do not re-dispatch from zero. |
| **Two** permission files are consulted | `~/.gemini/antigravity-cli/settings.json` → `permissions.allow`, and `~/.gemini/config/config.json` → `userSettings.globalPermissionGrants.{allow,deny}`. The global one ships with `command(cargo test)` and `command(cargo check)`. |
| `deny` beats `allow` | The only thing that actually closes the rebuild path. `lock` writes `command(cargo)` into the **global** deny list; `dispatch` refuses to start without it. |
| Redirection `>` and pipes work under an allow-rule | Ticket phases like `$BIN --list > all-tests.txt` need no rewriting. |

## The ritual

Run from `.claude/skills/agy-dispatch/`.

**1 — lock.** `./agy-wave.sh lock`

Two edits, because there are two permission files:

1. CLI settings — drop `command(cargo)`, `command(cargo check)`,
   `command(git push --force-with-lease)`; add the pinned test binary and the
   builtins agents reach for (`echo`, `printf`, `awk`, `cut`).
2. Global grants — add `command(cargo)`, `command(git push)`,
   `command(git commit)` to **`deny`**. This is the load-bearing half: the
   global file grants `command(cargo test)` independently of the CLI file, and
   `cargo test` is what rebuilds the binary. Deny beats allow.

`dispatch` refuses to start unless both halves are in place and the binary sha
matches. `lock` mutates global agy state — tell the user, and `unlock` when the
wave ends. Both originals are backed up next to the script.

**2 — snapshot the tree.** Before dispatching, record: `HEAD`, `git stash list`
count, the newest mtime under `projects/mamba/src` and `projects/mamba/tests`,
`CAPABILITIES.md` mtime, and the binary sha256. This is the only way to prove
afterwards that the executor touched nothing.

**3 — file the oracle.** Independently derive the expected answer, or an
expected band plus a hard floor, and write it to disk **before** the report
arrives. Record its sha256. An oracle computed after the fact is a
rationalisation, not a check.

A usable oracle names at least one **fabrication tell** — an outcome that is
impossible if the work was actually done. Example from #2642: a denominator for
"Scope, closures and cells" that omits the 8 tests ported from
`Lib/test/test_scope.py` cannot be honest, whatever its total.

**3b — read the Promise as a specification.** Before writing the ticket, check
whether the work root's `Gate / Evidence` cell names a **conjunction** —
"grammar *and* invalid-syntax", "parse-*transform*-compile", "assemble */*
codegen". Each conjunct is a separate surface, and a single selector reliably
covers one of them and silently misses the rest. Demand one branch per conjunct
and per-branch before/after counts.

This is the defect no group count can reveal, because the branch that was never
written forms no group. #2640 reproduced perfectly at 84 tests and was still
missing half its denominator: 30 of the 32 `test_grammar.py` tests, absent
because the selector keyed on an error token and grammar fixtures assert nothing
at all.

**4 — lint your own ticket body against your own prompt.** Every form the prompt
forbids, grep the body for:

    gh issue view <n> -R <repo> --json body -q .body \
      | grep -nE '\$\(|`|[A-Z_]+=|\$[A-Z_]+|\bcd \b|\bgit \b|\bcargo\b'

Anything that matches is a contradiction, and the executor resolves it against
you: a verbatim code block in the ticket outranks a prose prohibition in the
prompt, because STEP 2 says "execute exactly as written." #2640 died twice,
~20 minutes each with no output, on a `BIN=` assignment my prompt had banned
since v9 and my own PHASE 0 handed over as a copyable block. v13 adds the
precedence rule, but the rule is a backstop — a body that never contains the
banned form never tests it.

Same class, and it hides in acceptance criteria as readily as in phases: an AC
demanding `git status --porcelain` from an executor forbidden to run git is
unsatisfiable, and the honest executor stalls on it while the obliging one
violates a prohibition to close it out.

**5 — dispatch.** `./agy-wave.sh dispatch <issue> [<issue>...]`

Serial is one issue number. Parallel is several, capped at 8. Each gets its own
log under `runs/`. On completion the runner re-checks the binary sha and prints
`*** BINARY REBUILT` if a sibling broke the wave.

**6 — verify, then close.** Recompute every load-bearing number from source.
Post `## VERDICT`, then and only then `gh issue close`.

## Triage

`./agy-wave.sh status` classifies each run by log content:

| Verdict | Meaning | Action |
|---|---|---|
| `DENIED` | a command was refused | run the drill below; add the rule or rewrite the phase — do **not** reach for `--dangerously-skip-permissions` |
| `EMPTY` | timeout or crash, nothing reported | raise `AGY_TIMEOUT`, re-dispatch; the ticket is untouched |
| `reported` | a comment was posted | it is now **unverified input**, not a result |
| `finished without a comment URL` | ran, reported to stdout only | reject: chat is not the record |

**The denial drill** — a DENIED verdict names nothing by itself. Two steps, about
a minute, and it has found the cause every time:

1. `grep 'soft-denying' runs/<n>.agy.log` → the **step number** that died. A high
   step number means the work was done and only the last invocation was
   unrunnable — resume, do not restart.
2. Read that step's `permissions` blob out of
   `~/.gemini/antigravity-cli/conversations/<id>.db` (`agy-wave.sh lastcmds` dumps
   the command payloads) → the exact command text.

Then probe the suspected trigger in isolation with a throwaway
`agy -p 'Run exactly: …' --print-timeout 3m` before writing a clause. Two of the
four hypotheses this wave produced were wrong, and each probe cost 40 seconds.

## Verifying a report

Reject on any of these without further analysis:

| Tell | What it means |
|---|---|
| A number appears only in prose, with no re-runnable selector | not reproducible; cannot be checked, so it cannot be accepted |
| Shortlist size equals denominator size — **including within any one group** | the judgment predicate was never applied there; a group-level equality hides behind a healthy-looking total |
| An exclusion is keyed on the item's container (module, file, directory) | that is a list, not a predicate. Sample the discarded set for items whose own content names the target behaviour — false negatives are usually findable in one pass |
| The **candidate surface** is a typed list of paths, however good the predicate downstream | the same defect one stage earlier, and the only one no group count can reveal: a file never enumerated forms no group and prints no number. Check it against the oracle's hard floor, never against the run's own totals |
| Two tickets in one wave cite the **same** artifact path | a parallel-write collision; at most one of those selectors survives and the other ticket's number is unreproducible. Check paths across the batch, not per ticket — from inside one run the collision is invisible |
| The previous rejected attempt's number is restated | the thread was copied, not the corpus |
| Any hard-floor element from the oracle is missing | the artifact was not read |
| A repo file's mtime is newer than the snapshot | a prohibition was violated; the whole run is void |
| The binary sha changed | every measurement in the wave is incommensurable |
| **No ticket comment at all** | not a slow run. Read `agy-runs/<n>.log` first — a one-line `jetski: no output produced …` means a command hit an unlisted permission and the run died silently, possibly at minute 1 of a 20-minute wait. Check early; the wait loop cannot tell death from progress |

Once nothing above fires, **audit the set, not the code.** A predicate can be
per-item in form and still be fitted to the sample you handed over. Two scans,
both cheap, both independent of the executor's selector:

- **False positives** — for every admitted item, print the lines of its own body
  that mention the target behaviour, next to its assertions. An item whose
  assertions are all about something else is the textbook non-attributing case.
- **False negatives** — scan the **whole corpus**, not the executor's shortlist,
  for items that exhibit the behaviour structurally (a `global`/`nonlocal`
  *statement*, a `__closure__` access) and are absent from the result. The
  shortlist is the executor's construct; reusing it inherits its blind spot.

A `corpus_index.json` built once — one row per test with its metadata and body —
makes both scans seconds long and serves every later ticket's oracle.

**The hard floor is the only check that sees an omitted surface.** Group counts,
totals, and byte-identical reproduction are all computed *inside* the executor's
construction, so they stay healthy while a whole suite is missing. #2645 round 1
was internally consistent in every respect and still absent an entire canonical
CPython suite. Derive the floor from the corpus, never from the report.

**The cheapest form tell: does the attribution function ever mention the body?**
Grep the function that makes the final admission for the item's body field. If
`body`/`py_body`/`code` never appears inside it, no per-item judgment happened,
whatever the docstring above it says. #2648's predicate carried a docstring
reciting the specification verbatim and then tested `lib` and `fn_name`. One
grep, no analysis.

**Audit both sides: a drop is a claim and so is a keep.** Checking the
rejections is the natural instinct — they are enumerated, justified, and small.
Checking the admissions is the one that finds the false positives, and it is the
side a verdict silently certifies. #2640 round 1 verified all 24 drops, found
every one sound, said so, and passed `test_consts_in_conditionals`: admitted on
a `SyntaxError` token that lives only in the ported prologue helper the executed
tail never calls, while the tail asserts `dis` opcodes. Prioritise keeps whose stated
justification is a **token match** rather than an executed assertion; a fixture
prologue can manufacture those by the dozen. When the miss is yours, say so in
the verdict — an executor asked to re-audit a side you already certified deserves
to know why it is being asked twice.

**Zero drops is a failed predicate until proven otherwise.** A branch that
attributes N of N, with a rationale written per *group* rather than per item, has
reported a selector twice and a predicate never. Pair it with the body-grep tell
above: blanket rationale + zero drops + no `body` reference in the admission
function is three readings of the same fact. #2640 round 2 attributed 110/110;
30 did not flip on the promised subsystem, in three families a per-item pass
separates in minutes.

**A form defect is rejectable when it demonstrably changes the set.** When the
set survives an independent two-sided audit, the form defect becomes a *carried
finding* on the ticket that will encode the gate — not another round. #2647 and
#2648 committed the identical defect and got opposite verdicts: #2647's admitted
18 of 195 real candidates, #2648's picked exactly the tests a body predicate
would have picked. Reject the consequence, not the shape. Rejecting a correct
set on shape alone teaches obedience to the letter, which is the failure mode
directly below this one.

**After a rejection, expect the named examples to be patched in.** A verdict that
names the missing items hands over the shortest path to a passing count:
`if fn_name in ["exec_redirected", ...]`. Names in a verdict are instances, never
the specification — say so in the verdict itself, and check the next round's
selector for an identity list before checking anything else.

**Run the counterfactual BEFORE ordering a form change.** Replace the branch you
object to and see whether the set actually moves. #2647 round 3 was ordered to
swap a three-path admission for a body predicate; stripping the path branch
admitted **386** instead of 9, because in that corpus the two files *were* the
extension of the promise. The order could not have improved the answer. An order
like that costs a round and is indistinguishable, from the executor's side, from
one that could — three rejections deep it reads as "your shape is wrong", not
"your shape is wrong *and it matters*".

**Two routes, one number, is corroboration.** When a rejected round's set comes
back byte-identical from a materially different selector, that is the strongest
evidence a denominator ticket can produce — stronger than any single derivation.
Weigh it against a surviving form defect accordingly.

**Assertion-free tests need their own clause.** Some tests carry no assertion at
all; the oracle is "does not raise" (`x = eval('1, 0 or 1')` — the `eval_input`
grammar production). Every predicate keyed on what the assertions *reference* is
structurally blind to them, so a selector will reach them only by a fitted
container gate. Specify them up front or they become a carried finding every
time.

**The attributing question is which subsystem, if changed, flips the test.**
Fixing `compile()` does not flip the 40 `pattern_matching` tests that call it;
fixing the match compiler does. A test that uses X as the vehicle to assert Y's
contract attributes to Y. This one line settled every boundary dispute in the
wave — `compile` vs. PEP grammar, `==` vs. `set` semantics, `import_module` vs.
`uuid`.

**Accept a one-off impurity; reject a family.** Naming a single false positive
and quantifying its effect is auditable and cheap (#2642 shipped 1 in 29,
#2643 shipped a thin-corpus caveat). A family of false positives sharing one
systematic cause earns another round — the cause will recur in every later
ticket, and for a *denominator* the impurity is permanent: #2632 encodes it as a
durable gate, so it defines what the gate covers rather than merely mismeasuring
it.

**When an instruction names a field to match on, name the variable.** "match the
regex against its test path" cost a whole round of #2645: the executor applied it
to the path *and* the test name, which is a fair reading. A path, a module, and
a test name are three different fields.

Accepting is also a finding. Say which fields were recomputed and how, so the
acceptance itself is auditable. Name impurities you accepted and quantify their
effect on the gate; an acceptance that hides its known defects is not auditable.

## Improving the prompt

`prompt.tmpl` is the dispatch prompt; `agy-wave.sh` only substitutes
`{{ISSUE}} {{REPO}} {{ROOT}} {{BIN}} {{SHA}}`. Improvement is a closed loop:

1. A report is rejected, or an environment fact costs a round trip.
2. Name the **tell** that exposed it — and prefer a tell cheaper than redoing
   the work.
3. Add the narrowest clause to `prompt.tmpl` that makes that defect impossible
   to produce innocently. Bump the version line at the top.
4. If the defect was a **trap** — an environment fact, a shell form, a way a run
   dies silently — add or update its row in `GOTCHAS.md`, keyed by what you were
   about to do. If it was a **judgment** — how a set is derived, audited, or
   accepted — it belongs in "Verifying a report" above, not in a separate file.

   `GOTCHAS.md` is a lookup table, not a log: one row per trap, deduplicated,
   each carrying the date last observed. Re-probe the environment rows after an
   agy upgrade and **delete** the ones that no longer reproduce. A gotchas file
   that only grows has turned back into a log.

Rules for clauses:

- **Generic clauses go in `prompt.tmpl`; corpus-specific ones go in the ticket
  body.** "State every selector as a runnable command" is generic. "Select on
  the PEP 723 key first" belongs to the ported-suite tickets.
- **A clause must forbid a behaviour, not request a virtue.** "Be careful with
  regexes" changes nothing. "State every selector as a runnable command, so the
  supervisor can re-run it and get your exact set back" changes what gets
  written.
- **Prefer terminology to prose.** A named predicate constrains; an explanation
  invites interpretation. Pair the term with its operational definition and,
  where the failure has been seen, a counter-example.
- **Do not accumulate.** If a clause never catches anything across several
  batches, delete it and record the deletion. A prompt nobody finishes reading
  is worse than a short one.
