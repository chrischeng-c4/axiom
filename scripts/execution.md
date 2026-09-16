# Worktree executor

The controller creates one immutable `execution-assignment-v2` JSON file
outside the repository. The assignment contains no backend Project ID, model,
effort, permission, or sandbox setting.

The executor starts at the root of its assigned linked Git worktree. It uses
isolated `uv` commands. For example:

```sh
uv --cache-dir /tmp/axiom-execution-uv-cache run --isolated --no-project \
  python scripts/execute_assignment.py doctor \
  --assignment /absolute/controller/assignment.json
```

The lifecycle is `doctor`, `snapshot`, `dispatch`, `status`, and `verify`.
Ticketed work may use `resume` after the controller verifies the prior round.
The executor never changes the assignment. It never accepts or merges a result.

## CLI-only Project setup

The controller creates one persistent Project from the persistent repository
root. It runs `agy --new-project`, then uses the Project ID that the CLI shows.
It can reopen the Project with `agy --project=<id>`.

In the same CLI session, the controller runs `/permissions`. It reviews Global
scope and Project scope. It records the Project-scope observation once in
`~/.codex/execution/agy-projects.json` with schema
`execution-agy-registry-v2`.

The effective `allowNonWorkspaceAccess` value must be `false`. The CLI may omit
this key because `false` is its documented sparse default. The controller
preserves all other settings. The executor only reads this file.

Before `snapshot`, the controller installs one exact
`execution-assignment-guard` entry in one user-global AGY `hooks.json` file.
It executes one fixed user-local loader. The loader reads a private, short-lived
registration only when its worktree exactly matches an active executor task.
The executor never writes settings, Global hooks, Project files, or the task
worktree. Do not list `settings.json` as a protected artifact. AGY may rewrite
its false default away. The frozen permission-state digest binds the effective
value, Global rules, hook configuration, loader, and guard-core digests.

The only accepted loader entry is:

```json
{
  "execution-assignment-guard": {
    "PreToolUse": [{
      "matcher": "*",
      "hooks": [{
        "type": "command",
        "command": "exec /Users/<user>/.codex/execution/agy-assignment-guard-loader.py",
        "timeout": 10
      }]
    }]
  }
}
```

Install the loader and the matching fixed guard core with mode `0700` at:

```text
~/.codex/execution/agy-assignment-guard-loader.py
~/.codex/execution/agy-assignment-guard-core.py
```

The controller copies them from `scripts/_agy_global_guard_loader.py` and
`scripts/_agy_assignment_guard.py`. It can share the hooks file with other
reviewed Global hooks. The entry must occur exactly once and remain equivalent
after JSON parsing. The loader never selects a program through the environment.

If another active Global `PreToolUse` rule has the exact `run_command` matcher,
AGY selects that matcher before the wildcard entry. The controller must append
the fixed `~/.codex/execution/agy-assignment-guard-chain.py` command as that
matcher’s only hook. The fixed chain preserves the reviewed cap check and then
runs the fixed loader after cap allows. Doctor rejects the task until this
coverage exists. A cap deny can stop the fixed safe negative control before the
loader runs; raw stream verification accepts only that one specified denial.

AGY owns this cache:

```text
~/.gemini/antigravity-cli/cache/projects.json
```

Treat the cache as read-only and optional. A persistent-root or task-worktree
entry may be absent. Every present entry must use the observed Project ID.
Duplicate canonical roots stop execution. The executor never creates or writes
an ID, settings file, cache file, or registry file.

The registry observation is the Project ID authority. The executor reads the
matching Project config at:

```text
~/.gemini/config/projects/<project-id>.json
```

The config ID must match the observation. Its single official workspace
resource must resolve to the canonical persistent root. The resource may use
`folderUri` or `gitFolder.folderUri`. Non-file, relative, or hosted URIs stop
execution. The Project-config digest normalizes that URI and omits only known
CLI access timestamps. It binds all other identity and policy data.

AGY CLI 1.2.3 may not add a `projects.json` mapping for a repository that uses
Git's `extensions.worktreeConfig`. This is a CLI bootstrap issue. It is why the
Project config is authoritative and the cache is only a cross-check.

Missing, malformed, ambiguous, stale, or conflicting required data stops every
lifecycle verb. Existing task state also stops if the assignment, Project
config, relevant cache observation, registry entry, Project ID, persistent
root, or `agy --version` value changes.

## Headless evidence

Every dispatch enables `--sandbox` and `--output-format stream-json`. The first
event must be one `init`. Its absolute `cwd` must resolve to the assigned task
worktree. Its permission mode must equal the frozen settings observation. The
final event must be one successful `result`. New and resumed conversation IDs
must agree.

Before AGY starts, the executor copies the guard program and policy into its
run directory under `/tmp`. It creates one private registration that binds the
exact task worktree, policy, audit path, and fixed core digest. The fixed global
loader finds that registration from the hook payload. It checks the exact
assignment, model, worktree, command, command working directory, and Dev write
path before each tool runs. QA write tools are denied. Unlisted tools are
denied. No repository hook file is created or changed.

The run evidence binds the guard program, policy, static global-hook loader
contract, and one audit record for every tool request that reaches the loader.
The fixed safe canary is the only exception. An earlier reviewed Global deny
can stop the later loader before it receives that canary request. In that case,
`verify` accepts no guard record only when raw stream data proves that exact
canary was denied before execution. A missing audit record for every other tool
request stops verification. A missing loader or an altered loader stops at
`doctor`; an altered hook state after snapshot stops verification.

The executor saves raw stdout NDJSON, stderr diagnostics, the AGY log, the final
response, the normalized `## EXEC REPORT`, and evidence digests separately.
`verify` reparses the raw stream and stderr. A permission denial is
`soft-denied`, even when AGY exits with code `0`.

A measure-only isolation canary may set
`task_contract.expected_soft_denied_commands`. Each exact command must also be
in `task_commands.allow`. The only accepted canary is the safe, read-only
`git clean -nd -- .` command. Preflight requires the persistent policy to
resolve it to Deny. The global assignment guard also denies it before
execution. AGY can report either the linked worktree or the durable Project
root in hook `workspacePaths`. The guard permits only those frozen roots. It
still requires every command Cwd to equal the linked worktree. The prompt
requests the canary once after all normal commands and never retries it.
AGY can call the hook more than once for one streamed tool step. Each audit
record must still match that one stream step and the frozen policy exactly.
The stream can omit command defaults that the hook receives. Verification
binds an omitted command Cwd to `init.cwd`, then checks the command, Cwd,
policy decision, and reason. It retains the hook argument digest as evidence
but does not pretend the abbreviated stream can reproduce that digest.
For the negative control, the local conversation database proves the single
request. The raw stream ERROR plus the frozen assignment-guard denial prove
that AGY did not execute it. The database status value alone is not used to
classify a pre-tool hook denial.

Transient evidence is stored at
`/tmp/execution/agy/<project-id>/<task-key>/`. Measure-only tasks may share one
Project lock. A bounded-write task owns the exclusive Project lock through
verification.
