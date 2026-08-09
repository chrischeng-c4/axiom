# Codex adapter

The lifecycle this skill implements is in
`.claude/skills/agy-dispatch/references/lifecycle.md` and is not repeated here.
This file records only what `codex exec` supplies for each adapter operation,
what it supplies *better* than AGY, and the one hazard that makes per-round
isolation mandatory rather than a preference.

Measured against `codex-cli 0.146.0` at `/opt/homebrew/bin/codex`.

## Operation mapping

| # | lifecycle operation | Codex implementation |
|---|---|---|
| 1 | `derive` | `codex_dispatch.py worktree` (branch `codex/<task-key>`) |
| 2 | `bind` | `codex exec -C <worker_root>`, plus `[projects."<root>"] trust_level = "trusted"` in the round's config |
| 3 | `release` | `codex_dispatch.py discard`; nothing outside `state_dir` was mutated, so there is no previous state to restore |
| 4 | `inspect_effective_permissions` | `codex execpolicy check --rules <round.rules> -- <cmd>`, offline, one command at a time |
| 5 | `preflight` | `codex_dispatch.py doctor` |
| 6 | `snapshot` | `codex_dispatch.py snapshot` |
| 7 | `start` | `codex exec --json -o REPORT --output-schema SCHEMA` under a round-private `CODEX_HOME` |
| 8 | `resume` | `codex exec resume <thread_id>`; the id comes from the run log's `thread.started` event |
| 8a | `revise` | not implemented; a one-shot round that needs re-briefing is `discard`ed and re-dispatched |
| 9 | `poll` | the run is a foreground child of the dispatcher; the Bash tool's `run_in_background` owns liveness |
| 10 | `collect_audit` | `command_execution` items in the `--json` event stream |
| 11 | `normalize_report` | `--output-schema` constrains the final message; `-o FILE` writes it |
| 12 | `classify_failure` | process exit code plus `item.type == "error"` events |
| 13 | `classify_scope` | `codex_dispatch.py verify` — git status against the frozen complement |

## Where Codex is a better substrate than AGY

Three of the AGY adapter's weakest joints disappear.

**Permissions are a file, not persistent Project state.** AGY's `grant`
installs a permission set into a live Project, has to carry the Project's
inherited guards or silently revoke them (the defect that cost twenty deny rules
on one round), and has to restore them on `discard`. A Codex round generates
`<state_dir>/home/rules/round.rules` from `task_commands` and points `CODEX_HOME`
at it. There is no inheritance and nothing to restore.

**The permission surface can be tried before dispatch, not just described.**
`codex execpolicy check` evaluates one command against one rule file offline, so
`doctor` runs the round's own gate commands through it *and* runs one command
that must not be allowed. That negative direction is the only one that
distinguishes a real allowlist from an empty file; AGY's `doctor` can only read
the Project surface back and compare it to what it just wrote.

**The report shape is refused rather than requested.** AGY asks for an
`## EXEC REPORT` section in prose and parses it. `--output-schema` makes a
non-conforming final message impossible, so `verify`'s report check degenerates
to "did the file parse" — and a report that does not parse means the schema was
not applied, which is a dispatcher defect rather than a worker one.

One thing is worse: `codex exec` has no `-a/--ask-for-approval`. Only top-level
`codex` takes it. The sandbox mode and the rule file are the whole surface.

## `execpolicy check` result shapes

Both shapes matter, and they are distinguishable:

```
matched:    {"matchedRules":[{"prefixRuleMatch":{"matchedPrefix":["cargo","test"],
                              "decision":"allow"}}],"decision":"allow"}
unmatched:  {"matchedRules": []}
```

An unmatched command carries **no `decision` key at all**. Treating a missing
key as permissive would make an empty rule file indistinguishable from a
complete one, which is why `execpolicy_decision` returns `None` there and
`doctor` compares against `"allow"` explicitly rather than testing truthiness.

## Event stream

```
{"type":"thread.started","thread_id":"019fe547-…"}          ← the resume key
{"type":"item.completed","item":{"type":"error","message":…}}
{"type":"turn.started"}
{"type":"item.completed","item":{"type":"agent_message","text":…}}
{"type":"item.started","item":{"type":"command_execution","command":…,
                               "exit_code":null,"status":"in_progress"}}
{"type":"item.completed","item":{"type":"command_execution","command":…,
                                 "aggregated_output":…,"exit_code":0,
                                 "status":"completed"}}
{"type":"turn.completed","usage":{…}}
```

`command_execution.command` is the command line **as executed**, which is what
`verify` compares byte-for-byte against the allowlist — not what the model
intended to run. The distinction is the next section.

## Why the round needs its own `CODEX_HOME`

This is measured on this machine, not argued from principle.

The ambient `~/.codex/config.toml` installs a `PreToolUse` hook on `^Bash$`
running `cap hook bash --codex`. A probe asking Codex to run `echo hello` was
recorded as:

```
/bin/zsh -lc "/Users/chrischeng/.local/bin/cap run 'echo hello-from-codex'"
```

Under that config, no byte-exact command audit is possible: every authorized
line arrives at the audit rewritten. Beyond the hook, `~/.codex/rules/default.rules`
holds 82 accumulated `prefix_rule`s including `git add`, `git commit`,
`git push`, `git checkout`, `git rm`, `git stash`, `git rebase`, `gh pr`, and
`gh issue close` — the exact class of inherited broadening AGY's `doctor` has to
guard against, and which a round-private home simply does not have.

`--ignore-rules` is not the answer: it would drop the round's own rules too.
Isolation comes from `CODEX_HOME`, and the round home carries exactly three
things — a copy of `auth.json`, a generated `config.toml`, and
`rules/round.rules`. `doctor` refuses any other file under `rules/`, because
each one widens the worker past the declared surface.

Project trust lives in the config layer, and a derived worktree is a path that
home has never seen, so the generated config declares
`[projects."<worker_root>"] trust_level = "trusted"` explicitly. Without it the
first turn stalls on a trust decision with no human present.

## There is no non-shell read tool

Codex reads a file by running a command. There is no `Read` equivalent, so an
allowlist holding only the gate makes the worker blind, and a blind worker fills
`### Verified premises` with the brief's own claims restated as its observations.
That is why `make_profile.py` carries `--read-commands` and why `doctor` refuses
a bounded-write round that authorizes no prefix.

Prefixes cost something real: `--allow` is audited byte-for-byte, `--allow-prefix`
only up to the prefix. Keep prefixed commands read-only — anything that can also
write turns the audit into a formality.

Measured on round `r1`, dispatched with the gate as the only authorized command:
the worker reported it could not inspect the repository, and reproduced the
sha256 this dispatcher prints under "Frozen design inputs" as though it had
measured it. The value was correct and its provenance was the prompt.

## The write tool is a second audit channel

Writes do not arrive as `command_execution`. They arrive as:

```
{"type":"item.completed","item":{"type":"file_change","changes":[
   {"path":"/abs/path","kind":"add|update"}],"status":"completed"}}
```

`path` is absolute and unconstrained by the command allowlist. Auditing only
`command_execution` therefore misses every write, and `git status` misses any
write that lands outside the worktree. `scope_findings` reads both.

Two related traps in the git side of that check:

- `git status --porcelain` collapses an untracked directory to the directory
  name, so a deliverable at `<new-dir>/<file>` reports as `<new-dir>/` and fails
  an allowlist comparison keyed on file paths. Use `--untracked-files=all`.
- Prefix-matching an exact allowlist entry authorizes that command plus anything
  appended to it — `<gate> ; curl …` starts with `<gate>`. Exact entries must be
  compared for equality, after unwrapping `"/bin/zsh -lc '…'"`.

## Compound command lines

Codex evaluates a chain of safe operators per command, but a line carrying a
redirection, substitution, or environment assignment reaches the policy as a
single `["/bin/zsh","-lc","<script>"]` invocation. `rule_patterns` therefore
emits both forms for any allowlist entry containing shell metacharacters.
Emitting only the tokenized form authorizes the simple case and denies the
compound one mid-round, where it surfaces as a worker complaining about a
product defect.
