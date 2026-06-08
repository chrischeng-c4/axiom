---
change: genesis-fetch-issues
date: 2026-02-12
---

# Clarifications

## Q1: Scope: next field migration
- **Question**: Should the mainthread_instruction → next field migration be included in this change, or split into a separate issue?
- **Answer**: Include in this change — migrate all actions to the new next field format in one shot.
- **Rationale**: Doing it together avoids maintaining two response formats. The spec already defines next as the standard, so aligning everything at once is cleaner.

## Q2: GitHub API access method
- **Question**: genesis_fetch_issues needs to fetch GitHub issue content. How should it access GitHub?
- **Answer**: Use gh CLI — shell out to `gh issue view`.
- **Rationale**: Simple, uses existing auth, no additional dependencies (reqwest, GITHUB_TOKEN). The gh CLI is already used extensively in the project.

## Q3: Git workflow
- **Question**: Which git workflow for this change?
- **Answer**: new_branch — create cclab/genesis-fetch-issues branch.
- **Rationale**: Standard branch workflow for feature development.

