---
id: platform-config-repo-spec
main_spec_ref: "crates/cclab-sdd/config/platform.md"
merge_strategy: extend
fill_sections: [overview, requirements, scenarios, config, changes]
filled_sections: [overview, requirements, scenarios, config, changes]
create_complete: true
---

# cclab sdd platform

Issue tracking platform configuration CLI. Supports GitHub, GitLab, and Jira.

## Overview

## Overview
<!-- type: overview lang: markdown -->

Extend `cclab/config.toml` platform configuration from a single `[sdd.issue_platform]` section to a multi-platform architecture with four distinct sections:

| Section | Purpose | Status |
|---------|---------|--------|
| `[sdd.issue_platform]` | Issue tracking (GitHub/GitLab/Jira) | Existing — unchanged |
| `[sdd.repo_platform]` | Git repo, commits, PRs | New |
| `[sdd.spec_platform]` | Spec storage location | New |
| `[sdd.docs_platform]` | User-facing documentation | Future — commented out |

`repo_platform` enables post-merge git operations (auto-commit, auto-PR) consumed by `change-merge-git-integration`. `spec_platform` declares spec storage (currently `type = "local"` only). `docs_platform` is reserved for future use.

Parsing changes:
- `SddSection` in `config.rs` gains `repo_platform`, `spec_platform` fields
- `SddConfig` in `models/change.rs` gains `repo_platform: Option<RepoPlatformConfig>`, `spec_platform: Option<SpecPlatformConfig>`
- `platform show` CLI displays all configured platform sections
- `repo_platform.repo` is required (no fallback to `issue_platform.repo`)
## Commands

### `cclab sdd platform set`

Interactive wizard to configure the issue tracking platform.

**Flow**:
1. Select platform: GitHub (1) / GitLab (2) / Jira (3)
2. Platform-specific prompts:
   - **GitHub/GitLab**: Auto-detect repo from git remote, choose CLI OAuth or API token auth
   - **Jira**: Prompt for URL, project key, API token, and email
3. Write `[sdd.issue_platform]` section to `cclab/config.toml`

**Prerequisite**: `cclab/config.toml` must exist (run `cclab init` first).

### `cclab sdd platform show`

Display current platform configuration.

**Load priority**:
1. `[sdd.issue_platform]` (new namespaced path)
2. `[platform]` (legacy fallback)
3. "No platform configured" message

Shows: source section, type, repo/URL/project, auth method, and auth details.

## Configuration

### New format: `[sdd.issue_platform]`

```yaml
sdd.issue_platform:
  type: github          # github | gitlab | jira
  repo: owner/repo      # GitHub/GitLab only
  auth_method: cli      # cli | token
  auth:                  # Only when auth_method = token
    envfile: ".env"
    envfield: GITHUB_TOKEN
```

### Jira format

```yaml
sdd.issue_platform:
  type: jira
  url: https://yourorg.atlassian.net
  project: PROJ
  auth_method: token
  auth:
    envfile: ".env"
    envfield: JIRA_API_TOKEN
    envfield_email: JIRA_EMAIL
```

### Legacy format (backward compatible)

```yaml
platform:
  type: github
  repo: owner/repo
  auth:
    envfile: ".env"
    envfield: GITHUB_TOKEN
```

The `PlatformConfig::load()` function tries `[sdd.issue_platform]` first, falling back to `[platform]`.

## Architecture

### File layout

| File | Purpose |
|------|---------|
| `crates/cclab-cli/src/main.rs` | `#[command(alias = "sdd")]` on `Gen` variant |
| `crates/cclab-sdd/src/cli/platform.rs` | `set` + `show` commands, `detect_repo_from_git()` |
| `crates/cclab-sdd/src/cli/init.rs` | Delegates to `platform::detect_repo_from_git()` |
| `crates/cclab-sdd/src/services/platform_sync/config.rs` | `PlatformConfig::load()` with `SddSection` fallback |

### Shared helpers

`detect_repo_from_git(project_root)` parses the git remote URL to extract `owner/repo`. Supports both SSH (`git@host:owner/repo.git`) and HTTPS (`https://host/owner/repo.git`) formats. Used by both `platform set` and `cclab init`.

### Config upsert

`platform set` writes config by:
1. Reading `cclab/config.toml` as a string
2. Stripping old `[sdd.issue_platform]`, `[sdd.issue_platform.*]`, `[platform]`, and `[platform.*]` sections line-by-line
3. Appending the new `[sdd.issue_platform]` block
4. Writing back

This approach preserves all other config sections and comments.


## Requirements

## Requirements
<!-- type: requirements lang: markdown -->

| ID | Title | Priority | Acceptance Criteria |
|----|-------|----------|---------------------|
| R1 | repo_platform config section | P0 | New `[sdd.repo_platform]` section in `cclab/config.toml`. Fields: `type` (string, required), `repo` (string, required — no fallback to issue_platform.repo), `default_branch` (string, default "main"), `auto_commit` (bool, default false), `auto_pr` (bool, default false). Parsed by `RepoPlatformConfig` struct. |
| R2 | spec_platform config section | P1 | New `[sdd.spec_platform]` section. Fields: `type` (string, required — currently only "local"), `path` (string, default "cclab/specs"). Parsed by `SpecPlatformConfig` struct. |
| R3 | docs_platform reserved section | P2 | `[sdd.docs_platform]` documented in config template as commented-out section. Not parsed — no struct needed yet. Template shows `# type = "github_pages"` as example. |
| R4 | SddSection extension | P0 | `SddSection` struct in `config.rs` gains `repo_platform: Option<RepoPlatformConfig>` and `spec_platform: Option<SpecPlatformConfig>` fields with `#[serde(default)]`. Existing `issue_platform` field unchanged. |
| R5 | SddConfig integration | P0 | `SddConfig` struct gains `repo_platform: Option<RepoPlatformConfig>` and `spec_platform: Option<SpecPlatformConfig>`. Loaded via `SddConfig::load()` from `[sdd.repo_platform]` and `[sdd.spec_platform]` TOML sections. Missing section → `None`. |
| R6 | platform show multi-section display | P1 | `cclab sdd platform show` displays all configured platform sections (issue, repo, spec). For each present section: show section name, type, and key fields. Absent sections show "not configured". |
| R7 | Config template update | P1 | `crates/cclab-sdd/templates/config.toml` includes `[sdd.repo_platform]` and `[sdd.spec_platform]` sections with default values and comments. `[sdd.docs_platform]` included as commented-out block. |

### Constraints

- `repo_platform.repo` is required with no fallback — per clarification Q3
- `spec_platform.type` currently only supports `"local"` — remote spec backends are future work
- `docs_platform` is placeholder only — no runtime code changes
- Existing `PlatformConfig` (issue_platform) struct and loading logic unchanged
- `SddSection` in `config.rs` is module-private — only used for TOML deserialization intermediate


## Scenarios

## Scenarios
<!-- type: scenarios lang: markdown -->

### S1: Full repo_platform config loaded (R1, R5)

**GIVEN** `cclab/config.toml` contains:
```toml
[sdd.repo_platform]
type = "github"
repo = "owner/repo"
default_branch = "main"
auto_commit = true
auto_pr = false
```
**WHEN** `SddConfig::load()` is called
**THEN** `config.repo_platform` is `Some(RepoPlatformConfig)` with `type_ = "github"`, `repo = "owner/repo"`, `default_branch = "main"`, `auto_commit = true`, `auto_pr = false`.

### S2: repo_platform section absent — defaults to None (R5)

**GIVEN** `cclab/config.toml` has no `[sdd.repo_platform]` section
**WHEN** `SddConfig::load()` is called
**THEN** `config.repo_platform` is `None`. No error. All consumers treat absent as auto_commit=false, auto_pr=false.

### S3: repo_platform with defaults only (R1)

**GIVEN** `cclab/config.toml` contains:
```toml
[sdd.repo_platform]
type = "github"
repo = "owner/repo"
```
**WHEN** `RepoPlatformConfig` is deserialized
**THEN** `default_branch = "main"`, `auto_commit = false`, `auto_pr = false` (all defaults applied).

### S4: spec_platform local config (R2, R5)

**GIVEN** `cclab/config.toml` contains:
```toml
[sdd.spec_platform]
type = "local"
path = "cclab/specs"
```
**WHEN** `SddConfig::load()` is called
**THEN** `config.spec_platform` is `Some(SpecPlatformConfig)` with `type_ = "local"`, `path = "cclab/specs"`.

### S5: spec_platform absent — defaults to None (R5)

**GIVEN** `cclab/config.toml` has no `[sdd.spec_platform]` section
**WHEN** `SddConfig::load()` is called
**THEN** `config.spec_platform` is `None`. Consumers fall back to hardcoded `"cclab/specs"` path.

### S6: SddSection deserializes all platform sections (R4)

**GIVEN** `cclab/config.toml` contains `[sdd.issue_platform]`, `[sdd.repo_platform]`, and `[sdd.spec_platform]` sections
**WHEN** TOML is parsed into `ConfigFile` → `SddSection`
**THEN** all three `Option` fields are `Some(...)`. Each section is independently parsed.

### S7: platform show displays all sections (R6)

**GIVEN** `cclab/config.toml` has `issue_platform` (github), `repo_platform` (github, auto_commit=true), and `spec_platform` (local)
**WHEN** `cclab sdd platform show` is run
**THEN** output shows three sections with their types and key fields. Missing sections show "not configured".

### S8: repo_platform missing repo field — parse error (R1)

**GIVEN** `cclab/config.toml` contains:
```toml
[sdd.repo_platform]
type = "github"
auto_commit = true
```
(missing required `repo` field)
**WHEN** TOML is parsed
**THEN** deserialization fails with error indicating `repo` field is required.


## Config

## Config: New Platform Sections
<!-- type: config lang: json -->

### repo_platform

```json
{
  "$id": "repo-platform-config",
  "title": "RepoPlatformConfig",
  "description": "Git repository and PR operations config — [sdd.repo_platform] in cclab/config.toml",
  "type": "object",
  "properties": {
    "type": {
      "type": "string",
      "enum": ["github", "gitlab"],
      "description": "VCS platform type"
    },
    "repo": {
      "type": "string",
      "pattern": "^[\\w.-]+/[\\w.-]+$",
      "description": "Repository in owner/repo format. Required — no fallback to issue_platform.repo."
    },
    "default_branch": {
      "type": "string",
      "default": "main",
      "description": "Target branch for auto-PR creation"
    },
    "auto_commit": {
      "type": "boolean",
      "default": false,
      "description": "Auto git-commit cclab/ changes after merge archive"
    },
    "auto_pr": {
      "type": "boolean",
      "default": false,
      "description": "Auto-create PR after auto-commit. Requires auto_commit=true."
    }
  },
  "required": ["type", "repo"],
  "additionalProperties": false
}
```

### spec_platform

```json
{
  "$id": "spec-platform-config",
  "title": "SpecPlatformConfig",
  "description": "Spec storage config — [sdd.spec_platform] in cclab/config.toml",
  "type": "object",
  "properties": {
    "type": {
      "type": "string",
      "enum": ["local"],
      "description": "Storage backend type. Currently only 'local' supported."
    },
    "path": {
      "type": "string",
      "default": "cclab/specs",
      "description": "Relative path to spec storage directory from project root"
    }
  },
  "required": ["type"],
  "additionalProperties": false
}
```

### docs_platform (future)

```json
{
  "$id": "docs-platform-config",
  "title": "DocsPlatformConfig",
  "description": "[sdd.docs_platform] — reserved for future use. Not parsed at runtime.",
  "type": "object",
  "properties": {
    "type": {
      "type": "string",
      "enum": ["github_pages", "notion", "confluence"],
      "description": "Documentation platform type"
    }
  },
  "required": ["type"],
  "additionalProperties": true
}
```

### TOML Example (all sections)

```toml
[sdd.issue_platform]
type = "github"
repo = "chrischeng-c4/cclab"
auth_method = "cli"

[sdd.repo_platform]
type = "github"
repo = "chrischeng-c4/cclab"
default_branch = "main"
auto_commit = true
auto_pr = false

[sdd.spec_platform]
type = "local"
path = "cclab/specs"

# [sdd.docs_platform]
# type = "github_pages"
```


## Changes

## Changes
<!-- type: changes lang: yaml -->

```yaml
files:
  - path: crates/cclab-sdd/src/models/change.rs
    action: MODIFY
    desc: |
      Add RepoPlatformConfig and SpecPlatformConfig structs.

      Add struct: RepoPlatformConfig {
        #[serde(rename = "type")] type_: String,
        repo: String,
        #[serde(default = "default_main")] default_branch: String,
        #[serde(default)] auto_commit: bool,
        #[serde(default)] auto_pr: bool,
      }

      Add struct: SpecPlatformConfig {
        #[serde(rename = "type")] type_: String,
        #[serde(default = "default_specs_path")] path: String,
      }

      Add fields to SddConfig:
        #[serde(default, skip_serializing_if = "Option::is_none")]
        repo_platform: Option<RepoPlatformConfig>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        spec_platform: Option<SpecPlatformConfig>,

  - path: crates/cclab-sdd/src/services/platform_sync/config.rs
    action: MODIFY
    desc: |
      Extend SddSection to parse new platform sections.

      Import RepoPlatformConfig, SpecPlatformConfig from models::change.

      Add fields to SddSection:
        #[serde(default)]
        repo_platform: Option<RepoPlatformConfig>,
        #[serde(default)]
        spec_platform: Option<SpecPlatformConfig>,

      No changes to PlatformConfig::load() — it only loads issue_platform.
      New sections are loaded via SddConfig::load() path.

  - path: crates/cclab-sdd/src/cli/platform.rs
    action: MODIFY
    desc: |
      Extend `run_show` to display repo_platform and spec_platform sections.

      After displaying issue_platform, load SddConfig and:
      - If repo_platform is Some: print section header, type, repo,
        default_branch, auto_commit, auto_pr
      - If repo_platform is None: print "repo_platform: not configured"
      - If spec_platform is Some: print section header, type, path
      - If spec_platform is None: print "spec_platform: not configured"

  - path: crates/cclab-sdd/templates/config.toml
    action: MODIFY
    desc: |
      Add repo_platform and spec_platform sections to config template.

      Add after [sdd.issue_platform] section:
        [sdd.repo_platform]
        type = "github"
        repo = "owner/repo"
        default_branch = "main"
        auto_commit = false
        auto_pr = false

        [sdd.spec_platform]
        type = "local"
        path = "cclab/specs"

        # [sdd.docs_platform]
        # type = "github_pages"
```

# Reviews
