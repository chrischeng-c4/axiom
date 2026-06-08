---
change: issues-cli-crud
group: issues-cli-crud
date: 2026-04-06
---

# Requirements

## Requirements

### R1: `score issues create` — agent-driven issue authoring
- Create local draft: write `.score/issues/{type}-{slug}.md` with `state: draft`, no `id` (not yet on GitHub)
- Push to GitHub: optional `--push` flag calls `gh issue create` and backfills `id` + `url` in frontmatter
- Input: `--title`, `--type`, `--body` (or `--body-file` for stdin/pipe from agent)
- Output: `--json` returns the created Issue struct
- No interactive prompts — all fields via args

### R2: `score issues update` — modify metadata + body
- Local: parse frontmatter, apply patch (add/remove labels, change state/title/body), rewrite file
- GitHub: optional `--push` flag calls `gh issue edit` to sync changes upstream
- Input: `<slug-or-id>`, `--title`, `--add-label`, `--remove-label`, `--state`, `--body-file`
- Output: `--json` returns updated Issue

### R3: `score issues close` — close with reason
- Local: set `state: closed` in frontmatter
- GitHub: `--push` flag calls `gh issue close` (with `--reason` if supported)
- Input: `<slug-or-id>`, `--reason` (optional comment)
- Output: `--json` returns closed Issue

### R4: `score issues find <query>` — full-text search
- Local backend: grep across all `.score/issues/*.md` content (frontmatter + body)
- GitHub backend: `gh issue list --search <query>`
- Returns matching issues sorted by relevance
- Agent uses this to check for duplicates before creating
- Output: `--json` returns array of matching Issues

### R5: Cross-artifact references in frontmatter
- New optional frontmatter fields: `related: [<slug-or-path>...]`, `implements: [<slug-or-path>...]`
- `related` = soft link ("see also"): other issues, BRDs, PRDs
- `implements` = hard link ("this issue is realized by"): changes, tech_designs
- Validate: referenced slugs/paths exist on `score issues list` (warn if broken)
- Type system: `Issue`, `IssueFrontmatter` gain these fields

### R6: GitLab backend (real implementation)
- Replace `todo!()` stub with `glab` CLI shell-out
- `glab issue list --json`, `glab issue view`, `glab issue create`
- Reuse pattern from `crates/sdd/src/tools/fetch_issues.rs::fetch_issue_glab`
- Factory: `make_backend("gitlab", ...)` returns working GitLabBackend

### R7: Agent-first design constraints (all verbs)
- Every verb has `--json` output
- No `dialoguer` or interactive prompts anywhere
- Structured error JSON on stderr: `{"error": "message", "code": "NOT_FOUND"}`
- Exit codes: 0 = success, 1 = not found, 2 = validation error, 3 = backend error
- Input large content via `--body-file -` (stdin pipe)

### R8: `IssueBackend` trait extension
- Add `create(&self, draft: &Issue) -> Result<Issue>` (returns issue with id assigned)
- Add `update(&self, id: &str, patch: &IssuePatch) -> Result<Issue>`
- Add `close(&self, id: &str, reason: Option<&str>) -> Result<()>`
- Add `search(&self, query: &str) -> Result<Vec<Issue>>`
- All backends implement all methods (LocalBackend, GitHubBackend, GitLabBackend)
- Jira remains stub (not implemented in this change)
