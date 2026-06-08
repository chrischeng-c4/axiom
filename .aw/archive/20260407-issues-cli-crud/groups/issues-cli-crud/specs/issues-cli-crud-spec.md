---
id: issues-cli-crud-spec
main_spec_ref: "crates/sdd/logic/issues-backend.md"
merge_strategy: new
filled_sections: [overview, requirements, scenarios, changes]
fill_sections: [overview, requirements, scenarios, changes]
create_complete: true
---

# Issues Cli Crud Spec

## Overview

## Overview

Extends the `score issues` CLI and the underlying `IssueBackend` trait with full CRUD + search capabilities. The existing MVP (list/show/sync, read-only) becomes a complete lifecycle management interface that agents use to create, update, close, and search issues across local files, GitHub, and GitLab — all through a single uniform CLI.

### Architecture

**Arsenal layer** (`crates/sdd/src/issues/`):
- `IssueBackend` trait gains 4 new methods: `create`, `update`, `close`, `search`
- `LocalBackend` implements all via frontmatter read/write
- `GitHubBackend` shells out to `gh issue create/edit/close/list --search`
- New `GitLabBackend` shells out to `glab` (pattern from `fetch_issues.rs::fetch_issue_glab`)
- `IssuePatch` struct for partial updates (only changed fields)
- `Issue` frontmatter gains `related:` and `implements:` cross-reference fields

**Show case layer** (`projects/score/cli/src/issues.rs`):
- 4 new subcommands: `create`, `update`, `close`, `find`
- All verbs have `--json` output (agent-first)
- Structured JSON error output on stderr with exit codes
- `create` supports local-first (default) + `--remote` (direct-to-GitHub)
- `update` uses `--body-file` for full body replacement (no patch)

### Agent-First Design

Score is an agent development tool. The CLI is designed for Claude Code agents, not humans:
- Every verb outputs `--json` for structured parsing
- No interactive prompts (`dialoguer`) anywhere
- Input via `--body-file -` (stdin pipe) for large content
- Structured errors: `{"error": "message", "code": "NOT_FOUND"}` on stderr
- Exit codes: 0=success, 1=not found, 2=validation error, 3=backend error

### Cross-Artifact References

Issue frontmatter gains two optional fields:
- `related: [slug-or-path, ...]` — soft "see also" links to other issues, BRDs, PRDs
- `implements: [slug-or-path, ...]` — hard links to changes, tech_designs that realize this issue

Validation is warn-at-list-time only — broken references don't block writes.
## Requirements

## Requirements

| ID | Requirement | Priority | Acceptance Criteria |
|----|------------|----------|--------------------|
| R1 | `score issues create` — local draft + optional GitHub push | P0 | `create --title T --type bug --body B` writes `.score/issues/bug-T.md` with `state: draft`. `create --title T --type bug --body B --remote` also calls `gh issue create` and backfills `id` + `url`. `--json` returns the `Issue` struct. |
| R2 | `score issues update` — metadata + body patch | P0 | `update <slug> --title T2 --add-label L` rewrites frontmatter. `update <slug> --body-file path` replaces body entirely. `--push` syncs to GitHub via `gh issue edit`. `--json` returns updated `Issue`. |
| R3 | `score issues close` — close with reason | P1 | `close <slug> --reason R` sets `state: closed` locally. `--push` calls `gh issue close`. `--json` returns closed `Issue`. |
| R4 | `score issues find <query>` — full-text search | P1 | Local: grep across all `.score/issues/*.md`. GitHub: `gh issue list --search`. Returns array of matching `Issue`s sorted by relevance. `--json` output. |
| R5 | Cross-artifact references | P1 | `Issue` frontmatter gains `related: [...]` and `implements: [...]` optional fields. Broken references warned at `score issues list` time, not at write time. |
| R6 | GitLab backend | P2 | `make_backend("gitlab", ...)` returns `GitLabBackend` that shells out to `glab issue {list,view,create,edit,close}`. Tested with mocked `glab` output. |
| R7 | Agent-first design (all verbs) | P0 | `--json` on every verb. No `dialoguer` prompts. Structured JSON errors on stderr. Exit codes: 0=ok, 1=not_found, 2=validation, 3=backend. `--body-file -` for stdin. |
| R8 | `IssueBackend` trait extension | P0 | Trait gains: `create(&Issue) -> Issue`, `update(&str, &IssuePatch) -> Issue`, `close(&str, Option<&str>)`, `search(&str) -> Vec<Issue>`. LocalBackend + GitHubBackend + GitLabBackend implement all. Jira stays `todo!()`. |
## Scenarios

## Scenarios

### S1: Create local draft issue (R1, R7)

**GIVEN** the user runs `score issues create --title "Fix login bug" --type bug --body "Login fails on Safari"`
**WHEN** the command completes
**THEN** `.score/issues/bug-fix-login-bug.md` exists with:
  - frontmatter: `type: bug`, `state: draft`, `id: null`, `title: "Fix login bug"`
  - body: "Login fails on Safari"
  - exit code 0
  - `--json` returns `{"type":"bug","title":"Fix login bug","state":"draft","id":null,...}`

### S2: Create with --remote pushes to GitHub (R1)

**GIVEN** `gh auth status` is authenticated
**WHEN** user runs `score issues create --title "New feature" --type enhancement --body "..." --remote`
**THEN** `gh issue create` is called, local file written with `id` and `url` populated from GitHub response, `state: open`

### S3: Update metadata locally (R2)

**GIVEN** `.score/issues/bug-fix-login-bug.md` exists with `state: draft`
**WHEN** user runs `score issues update bug-fix-login-bug --add-label priority:p1 --state open`
**THEN** frontmatter gains `priority:p1` in labels, `state: open`. Body unchanged.

### S4: Update body via --body-file (R2)

**GIVEN** a file `/tmp/new-body.md` with updated content
**WHEN** `score issues update bug-fix-login-bug --body-file /tmp/new-body.md`
**THEN** body is fully replaced. Frontmatter unchanged.

### S5: Close with reason (R3)

**GIVEN** `bug-fix-login-bug.md` exists with `state: open`
**WHEN** `score issues close bug-fix-login-bug --reason "Fixed in PR #42"`
**THEN** frontmatter `state: closed`. Body unchanged. Exit 0. `--json` returns closed issue.

### S6: Close with --push syncs to GitHub (R3)

**GIVEN** issue has `id: 1234` (synced to GitHub)
**WHEN** `score issues close bug-fix-login-bug --push --reason "Resolved"`
**THEN** `gh issue close 1234 --comment "Resolved"` is called. Local state updated.

### S7: Find across local backend (R4)

**GIVEN** 3 issues in `.score/issues/`: bug-login, enhancement-oauth, epic-auth
**WHEN** `score issues find "auth"`
**THEN** returns enhancement-oauth and epic-auth (body/title match "auth"). bug-login excluded. `--json` returns array.

### S8: Find across GitHub backend (R4)

**WHEN** `score issues find "auth" --backend github`
**THEN** runs `gh issue list --search "auth" --json ...` and returns matching issues.

### S9: Cross-references at list time (R5)

**GIVEN** bug-fix-login.md has `related: [epic-auth]` and `epic-auth.md` exists
**WHEN** `score issues list`
**THEN** no warnings. Both issues listed.

**GIVEN** bug-fix-login.md has `related: [nonexistent-slug]`
**WHEN** `score issues list`
**THEN** warning printed: "broken reference 'nonexistent-slug' in bug-fix-login"

### S10: GitLab backend list (R6)

**GIVEN** config has `[sdd.issue_platform] type = "gitlab"`
**WHEN** `score issues list --backend gitlab`
**THEN** `glab issue list --output json` is called, issues parsed and returned.

### S11: Structured error output (R7)

**WHEN** `score issues show nonexistent-slug --json`
**THEN** stderr: `{"error":"issue 'nonexistent-slug' not found","code":"NOT_FOUND"}`
**AND** exit code 1

### S12: IssueBackend trait uniformity (R8)

**GIVEN** `LocalBackend`, `GitHubBackend`, `GitLabBackend` all implement `IssueBackend`
**WHEN** any code holds `&dyn IssueBackend`
**THEN** `create`, `update`, `close`, `search`, `list`, `get`, `write` are all callable without knowing the concrete backend type.
## Diagrams

### Interaction
<!-- type: interaction lang: mermaid -->
<!-- TODO -->

### Logic
<!-- type: logic lang: mermaid -->
<!-- TODO -->

### Dependencies
<!-- type: dependency lang: mermaid -->
<!-- TODO -->

### State Machine
<!-- type: state-machine lang: mermaid -->
<!-- TODO -->

### Data Model
<!-- type: db-model lang: mermaid -->
<!-- TODO -->

## API Spec

### REST API
<!-- type: rest-api lang: yaml -->
<!-- TODO -->

### RPC API
<!-- type: rpc-api lang: json -->
<!-- TODO -->

### Async API
<!-- type: async-api lang: yaml -->
<!-- TODO -->

### CLI
<!-- type: cli lang: yaml -->
<!-- TODO -->

### Schema
<!-- type: schema lang: json -->
<!-- TODO -->

### Config
<!-- type: config lang: json -->
<!-- TODO -->

## Test Plan
<!-- type: test-plan lang: markdown -->

<!-- TODO -->

## Changes

## Changes

```yaml
changes:
  - file: crates/sdd/src/issues/backend.rs
    action: modify
    description: Add create, update, close, search methods to IssueBackend trait. Add IssuePatch struct.

  - file: crates/sdd/src/issues/types.rs
    action: modify
    description: Add IssuePatch struct. Add related/implements fields to Issue. Add IssueError enum for structured errors.

  - file: crates/sdd/src/issues/backends/local.rs
    action: modify
    description: Implement create (write draft), update (patch frontmatter + body), close (set state), search (grep content).

  - file: crates/sdd/src/issues/backends/github.rs
    action: modify
    description: Implement create (gh issue create), update (gh issue edit), close (gh issue close), search (gh issue list --search). Mark is_writable=true.

  - file: crates/sdd/src/issues/backends/gitlab.rs
    action: create
    description: New GitLabBackend — shells out to glab CLI. Implements full IssueBackend trait. Pattern from fetch_issues.rs::fetch_issue_glab.

  - file: crates/sdd/src/issues/backends/mod.rs
    action: modify
    description: Add pub mod gitlab. Re-export GitLabBackend.

  - file: crates/sdd/src/issues/mod.rs
    action: modify
    description: Update make_backend to return GitLabBackend for "gitlab". Re-export new types.

  - file: projects/score/cli/src/issues.rs
    action: modify
    description: Add Create, Update, Close, Find subcommands. Structured error handling with exit codes. --body-file support for stdin pipe.

  - file: projects/score/cli/src/commands.rs
    action: no-change
    description: Issues already wired. No change needed.
```
## Wireframe
<!-- type: wireframe lang: yaml -->

<!-- TODO -->

## Component
<!-- type: component lang: json -->

<!-- TODO -->

## Design Token
<!-- type: design-token lang: json -->

<!-- TODO -->

## Doc
<!-- type: doc lang: markdown -->

<!-- TODO -->

# Reviews
