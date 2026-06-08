# cclab sdd platform

Issue tracking platform configuration CLI. Supports GitHub, GitLab, and Jira.

## Overview

`cclab sdd` is an alias for `cclab gen`. The `platform` subcommand group manages issue tracking platform configuration, which is stored in `cclab/config.toml` under `[sdd.issue_platform]`.

```yaml
commands:
  - cclab sdd platform set    # Interactive setup
  - cclab sdd platform show   # Show current configuration
```

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
