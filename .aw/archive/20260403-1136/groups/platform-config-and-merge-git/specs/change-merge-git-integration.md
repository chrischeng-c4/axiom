---
id: change-merge-git-integration
type: spec
title: "Change Merge — Logic"
version: 3
files:
  - mcp/tools/change_merge/create.rs
  - workflow/merge.rs
  - prompts/review_archive.md
main_spec_ref: "crates/cclab-sdd/logic/change-merge.md"
merge_strategy: extend
fill_sections: [overview, requirements, scenarios, logic, config, changes]
filled_sections: [overview, requirements, scenarios, logic, config, changes]
create_complete: true
---

# Change Merge

## Phase Transition

```yaml
from: ChangeImplementationReviewed (all approved)
to: ChangeArchived
executor: [mainthread]
crr: false  # programmatic merge, no CRR
```

## Merge Logic

`sdd_workflow_create_change_merge` is **fully programmatic** — no agent needed.

```mermaid
flowchart TD
    Start([workflow_create_change_merge]) --> FindSpecs[find specs in changes/{id}/groups/*/specs/]
    FindSpecs --> Empty{specs found?}
    Empty -->|no| ArchiveEmpty[archive with no merge]
    Empty -->|yes| Loop[for each spec file]
    Loop --> ReadFM[read frontmatter: main_spec_ref]
    ReadFM --> UsePath[target = cclab/specs/main_spec_ref]
    UsePath --> Strip[strip change-spec-only frontmatter fields]
    Strip --> Write[write to cclab/specs/{target}]
    Write --> Loop
    Loop --> Done[all merged]
    Done --> Archive[phase → ChangeArchived]
    Archive --> Move[rename changes/{id} → archive/{date}-{id}]
```

## Frontmatter Stripping

Change-spec-only fields removed before writing to main specs:

```yaml
stripped_fields:
  - main_spec_ref      # only used for merge routing
  - merge_strategy     # only used during merge
  - create_complete    # internal marker
  - fill_sections      # internal tracking
  - filled_sections    # internal tracking
```

## Merge Strategy

From spec frontmatter `merge_strategy`:

| Strategy | Behavior |
|----------|----------|
| `new` | Create new file at `cclab/specs/{main_spec_ref}` |
| `update` | Overwrite existing file at `cclab/specs/{main_spec_ref}` |

## Archive

After merge:
- Phase set to `ChangeArchived`
- Tool moves change dir to `cclab/archive/{YYYYMMDD}-{change_id}` programmatically via `std::fs::rename`
- Response includes `archive_path` for caller reference

## Side Effects

| Action | STATE.yaml change |
|--------|-------------------|
| Merge all specs | Spec files written to `cclab/specs/` |
| Complete | `phase → ChangeArchived` |
| No specs to merge | `phase → ChangeArchived` (skip merge) |


## Changes

## Changes
<!-- type: changes lang: yaml -->

```yaml
files:
  - path: crates/cclab-sdd/src/tools/create_change_merge.rs
    action: MODIFY
    desc: |
      Add post-archive git operations after existing archive/rename logic.

      Add function: post_archive_git_ops(project_root, change_id, archive_path, repo_platform_config) -> GitOpsResult
        1. Guard: if !auto_commit, return early (no-op)
        2. Guard: find_git_binary() — if None, return warning
        3. Run: git status --porcelain -- cclab/ from project_root
        4. Guard: if no dirty paths, return git_commit_sha: null
        5. Run: git add {dirty_paths}
        6. Build commit message: read user_input.md from archive_path, truncate to 72 chars
        7. Run: git commit -m "chore(sdd): merge {change_id} — {summary}"
        8. Extract SHA from git output
        9. If auto_pr: dispatch agent for PR body generation, then gh pr create
        10. Return GitOpsResult { git_commit_sha, pr_url, git_warning }

      Modify: create_change_merge() to call post_archive_git_ops() after fs::rename,
      merge GitOpsResult fields into tool response JSON.

  - path: crates/cclab-sdd/src/models/change.rs
    action: MODIFY
    desc: |
      Add RepoPlatformConfig struct to SddConfig.

      Add struct: RepoPlatformConfig { type_: String, repo: String, default_branch: String, auto_commit: bool, auto_pr: bool }
      Add field to SddConfig: repo_platform: Option<RepoPlatformConfig>
      Defaults: auto_commit=false, auto_pr=false, default_branch="main"
      Serde: #[serde(default)] on repo_platform field so missing section → None

  - path: crates/cclab-sdd/src/tools/create_change_merge.rs
    action: MODIFY
    desc: |
      Add GitOpsResult struct.

      struct GitOpsResult {
        git_commit_sha: Option<String>,
        pr_url: Option<String>,
        git_warning: Option<String>,
      }

      Extend existing tool response to include these 3 fields.
```
## Changes

## Changes
<!-- type: changes lang: yaml -->

```yaml
files:
  - path: crates/cclab-sdd/src/tools/create_change_merge.rs
    action: MODIFY
    desc: |
      Add post-archive git operations after existing archive/rename logic.

      Add function: post_archive_git_ops(project_root, change_id, archive_path, repo_platform_config) -> GitOpsResult
        1. Guard: if !auto_commit, return early (no-op)
        2. Guard: find_git_binary() — if None, return warning
        3. Run: git status --porcelain -- cclab/ from project_root
        4. Guard: if no dirty paths, return git_commit_sha: null
        5. Run: git add {dirty_paths}
        6. Build commit message: read user_input.md from archive_path, truncate to 72 chars
        7. Run: git commit -m "chore(sdd): merge {change_id} — {summary}"
        8. Extract SHA from git output
        9. If auto_pr: dispatch agent for PR body generation, then gh pr create
        10. Return GitOpsResult { git_commit_sha, pr_url, git_warning }

      Modify: create_change_merge() to call post_archive_git_ops() after fs::rename,
      merge GitOpsResult fields into tool response JSON.

  - path: crates/cclab-sdd/src/models/change.rs
    action: MODIFY
    desc: |
      Add RepoPlatformConfig struct to SddConfig.

      Add struct: RepoPlatformConfig { type_: String, repo: String, default_branch: String, auto_commit: bool, auto_pr: bool }
      Add field to SddConfig: repo_platform: Option<RepoPlatformConfig>
      Defaults: auto_commit=false, auto_pr=false, default_branch="main"
      Serde: #[serde(default)] on repo_platform field so missing section → None

  - path: crates/cclab-sdd/src/tools/create_change_merge.rs
    action: MODIFY
    desc: |
      Add GitOpsResult struct.

      struct GitOpsResult {
        git_commit_sha: Option<String>,
        pr_url: Option<String>,
        git_warning: Option<String>,
      }

      Extend existing tool response to include these 3 fields.
```
## Overview

## Overview
<!-- type: overview lang: markdown -->

Extend the merge workflow (`sdd_workflow_create_change_merge`) with post-archive git operations. After `ChangeArchived` phase is set and the change directory is moved to archive, two optional steps execute in sequence:

1. **Auto git commit** (`repo_platform.auto_commit = true`): detect all dirty paths under `cclab/`, stage them, and commit with conventional message format `chore(sdd): merge {change_id} — {description}`.
2. **Auto PR** (`repo_platform.auto_pr = true`): dispatch an agent to generate a PR body from change context (user_input.md, spec summaries), then create a PR to `repo_platform.default_branch` via `gh` CLI.

Both steps are controlled by `[sdd.repo_platform]` config in `cclab/config.toml`. Git binary availability is detected at runtime — if git is unavailable, a warning is returned and git steps are skipped (non-fatal).

The existing merge logic (spec write, frontmatter stripping, validation, audit logging, archive) is unchanged. Git operations are appended as a post-archive extension point.


## Requirements

## Requirements
<!-- type: requirements lang: markdown -->

| ID | Title | Priority | Acceptance Criteria |
|----|-------|----------|---------------------|
| R1 | Auto git commit after archive | P0 | After `ChangeArchived` is set and directory moved to archive, if `repo_platform.auto_commit = true`: (1) run `git status --porcelain` under `cclab/` to detect dirty paths, (2) `git add` all dirty paths, (3) `git commit` with message `chore(sdd): merge {change_id} — {description}` where description comes from `user_input.md`. Return `git_commit_sha` in tool response. If no dirty paths, skip commit silently and return `git_commit_sha: null`. |
| R2 | Auto PR creation via agent dispatch | P1 | If `repo_platform.auto_pr = true` AND a commit was created (R1): dispatch an agent to generate a PR body from change context (user_input.md, spec summaries, archived specs list), then create PR to `repo_platform.default_branch` using `gh pr create`. Return `pr_url` in tool response. `auto_pr = true` requires `auto_commit = true` — if `auto_commit = false`, log warning and skip PR. |
| R3 | repo_platform config section | P0 | New `[sdd.repo_platform]` section in `cclab/config.toml` with required fields: `type` (string, e.g. "github"), `repo` (string, "owner/repo" — required, no fallback), `default_branch` (string, default "main"), `auto_commit` (bool, default false), `auto_pr` (bool, default false). Parsed by `RepoPlatformConfig` struct loaded via `SddConfig`. |
| R4 | Git binary detection and graceful fallback | P0 | Reuse existing `find_git_binary()` from `create_change_merge.rs`. If git binary not found when `auto_commit = true`, return warning in response (`git_warning: "git binary not found, skipping auto-commit"`) and skip all git operations. Non-fatal — merge itself succeeds. |
| R5 | Conventional commit message format | P1 | Commit message format: `chore(sdd): merge {change_id} — {summary}` where summary is the first line of `user_input.md` (truncated to 72 chars). If `user_input.md` is missing or empty, use `chore(sdd): merge {change_id}`. |
| R6 | Stage dirty paths under cclab/ | P0 | Detect dirty paths by running `git status --porcelain -- cclab/` from project root. Stage ALL reported paths (modified, untracked, deleted). Do not filter to specific subdirectories — stage everything dirty under `cclab/`. |

### Constraints

- Git operations execute AFTER archive move (`std::fs::rename`) — the change dir no longer exists at `cclab/changes/{id}`
- `auto_pr` without `auto_commit` is a config validation warning, not a hard error
- PR agent dispatch uses the existing `dispatch_agent()` mechanism from `sdd_run_change`
- All git operations run synchronously in the `create_change_merge` tool — no background tasks
- `repo_platform.repo` is required with no fallback to `issue_platform.repo` (per clarification Q3)


## Scenarios

## Scenarios
<!-- type: scenarios lang: markdown -->

### S1: Auto-commit with dirty files (R1, R5, R6)

**GIVEN** `repo_platform.auto_commit = true`, merge completes successfully, and `git status --porcelain -- cclab/` reports 3 modified files (specs, archive, changes removal)
**WHEN** post-archive git operations execute
**THEN** all 3 files are staged via `git add`, commit is created with message `chore(sdd): merge 1136 — feat(sdd): platform config restructure`, and `git_commit_sha` is returned in tool response.

### S2: Auto-commit disabled — no git operations (R1)

**GIVEN** `repo_platform.auto_commit = false` (or `[sdd.repo_platform]` section absent)
**WHEN** merge completes and archive move succeeds
**THEN** no git commands are executed. Response contains `git_commit_sha: null`, `pr_url: null`. Merge result is identical to current behavior.

### S3: Auto-commit with no dirty files (R1, R6)

**GIVEN** `repo_platform.auto_commit = true` but `git status --porcelain -- cclab/` reports no changes
**WHEN** post-archive git operations execute
**THEN** commit is skipped silently. Response contains `git_commit_sha: null`. No error.

### S4: Auto-PR after successful commit (R1, R2)

**GIVEN** `repo_platform.auto_commit = true`, `repo_platform.auto_pr = true`, commit succeeds with SHA `abc123`
**WHEN** PR creation step executes
**THEN** agent is dispatched to generate PR body from change context, `gh pr create` runs with generated title and body targeting `repo_platform.default_branch`, `pr_url` is returned in response.

### S5: Auto-PR without auto-commit — warning (R2)

**GIVEN** `repo_platform.auto_commit = false`, `repo_platform.auto_pr = true`
**WHEN** post-archive git operations execute
**THEN** PR step is skipped. Response includes `git_warning: "auto_pr requires auto_commit — skipping PR creation"`. No error — merge itself succeeds.

### S6: Git binary not available — graceful fallback (R4)

**GIVEN** `repo_platform.auto_commit = true` but `find_git_binary()` returns `None`
**WHEN** post-archive git operations execute
**THEN** all git operations are skipped. Response includes `git_warning: "git binary not found, skipping auto-commit"`. Merge result (spec write + archive) is unaffected.

### S7: Commit message truncation (R5)

**GIVEN** `user_input.md` contains a 150-character first line
**WHEN** commit message is constructed
**THEN** summary is truncated to 72 characters: `chore(sdd): merge {id} — {first_72_chars_of_summary}`.

### S8: Missing user_input.md — fallback message (R5)

**GIVEN** `user_input.md` does not exist or is empty in the archived change directory
**WHEN** commit message is constructed
**THEN** fallback message is used: `chore(sdd): merge {change_id}` (no description suffix).

### S9: repo_platform config absent — default behavior (R3)

**GIVEN** `cclab/config.toml` has no `[sdd.repo_platform]` section
**WHEN** merge workflow loads config
**THEN** `RepoPlatformConfig` defaults to `auto_commit = false`, `auto_pr = false`. No git operations execute. Functionally identical to pre-change behavior.


## Logic

## Post-Archive Git Operations
<!-- type: logic lang: mermaid -->

Extension to the merge flowchart. Executes after the existing Archive → Move step.

```mermaid
flowchart TD
    Start([workflow_create_change_merge]) --> FindSpecs[find specs in changes/{id}/groups/*/specs/]
    FindSpecs --> Empty{specs found?}
    Empty -->|no| ArchiveEmpty[archive with no merge]
    Empty -->|yes| Loop[for each spec file]
    Loop --> ReadFM[read frontmatter: main_spec_ref]
    ReadFM --> Validate{main_spec_ref has subfolder?}
    Validate -->|no| Error[hard error: root-level path rejected]
    Validate -->|yes| UsePath[target = cclab/specs/main_spec_ref]
    UsePath --> Strip[strip change-spec-only frontmatter fields]
    Strip --> Exists{file exists at target?}
    Exists -->|yes| LogOverwrite[audit log: overwrite]
    Exists -->|no| LogCreate[audit log: create]
    LogOverwrite --> Write[write to cclab/specs/{target}]
    LogCreate --> Write
    Write --> Loop
    Loop --> Done[all merged]
    Done --> Archive[phase → ChangeArchived]
    Archive --> Move[rename changes/{id} → archive/{date}-{id}]
    ArchiveEmpty --> Move
    Move --> LoadConfig[load repo_platform config]
    LoadConfig --> CheckAutoCommit{auto_commit?}
    CheckAutoCommit -->|false| SkipGit[return: no git ops]
    CheckAutoCommit -->|true| FindGit{git binary available?}
    FindGit -->|no| WarnNoGit[warning: git not found]
    FindGit -->|yes| GitStatus[git status --porcelain -- cclab/]
    GitStatus --> HasDirty{dirty paths?}
    HasDirty -->|no| SkipCommit[return: no changes to commit]
    HasDirty -->|yes| GitAdd[git add dirty paths]
    GitAdd --> GitCommit[git commit -m 'chore sdd: merge {id} — {desc}']
    GitCommit --> CheckAutoPR{auto_pr?}
    CheckAutoPR -->|false| ReturnSHA[return: git_commit_sha]
    CheckAutoPR -->|true| DispatchAgent[dispatch agent: generate PR body]
    DispatchAgent --> CreatePR[gh pr create → default_branch]
    CreatePR --> ReturnAll[return: git_commit_sha + pr_url]
    WarnNoGit --> SkipGit
```


## Config

## Config: repo_platform Fields Used by Merge
<!-- type: config lang: json -->

Subset of `[sdd.repo_platform]` config relevant to the merge git integration. Full schema defined in `platform-config-repo-spec`.

```json
{
  "$id": "repo-platform-merge-config",
  "type": "object",
  "properties": {
    "auto_commit": {
      "type": "boolean",
      "default": false,
      "description": "Stage all dirty paths under cclab/ and commit after ChangeArchived"
    },
    "auto_pr": {
      "type": "boolean",
      "default": false,
      "description": "Create PR to default_branch after auto-commit. Requires auto_commit=true."
    },
    "default_branch": {
      "type": "string",
      "default": "main",
      "description": "Target branch for auto-PR creation"
    },
    "repo": {
      "type": "string",
      "pattern": "^[\\w.-]+/[\\w.-]+$",
      "description": "GitHub owner/repo — required, no fallback to issue_platform.repo"
    }
  },
  "required": ["repo"],
  "x-sdd": {
    "refs": [
      {"$ref": "platform-config-repo-spec#repo-platform-config"}
    ]
  }
}
```

### Merge Tool Response Schema Extension

```json
{
  "$id": "merge-git-response",
  "type": "object",
  "properties": {
    "git_commit_sha": {
      "type": ["string", "null"],
      "description": "SHA of auto-commit, null if skipped"
    },
    "pr_url": {
      "type": ["string", "null"],
      "description": "URL of created PR, null if skipped"
    },
    "git_warning": {
      "type": ["string", "null"],
      "description": "Warning message if git ops skipped (binary missing, config mismatch)"
    }
  }
}
```

# Reviews
