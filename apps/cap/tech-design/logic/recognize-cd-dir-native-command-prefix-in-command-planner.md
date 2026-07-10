---
id: recognize-cd-dir-native-command-prefix-in-command-planner
summary: Recognize the exact `cd <dir> && <tail>` shell prefix in cap's command planner, resolve `<dir>`, and re-plan `<tail>` through the existing native/pipe-fusion planner using the resolved directory as a per-invocation cwd override — falling back to bash unchanged for any disqualified `cd` segment or non-native `<tail>`.
fill_sections: [logic, unit-test, changes]
capability_refs:
  - id: command-lease-throttling
    role: primary
    gap: command-planner-cd-prefix-native-recognition
    claim: command-planner-cd-prefix-native-recognition
    coverage: partial
    rationale: "A new plan_cd_prefix recognizer extends the command planner's same-name native dispatch to the 'cd <dir> && <tail>' shape, the single largest concrete native-plan gap found in run-log analysis."
---

# TD: cap command planner cd-prefix native recognition

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: cap-cd-prefix-native-recognition
entry: plan_shell
nodes:
  plan_shell: { kind: start, label: "plan_shell(command) called with the raw bash -c string" }
  pipe_attempt: { kind: process, label: "existing pipe-fusion attempt over the whole string (unchanged)" }
  shell_free_attempt: { kind: decision, label: "whole string is shell-metachar-free and plannable? (unchanged)" }
  cd_split: { kind: process, label: "plan_cd_prefix: quote-aware scan for exactly one top-level '&&'" }
  cd_split_ok: { kind: decision, label: "exactly one top-level '&&' found (no stray '&', no second '&&')?" }
  cd_head_check: { kind: decision, label: "head has no shell metachars (has_shell_control_syntax) AND tokenizes to exactly ['cd', dir]?" }
  resolve_dir: { kind: process, label: "resolve dir: absolute as-is, else join with env::current_dir(); require it exists and is_dir()" }
  replan_tail: { kind: process, label: "recursively call plan_shell(tail) (reuses pipe-fusion + single-command planning unchanged)" }
  tail_native: { kind: decision, label: "tail replanned to CommandPlan::Native?" }
  wrap: { kind: process, label: "wrap inner.command in NativeCommand::WithCwd(Box::new(inner.command), resolved_dir); original = full 'cd ... && ...' string; reason names resolved_dir" }
  bash_fallback: { kind: terminal, label: "CommandPlan::External bash -c <original> unchanged (existing final fallback)" }
  native_result: { kind: terminal, label: "CommandPlan::Native(wrapped plan) returned" }
edges:
  - { from: plan_shell, to: pipe_attempt }
  - { from: pipe_attempt, to: native_result, label: "pipe fusion matched (unchanged path)" }
  - { from: pipe_attempt, to: shell_free_attempt, label: "no pipe match" }
  - { from: shell_free_attempt, to: native_result, label: "plannable (unchanged path)" }
  - { from: shell_free_attempt, to: cd_split, label: "not plannable (e.g. contains '&&')" }
  - { from: cd_split, to: cd_split_ok }
  - { from: cd_split_ok, to: bash_fallback, label: "zero or >1 top-level '&&', or stray '&'" }
  - { from: cd_split_ok, to: cd_head_check, label: "exactly one" }
  - { from: cd_head_check, to: bash_fallback, label: "disqualified: glob/var/;/||/second &&/not-cd/wrong arity" }
  - { from: cd_head_check, to: resolve_dir, label: "qualifies" }
  - { from: resolve_dir, to: bash_fallback, label: "dir missing or not a directory" }
  - { from: resolve_dir, to: replan_tail, label: "resolved" }
  - { from: replan_tail, to: tail_native }
  - { from: tail_native, to: bash_fallback, label: "tail itself needs bash (no partial execution)" }
  - { from: tail_native, to: wrap, label: "tail is native" }
  - { from: wrap, to: native_result }
---
flowchart TB
  plan_shell["plan_shell(command) called with the raw bash -c string"] --> pipe_attempt["existing pipe-fusion attempt over the whole string (unchanged)"]
  pipe_attempt -->|pipe fusion matched, unchanged path| native_result(["CommandPlan::Native(wrapped plan) returned"])
  pipe_attempt -->|no pipe match| shell_free_attempt{"whole string is shell-metachar-free and plannable? (unchanged)"}
  shell_free_attempt -->|plannable, unchanged path| native_result
  shell_free_attempt -->|not plannable, e.g. contains &&| cd_split["plan_cd_prefix: quote-aware scan for exactly one top-level '&&'"]
  cd_split --> cd_split_ok{"exactly one top-level '&&' found (no stray '&', no second '&&')?"}
  cd_split_ok -->|zero or >1, or stray &| bash_fallback(["CommandPlan::External bash -c <original> unchanged"])
  cd_split_ok -->|exactly one| cd_head_check{"head has no shell metachars AND tokenizes to exactly ['cd', dir]?"}
  cd_head_check -->|disqualified| bash_fallback
  cd_head_check -->|qualifies| resolve_dir["resolve dir: absolute as-is, else join with current_dir(); require exists+is_dir()"]
  resolve_dir -->|dir missing or not a directory| bash_fallback
  resolve_dir -->|resolved| replan_tail["recursively call plan_shell(tail) (reuses pipe-fusion + single-command logic)"]
  replan_tail --> tail_native{"tail replanned to CommandPlan::Native?"}
  tail_native -->|tail itself needs bash: no partial execution| bash_fallback
  tail_native -->|tail is native| wrap["wrap inner.command in NativeCommand::WithCwd(inner.command, resolved_dir); original = full string; reason names resolved_dir"]
  wrap --> native_result
```

The recognizer is a pure additive step inserted at the tail of `plan_shell`,
after the existing pipe-fusion attempt and the existing shell-metachar-free
single-command attempt both decline (both are unchanged and take priority,
since neither one currently accepts a string containing `&`). Only when both
decline does `plan_shell` call a new `plan_cd_prefix(original, label)` helper
before constructing its existing final `bash -c <original>` fallback; any
`None` from `plan_cd_prefix` falls straight through to that unchanged bash
construction, so behavior for every command line that is not this exact shape
is provably unaffected (R5).

`plan_cd_prefix` first runs a new quote-aware scanner (same single/double-quote
state-machine shape as the existing `has_shell_control_syntax` /
`split_simple_shell_words`) that looks for exactly one top-level `&&`
substring. A lone `&` not immediately followed by a second `&`, or a second
top-level `&&`, disqualifies the whole line immediately (R4's "more than one
`&&`" and "stray metacharacter" cases) and no split even happens for those
shapes. On exactly one match the string splits into `head` ("`cd `...") and
`tail` (the remainder).

The `head` segment is validated with two checks, both reusing existing
helpers rather than adding new metacharacter logic: `has_shell_control_syntax`
must be false (this alone rejects glob characters, `$` variables, `;`, `|`,
backtick, `~`, and any further `&`/`&&` inside the `cd` segment — exactly the
R4 disqualifier list), and `split_simple_shell_words(head)` must tokenize to
exactly two words with the first equal to the literal `"cd"`. Any failure
here (wrong arity, first word isn't `cd`, unterminated quote) disqualifies the
line and falls back to bash unchanged.

The qualifying `<dir>` token is resolved to an absolute path: used as-is if
already absolute, otherwise joined onto `std::env::current_dir()` (the same
value the resident light-shell session captured as its own `cwd` at session
start, since nothing in cap today mutates the process's actual working
directory — see `resident_shell.rs::ResidentLightShellSession::capture`).
The resolved path must exist and be a directory (`Path::is_dir()`); this
mirrors real `cd`'s failure behavior (`cd` failing means `&&` never runs the
tail) and is itself a planning failure that falls back to bash unchanged
rather than guessing (R3).

The `tail` segment is re-planned by recursively calling the same
`plan_shell(tail, label)` entry point used for top-level commands — this is
the reuse point that gives the recognizer pipe-fusion for free (e.g. `cd
<dir> && cat foo | wc -l`) without duplicating any planning logic. If the
recursive call returns `CommandPlan::External` (tail is not independently
native-plannable, for any reason the existing planner already has), the
recognizer returns `None` and the *original, unmodified* `cd <dir> && <tail>`
string falls back to bash as a whole — never a partial native-tail
execution with a stale `cd` prefix silently dropped (R3).

When the recursive call returns `CommandPlan::Native(inner)`, the recognizer
does not thread a new `cwd` field through the 270+ existing `NativePlan`
construction call sites. Instead it introduces one new recursive
`NativeCommand::WithCwd(Box<NativeCommand>, PathBuf)` variant that wraps
`inner.command` with the resolved directory, and builds the outer
`NativePlan` with `original` set to the *full* `cd <dir> && <tail>` string and
a `reason` that names the resolved absolute directory (satisfying AC1's
"resolved absolute path visible in ... reason"). `run_native_to`'s existing
big dispatch match is refactored (no behavior change to any existing arm)
into a `run_native_command(&NativeCommand, ...)` helper so the new
`WithCwd(inner, dir)` arm can temporarily `std::env::set_current_dir(dir)`,
recursively dispatch `inner` through the same helper, and restore the prior
working directory via an RAII guard before returning — a per-invocation
override of the process's single global cwd, not a persisted session-state
change, matching R2 and the explicit "no stateful cwd change" scope
boundary. Because the resident shell processes one command string at a time,
this transient global-cwd mutation is safe under the session's existing
single-invocation-at-a-time model; it is not safe under concurrent in-process
native execution, and that constraint is documented at the `WithCwd` variant
and the guard type rather than silently assumed.
