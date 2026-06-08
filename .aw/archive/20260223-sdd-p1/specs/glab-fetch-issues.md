---
id: glab-fetch-issues
type: spec
title: "Add GitLab (glab) Support to fetch_issues"
version: 1
spec_type: algorithm
created_at: 2026-02-23T00:00:00+00:00
updated_at: 2026-02-23T00:00:00+00:00
requirements:
  total: 4
  ids: [R1, R2, R3, R4]
depends: [platform-config-init]
---

# Add GitLab (glab) Support to fetch_issues

## Overview

Fix #465: `sdd_fetch_issues` currently hardcodes `gh` (GitHub CLI). Add `glab` (GitLab CLI) support based on `config.toml` `[platform] type`.

## Requirements

### R1 - Platform-aware CLI dispatch

Read `[platform] type` from `cclab/config.toml` and route to the correct CLI.

**WHEN** `platform.type = "github"` or no `[platform]` config
**THEN** use `gh` CLI (current behavior, unchanged)

**WHEN** `platform.type = "gitlab"`
**THEN** use `glab` CLI

### R2 - GitLab issue fetching

Implement `run_glab()` parallel to existing `run_gh()`.

**WHEN** fetching a GitLab issue
**THEN** run `glab issue view NNN --repo owner/repo --output json`
**THEN** parse JSON output into same `issue_{NNN}.md` format

### R3 - GitLab issue relationships

Fetch issue links via GitLab REST API for DAG construction.

**WHEN** fetching GitLab issue links
**THEN** call `GET /projects/:id/issues/:iid/links`
**THEN** extract `blocks`/`is_blocked_by` for DAG edges
**THEN** extract `relates_to` for related issues context

### R4 - Auth method support

Support both auth strategies from #464.

**WHEN** `auth_method = "cli"`
**THEN** `glab` handles auth transparently

**WHEN** `auth_method = "token"`
**THEN** pass `GITLAB_TOKEN` from `.env` or environment

## Affected Files

- `crates/cclab-sdd/src/mcp/tools/fetch_issues.rs` — add `run_glab()`, platform dispatch
- `crates/cclab-sdd/src/models/change.rs` — platform config reading
