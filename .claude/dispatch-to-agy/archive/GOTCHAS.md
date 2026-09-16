# GOTCHAS — traps in this channel, keyed by what you are about to do

Look up the row for the thing you are about to do. Each row is a trap that has
actually fired, with the replacement that works.

**Retiring a row is the maintenance.** Every row carries the date it was last
observed. After an `agy` upgrade, re-probe the environment rows — a trap that no
longer reproduces gets **deleted**, not archived. This file is meant to get
smaller. If it only ever grows, it has turned back into a log.

Scope note: rows marked **(me)** fired in the supervisor's own shell, not the
executor's. The channel's asymmetry is authority, not accuracy.

---

## Shell forms that end an agy run with no output

A denied segment aborts the run: no output, no error, no ticket comment. These
die even when every tool inside them is allow-listed.

| About to write | What happens | Use instead | Seen |
|---|---|---|---|
| `<cmd> $(cat f)` | command substitution is refused **as a form** | `cat f \| xargs <cmd>` | 07-26 |
| `BIN=/p && $BIN …` | variables are matched unexpanded, so nothing matches | spell the absolute path every time | 07-26 |
| `python3 -c '<big payload>'` | soft-denied above some size; cause never established | `write_to_file` a script, then `python3 /tmp/awNNNN/x.py` | 07-26 |
| `cat <<'EOF' > f` | heredoc denied | `write_to_file` — a native tool, so the Bash matcher never sees it | 07-27 |
| `ls X ; echo done` | matched **per segment**, no builtin exemption; `echo` unlisted → dies | one command per line | 07-26 |

Redirection `>` and pipes are fine under an allow-rule. `env`, and any form of
setting a variable before a command, are not.

**The shell has no permitted way to author a multi-line file.** That is why
`write_to_file` + `python3 <path>` is the shape of every non-trivial step, not a
stylistic preference. A prohibition with no permitted path is a trap, not a rule
— when adding one, name the permitted route in the same breath.

## Permissions

| Situation | Trap | Do | Seen |
|---|---|---|---|
| removing `command(cargo)` from CLI settings | `~/.gemini/config/config.json` **independently** grants `command(cargo test)`, and that is what rebuilds the binary | add `command(cargo)` to the **global deny** list; deny beats allow | 07-26 |
| writing the prompt's tool list by hand | prompt advertised 40 tools, settings granted 23; the executor plans around tools it cannot run | derive the list from settings | 07-27 |

Two files are consulted. `lock` edits both and mutates **global** agy state —
tell the user, and `unlock` when the wave ends.

## Reading a finished run

| Signal | Means | Next | Seen |
|---|---|---|---|
| exit status `0` | nothing — a denied run also exits 0 | never triage on exit code; triage on log content | 07-26 |
| `soft-denying … at step N`, N large | the work was done; only the last invocation was unrunnable | `resume`, never re-dispatch from zero | 07-26 |
| no ticket comment after a long wait | possibly died at minute 1 | read `agy-runs/<n>.log` **early**; `jetski: no output produced` = silent death | 07-27 |
| a comment exists | unverified input, not a result | recompute every load-bearing number from source | — |

## Wave integrity

| Situation | Trap | Do | Seen |
|---|---|---|---|
| pinning the test binary | a path inside `target/` is not pinned — any sibling's `cargo` replaces it | stage the copy **outside** `target/` and check its sha after every run | 07-27 |
| parallel tickets | two tickets naming the same `/tmp` artifact path — second writer wins silently, and from inside one run the collision is invisible | check artifact paths **across the batch** | 07-26 |

## Writing the ticket

| In the body | Trap | Do | Seen |
|---|---|---|---|
| any banned shell form, even in a code block | the body outranks a prose prohibition — the executor is right to follow it | lint before filing: `grep -nE '\$\(\|[A-Z_]+=\|\bcd \b\|\bgit \b\|\bcargo\b'` | 07-26 |
| an AC demanding `git status` | unsatisfiable for an executor forbidden `git`; the honest one stalls, the obliging one violates a guardrail | mark it `NOT SELF-VERIFIED (supervisor check)` | 07-26 |
| a path that resolves to nothing | the search that follows a dead path is where runs die | ticket defect: report and move on | 07-26 |
| naming the missed items in a rejection | they get patched in as an identity list | say in the verdict that names are instances, never the specification; check the next selector for an identity list first | 07-26 |
| "match the regex against its path" | a path, a module, and a test name are three fields | name the variable | 07-26 |

## Reading the mamba fixture corpus

| About to do | Trap | Do | Seen |
|---|---|---|---|
| extract a fixture's Python body with `r[#]*"(.*?)"[#]*` | **not delimiter-aware.** Fixtures open with `r###"`, and the PEP 723 header supplies a `"` ~33 chars in, so the non-greedy body stops there. 12,647 of 13,767 tests were matched against a 33-character header fragment instead of the real body — and the 8 hits returned were just the fixtures whose raw strings happen to carry no early quote | select on the structured PEP 723 key first; report shortlist size **before and after** the attribution predicate | 07-25 |

Full fixture mechanics:
`projects/mamba/tests/harness/cpython/conventions/FIXTURE-LAYOUT.md`.

## The supervisor's own shell **(me)**

| About to write | What happens | Use instead | Seen |
|---|---|---|---|
| `$(...)` anywhere | cap guard rejects the whole command | separate calls | 07-27 |
| a heredoc | same | Write tool, then `python3 <path>` | 07-27 |
| `timeout …` | not installed on this machine | omit it | 07-26 |
| `open(p,'w').write(t + open(q).read())` | the call target is evaluated **before** the argument: `p` is truncated, then the read raises | read first; append with mode `'a'`; `wc -l` after every write | 07-27 |
| `2>/dev/null` on a command that writes | discards the only evidence that it failed | never on a write | 07-27 |
| `except: continue` around `open()` | a path error becomes a clean zero — 144 files "scanned", 0 found | assert `UNREADABLE: 0` before counting | 07-27 |
| a repo-root-relative path list | audit logs are repo-root-relative; running from a subdir opens nothing | run from the repo root | 07-27 |
| `grep -m1 X \| cut … \|\| echo ABSENT` | a pipeline's exit status is the **last** command's, so the fallback can never fire | do it in python | 07-27 |
| an untracked file you append to for days | no history, no backup, outside every safety net | keep a copy, or accept it is scratch | 07-27 |

**Run the control.** A checker that returns the same answer for a set and for a
set known to contain a positive is not measuring anything. One extra invocation.
Both silent-zero rows above were caught this way and by nothing else.
