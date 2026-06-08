---
id: platform-config-init
type: spec
title: "Add Platform Selection to cclab init"
version: 1
spec_type: algorithm
created_at: 2026-02-23T00:00:00+00:00
updated_at: 2026-02-23T00:00:00+00:00
requirements:
  total: 4
  ids: [R1, R2, R3, R4]
---

# Add Platform Selection to cclab init

## Overview

Fix #464: Add interactive platform configuration to `cclab init`. Users select their platform (GitHub/GitLab/Jira/None) and auth method (CLI OAuth vs API token), generating the `[platform]` section in `cclab/config.toml`.

## Requirements

### R1 - Platform selection prompt

Add interactive platform selection to `cclab init` after existing initialization steps.

**WHEN** user runs `cclab init`
**THEN** prompt: "Select platform:" with options GitHub, GitLab, Jira, None

### R2 - Auth method selection

For GitHub and GitLab, offer two auth methods. For Jira, only API token.

**WHEN** GitHub or GitLab selected
**THEN** prompt: "Authentication method:" with CLI OAuth (recommended) and API Token options

**WHEN** Jira selected
**THEN** only API token auth offered

### R3 - Config generation

Write `[platform]` section to `cclab/config.toml` based on selections.

**WHEN** CLI OAuth selected
**THEN** write `auth_method = "cli"` and verify CLI tool installed (`gh --version` / `glab --version`)

**WHEN** API Token selected
**THEN** prompt for token, write to `.env`, ensure `.env` in `.gitignore`, write `envfield` config

### R4 - Repository detection

Auto-detect repository from git remote.

**WHEN** git remote exists
**THEN** pre-fill `repo = "owner/repo"` from remote URL

## Affected Files

- `crates/cclab-sdd/src/cli/init.rs` — add platform selection flow
- `crates/cclab-sdd/src/models/change.rs` — add `PlatformConfig` struct if needed
