---
number: 464
title: "feat(cli): cclab init — platform selection with auth method options"
state: open
labels: [enhancement, crate:sdd]
---

# #464 — feat(cli): cclab init — platform selection with auth method options

## Overview

`cclab init` should guide users through platform configuration interactively, generating the `[platform]` section in `cclab/config.toml`.

## Platform Choices

| Option | CLI Tool | Auth Methods |
|--------|----------|-------------|
| GitHub | `gh` | API token (`GITHUB_TOKEN`) **or** CLI OAuth (`gh auth login`) |
| GitLab | `glab` | API token (`GITLAB_TOKEN`) **or** CLI OAuth (`glab auth login`) |
| Jira | — | API token only |
| None | — | Skip platform config |

## Auth Methods

For GitHub and GitLab, two auth strategies:

### 1. API Token
- User provides a personal access token
- Stored in `.env` file (gitignored)
- Config generates:
  ```toml
  [platform.auth]
  envfile = ".env"
  envfield = "GITHUB_TOKEN"  # or GITLAB_TOKEN
  ```

### 2. CLI OAuth (`gh` / `glab`)
- User runs `gh auth login` or `glab auth login` themselves
- No token stored in project — CLI handles auth transparently
- Config generates:
  ```toml
  [platform]
  type = "github"  # or "gitlab"
  repo = "owner/repo"
  auth_method = "cli"  # signals to use gh/glab directly without token
  ```

## Interactive Flow

```
$ cclab init

? Select platform:
  ❯ GitHub
    GitLab
    Jira
    None (skip)

? Repository (owner/repo): chris.cheng/cclab

? Authentication method:
  ❯ CLI OAuth (gh auth login — recommended)
    API Token (stored in .env)
```

If CLI OAuth selected → verify CLI is installed (`gh --version` / `glab --version`), warn if not found.
If API Token selected → prompt for token, write to `.env`, ensure `.env` is in `.gitignore`.

## Acceptance Criteria

- WHEN user runs `cclab init` THEN platform selection is presented
- WHEN GitHub or GitLab is selected THEN auth method choice (CLI OAuth vs API token) is offered
- WHEN Jira is selected THEN only API token auth is offered
- WHEN None is selected THEN `[platform]` section is skipped
- WHEN CLI OAuth is selected THEN verify CLI tool is installed and warn if missing
- WHEN API token is selected THEN write token to `.env` and ensure `.gitignore` includes `.env`
- WHEN init completes THEN valid `[platform]` section is written to `cclab/config.toml`

## Depends On

- `fetch_issues` currently hardcodes `gh` CLI — needs `glab` support (#TBD)
- `platform_sync` config already supports `type = "gitlab"` but CLI calls need `glab` integration
