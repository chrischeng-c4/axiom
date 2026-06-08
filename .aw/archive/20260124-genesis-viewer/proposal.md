---
id: genesis-viewer
type: proposal
version: 1
created_at: 2026-01-24T14:59:04.675590+00:00
updated_at: 2026-01-24T14:59:04.675590+00:00
author: mcp
status: proposed
iteration: 1
summary: "Expand Genesis Viewer with project-level browsing and enhanced rendering"
history:
  - timestamp: 2026-01-24T14:59:04.675590+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
impact:
  scope: minor
  affected_files: 8
  new_files: 2
affected_specs:
  - id: plan-viewer
    path: specs/plan-viewer.md
    depends: []
  - id: genesis-viewer-expansion
    path: specs/genesis-viewer-expansion.md
    depends: []
---

<proposal>

# Change: genesis-viewer

## Summary

Expand Genesis Viewer with project-level browsing and enhanced rendering

## Why

<meta>
  <purpose>PRD - Product Requirements Document</purpose>
  <constraint>Describes WHAT and WHY, not HOW</constraint>
</meta>

The current Genesis Viewer is limited to viewing individual changes, lacking a global view of project specifications and knowledge. Expanding this capability improves developers' understanding of project architecture and review efficiency. Additionally, supporting LaTeX and enhanced Markdown rendering is a fundamental requirement for technical documentation.

## What Changes

- Implement /{project}/genesis route in cclab-server for project-level directory browsing.
- Refactor ViewerManager to support scanning the entire genesis/ directory and generating tree structures.
- Implement a tree navigation interface and file content preview in the frontend.
- Integrate KaTeX for LaTeX mathematical formula rendering.
- Implement client-side interactive table sorting.
- Register a new genesis-view-project skill for automatic browser opening.

## Impact

- **Scope**: minor
- **Affected Files**: ~8
- **New Files**: ~2
- Affected specs:
  - `plan-viewer` (no dependencies)
  - `genesis-viewer-expansion` (no dependencies)
- Affected code: `crates/cclab-server/src/http_server.rs`, `crates/cclab-genesis/src/ui/viewer/manager.rs`, `crates/cclab-genesis/src/ui/viewer/render.rs`, `crates/cclab-genesis/src/ui/viewer/assets/app.js`, `crates/cclab-genesis/src/ui/viewer/assets/index.html`
- **Breaking Changes**: No

</proposal>

<review iteration="1" reviewer="codex" status="needs_revision">
## Summary
Tasks cover core backend, frontend, and skill registration work, but there are traceability and dependency gaps plus a missing HTML/asset update for KaTeX.

## Issues
1. **Spec traceability mismatch in tests**: Task 4.1 uses `spec_ref: plan-viewer:acceptance-criteria`, which is not a requirement ID and breaks requirement-to-task mapping. Use explicit IDs (e.g., `plan-viewer:R1`, `plan-viewer:R4`, `plan-viewer:R5`) or split the test task accordingly.
2. **Incorrect test dependencies**: Task 4.1 depends on `3.1` (frontend JS), but the described tests target server-side tree generation and LaTeX markup in `render.rs`, which depend on `1.1` and `2.2` (and possibly `2.1` if API output is validated). Update `depends_on` to reflect the actual build order.
3. **Missing task for KaTeX assets in HTML**: The proposal lists `crates/cclab-genesis/src/ui/viewer/assets/index.html` as affected, but no task covers adding KaTeX CSS/JS or required markup. Add a task to update the HTML asset to load KaTeX (and any necessary UI container changes).

## Verdict
needs_revision

## Next Steps
- Fix Task 4.1 spec references and dependencies.
- Add a task for updating `crates/cclab-genesis/src/ui/viewer/assets/index.html` to include KaTeX assets (and related markup if needed).
- Re-validate task/spec traceability after edits.
</review>
