---
number: 465
title: "feat(sdd): fetch_issues — add glab (GitLab) CLI support"
state: open
labels: [enhancement, crate:sdd]
---

# #465 — feat(sdd): fetch_issues — add glab (GitLab) CLI support

## Overview

`sdd_fetch_issues` currently hardcodes `gh` (GitHub CLI). Need to support `glab` (GitLab CLI) based on `config.toml` `[platform] type`.

## Current State

- `crates/cclab-sdd/src/mcp/tools/fetch_issues.rs` — `run_gh()` hardcodes `Command::new("gh")`
- Spec already defines GitLab support: `cclab/specs/cclab-genesis/fetch-issues.md`

## What Needs to Change

### 1. Platform-aware CLI dispatch

Read `[platform] type` from `cclab/config.toml` and route to the correct CLI:

| `platform.type` | CLI | Fetch command |
|---|---|---|
| `github` | `gh` | `gh issue view NNN --repo owner/repo --json number,title,body,labels,state,comments` |
| `gitlab` | `glab` | `glab issue view NNN --repo owner/repo --output json` |

### 2. GitLab API for relationships

Per spec, GitLab uses REST API for issue links:

```
GET /projects/:id/issues/:iid/links
```

| `link_type` | Usage | Tier |
|---|---|---|
| `blocks` / `is_blocked_by` | DAG edges | Premium+ |
| `relates_to` | Related issues (context) | Free |

### 3. Auth method support

Support both auth strategies (see #464):
- **CLI OAuth**: `glab` handles auth transparently (no token needed)
- **API Token**: pass via `GITLAB_TOKEN` env var or `.env` file

## Acceptance Criteria

- WHEN `platform.type = "gitlab"` THEN `fetch_issues` uses `glab` CLI
- WHEN `platform.type = "github"` THEN behavior unchanged (uses `gh`)
- WHEN no `[platform]` config exists THEN fallback to `gh` (current behavior)
- WHEN fetching GitLab issue links THEN extract `blocks`/`is_blocked_by` for DAG + `relates_to` for related issues
- WHEN GitLab issue is fetched THEN output format matches existing `issue_{NNN}.md` schema

## References

- Spec: `cclab/specs/cclab-genesis/fetch-issues.md`
- Related: #464 (cclab init platform selection)
