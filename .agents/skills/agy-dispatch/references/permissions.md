# AGY Project and permissions

Use this reference before creating or changing an AGY profile. It records only
public AGY surfaces verified against CLI 1.1.15 and Antigravity documentation;
it is not a schema for private `~/.gemini` registries or caches.

## Official surfaces

- [CLI Projects](https://antigravity.google/docs/cli/projects): launch an
  existing Project with `agy --project=<id>`; `--new-project` creates one.
- [AGY `/permissions`](https://antigravity.google/docs/cli/commands/permissions):
  its interactive scope picker exposes **Project**, **Shared**, and **Global**.
- [Permissions](https://antigravity.google/docs/permissions): rules are
  `action(target)`; command rules use token-prefix matching and the decision
  order is **Deny > Ask > Allow**.
- [Settings](https://antigravity.google/docs/settings): Global permissions are
  common defaults; Project Settings own folder boundaries, terminal policy,
  Outside of Folder File Access, sandbox, and Project exceptions.

The documented CLI Global settings file is
`~/.gemini/antigravity-cli/settings.json`, field `permissions`. It is the only
on-disk policy source `doctor` reads. Project policy storage and Project
discovery caches are not public APIs: never read them as proof and never edit
them. Do not use Computer Use, AppleScript, screen automation, or an internal
registry/cache mutation to provision AGY.

## Three scopes

| Layer | Owner | Purpose | Enforcement |
| --- | --- | --- | --- |
| Global | AGY Settings / `/permissions` Global | one reusable cross-repository baseline | AGY inherited policy |
| Project | persistent Project Settings / `/permissions` Project | rare repository-specific exceptions and file boundary | AGY Project policy |
| Task | controller profile | one ticket's byte-exact commands and repository writes | dispatcher snapshot/verify |

Project rules augment inherited Global rules. Across the AGY rules, apply the
documented **Deny > Ask > Allow** precedence. The dispatcher then applies the
narrower task layer first: a command absent from `task_commands.allow`, or
present in `task_commands.deny`, is rejected even if AGY would allow it.
`allowed_repo_writes`, protected-artifact hashes, snapshots, and diff budgets
remain independent write constraints.

## Baseline and exceptions

Put this reusable baseline in `global_permissions`:

- Allow: `pwd`, `rg`, `sed`, `shasum`, `git log`, `git status`, `git diff`,
  `git show`, `git rev-parse`, `git ls-files`, `git merge-base`, `uv`, and
  `python3`.
- Deny: Git mutation/publication (`add`, `commit`, `push`, `checkout`,
  `switch`, `reset`, `restore`, `stash`, `worktree`, `merge`, `rebase`,
  `cherry-pick`, `revert`, `clean`, `tag`, `update-ref`, `apply`, `am`, `rm`,
  `mv`); selected GitHub tracker/publication commands; and `rm -rf`.
- Ask: empty unless a human deliberately needs an interactive exception.

Do not deny `command(git)` or `git *`: it blocks safe read-only discovery.
Do not place build/test commands in the global skill baseline. Add a reusable
command to Global only when it is safe across repositories; otherwise add it
as an explicit Project exception, and still list the exact line in each task.

## Project/worktree binding

For one repository or persistent app worktree, use one AGY Project. A bounded
profile has three separate paths:

1. **Persistent Project root**: `agy_project_root`; stable app/repository
   worktree registered with AGY.
2. **Project-associated worktree**: the worktree selected in AGY Project
   Settings. It is normally the persistent root, not a ticket directory.
3. **Task linked worktree**: `root`; a clean linked Git worktree used once for
   this ticket. It must be a distinct exact worktree top level physically
   beneath `agy_project_root`, be ignored by that persistent root, and share
   its Git common directory.

Dispatch reuses the recorded Project id with `agy --project <id>` and starts
the process with `cwd=<root>`; it never creates a Project or attaches an
external sibling with `--add-dir`. Controller state stays outside both paths
under `/tmp/agy-dispatch/<project-id>/<task-key>/`.

## Formal discovery limitation and manual observation

As of CLI 1.1.15, `agy --help` exposes neither a `projects` listing command
nor a machine-readable effective Project-policy command. A headless controller
therefore cannot prove a Project's root, detect duplicates, or inspect Project
rules through a formal CLI/API. `doctor` must return `PROJECT_SETUP_REQUIRED`
until a human performs this official-UI sequence:

1. Open Antigravity **Select Project** and look for Projects containing the
   canonical `agy_project_root`.
2. If zero or more than one exists, stop. Resolve the intended Project manually;
   do not create/select/delete one from the dispatcher.
3. Confirm the selected Project id and its persistent root. Do not add a
   Project for the task linked worktree.
4. Open `/permissions` → **Global** and configure the profile's
   `global_permissions` baseline once.
5. Open the selected Project gear or `/permissions` → **Project**. Configure
   only `project_permissions` exceptions and set **Outside of Folder File
   Access** to **Always Deny**.
6. Copy the matching Project ids and the values displayed by the official UI
   into the local profile's `project_policy_observation`, then rerun `doctor`
   and `snapshot`.

`project_policy_observation` is a dated human observation, not a fabricated
machine proof. It binds the Project id/root, observed allow/deny/ask lists,
and file boundary to the snapshot so any profile change voids the run. Refresh
it after any Project-policy change. A future formal list/effective-policy API
may replace this human observation; do not emulate one through private files.

```json
"project_policy_observation": {
  "source": "official_project_ui_or_permissions",
  "observed_at": "2026-08-05T12:34:56Z",
  "project_id": "existing-persistent-project-id",
  "matching_project_ids": ["existing-persistent-project-id"],
  "project_root": "/absolute/persistent/app-root",
  "permissions": {"allow": [], "deny": [], "ask": []},
  "outside_of_folder_file_access": "always_deny"
}
```

## Diagnosing a denial

Run `doctor` before changing anything. Its `task_command_checks` show the
command, effective decision, matched rule, and source:

- `task_contract`: change the frozen task only when the command is genuinely
  needed; otherwise leave it denied.
- `global`: use official `/permissions` Global scope if the reusable baseline
  is incomplete or conflicting.
- `project`: use the selected Project's official Project scope for a
  repository-specific exception or conflict.
- `project-unobserved`: perform the manual observation above; no policy is
  inferred.
- `default`: AGY has no matching rule and would ask; do not blame Codex.

An Outside of Folder File Access failure is a Project Settings issue. A
`SANDBOX_COMMAND_BLOCKED`/`Operation not permitted` log is the per-run AGY
terminal sandbox, not a Global or Project permission request. A host launch
failure is reported separately as `controller_host`; it proves payload was not
sent and must not be repaired by changing AGY policy. Never silently fall back
to another worker.

## Migration

1. Stop active dispatches and finish/VOID their existing snapshots; a changed
   dispatcher or permission digest requires a fresh snapshot.
2. Copy the old identical Project baseline into `global_permissions` in each
   new profile and configure it once through the official Global UI.
3. Remove those shared rules from each Project through the official Project UI.
   Retain only documented repository exceptions and the Always Deny file
   boundary.
4. Fill `project_policy_observation`, run `doctor`, then take a new snapshot.
5. Do not modify a live task contract merely to inherit the new baseline; task
   commands and write envelopes remain exact and controller-owned.
