---
change: issues-cli-crud
group: issues-cli-crud
date: 2026-04-06
status: answered
---

# Pre-Clarifications

### Q1: Create workflow
- **Question**: Should score issues create always create a local draft first, then optionally push? Or should there be a direct-to-GitHub mode?
- **Answer**: Support both. Default: local-first (creates .score/issues/ draft with state: draft). Add --remote flag for direct-to-GitHub (calls gh issue create, writes local file with id backfilled). Agent flexibility: draft offline, push later; OR create directly when online.
- **Rationale**: Agent may be working offline (no gh access) or online (wants immediate tracking). Supporting both gives maximum flexibility without complexity — the difference is just whether gh issue create runs before or after local file write.

### Q2: Update scope
- **Question**: For score issues update, should body editing be in-place (rewrite full body from --body-file) or patch-based?
- **Answer**: Full body replacement via --body-file. Agent generates complete content, passes it in. No partial patching.
- **Rationale**: Agents already produce complete content. Patch-based editing adds complexity (section detection, merge logic) with no real benefit since the agent rewrites the whole body anyway.

### Q3: Cross-reference validation
- **Question**: Should related:/implements: references be validated at write-time or list-time?
- **Answer**: Warn at list-time only. Write accepts any reference without validation.
- **Rationale**: Write-time validation creates ordering dependencies — agent can't create an issue referencing a change that doesn't exist yet. List-time warnings let agents work in any order, then check consistency afterward.

### Q4: GitLab testing
- **Question**: Is there a GitLab repo available for integration testing?
- **Answer**: Unit tests only with mocked glab CLI output. No GitLab repo available for integration testing.
- **Rationale**: Real GitLab integration testing requires a running GitLab instance. Mock the glab binary's JSON output to verify parsing and error handling. Add integration tests later when a GitLab repo is available.

