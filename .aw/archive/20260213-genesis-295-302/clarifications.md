---
change: genesis-295-302
date: 2026-02-13
---

# Clarifications

## Q1: Approach
- **Question**: These 8 issues split into spec gaps (#295-#301) and code change (#302). How should we approach them?
- **Answer**: Spec and code together — updating specs means implementation must also change to match.
- **Rationale**: Specs define contracts, code must align. Doing both ensures consistency.

## Q2: Legacy v1 paths (#297)
- **Question**: For legacy/v1 paths: document what exists (Option A) or remove legacy code (Option B)?
- **Answer**: Remove legacy (Option B) — remove v1 paths from code.
- **Rationale**: Legacy v1 paths add maintenance burden and spec complexity. Removing them simplifies both spec and code.

## Q3: Tag/type validation (#298)
- **Question**: Should validation be tag-union based (as spec says) or type-based (as code does)?
- **Answer**: Align code to spec — refactor code to use tag-union validation.
- **Rationale**: Tag-union is more flexible and compositional. The spec already defines the correct approach.

## Q4: Git workflow
- **Question**: Which git workflow for this change?
- **Answer**: new_branch — create cclab/genesis-295-302 branch
- **Rationale**: Separate branch for a large multi-issue change to keep main clean.

