---
number: 925
title: "feat(agent): Add CodeAgent — spec-to-code with server-side git"
state: open
labels: [enhancement, crate:agent, P1]
group: "code-agent-core"
---

# #925 — feat(agent): Add CodeAgent — spec-to-code with server-side git

**Parent**: #920

## Summary

Agent that transforms approved specs into multi-file code implementation. Creates branches, commits, and opens PRs via platform integration.

## Capabilities

- Parse spec to identify implementation tasks
- Topological sort by dependency
- Multi-file code generation
- Git operations via platform integration (not local git)
- PR/MR creation with spec link

## Dependencies
- #921 GitLab integration (files, MR)
- #922 GitHub integration (files, PR)
- ReviewAgent (for CRR code review)

## Test Plan
- [ ] Unit: spec → task decomposition
- [ ] Unit: topological sort
- [ ] Integration: code generation from spec
- [ ] Integration: PR creation (mock API)
