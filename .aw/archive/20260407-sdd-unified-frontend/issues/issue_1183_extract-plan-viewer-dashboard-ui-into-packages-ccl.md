---
number: 1183
title: "extract plan viewer + dashboard UI into packages/@cclab/* as shared components"
state: open
labels: [type:enhancement, priority:p2, project:conductor, project:score]
group: "sdd-unified-frontend"
---

# #1183 — extract plan viewer + dashboard UI into packages/@cclab/* as shared components

## Context

`crates/cclab-server/` currently hosts:
- Dashboard (project listing, `/`)
- Plan Viewer UI (change review, `/view/*`)
- Spec viewer, knowledge viewer, project.md viewer
- Project cclab tree viewer (`/{project}/cclab`)

These are server-side rendered HTML views embedded in the Rust crate. They work, but they're the wrong shape architecturally:

1. **Conductor** (the cloud web SDD project) already has its own React frontend under `projects/conductor/fe/`. It doesn't consume cclab-server's dashboard.
2. **Score** (the local CLI SDD project) doesn't need a web UI at all — CLI is the primary interface.
3. When a **GUI view of SDD artifacts is needed** (either by Conductor or by Score's future `score view` command), it should be **the same React component**, not a server-rendered HTML fragment.

The correct architecture per the repo philosophy (`crates/` = arsenal, `packages/` = JS/TS arsenal, `projects/` = show cases):

> **Plan viewer, dashboard, and all spec/change/issue viewers belong in `packages/@cclab/*` as reusable React components, consumed identically by Conductor's web UI and any future GUI that Score wants to launch.**

## Scope

Extract the following from `crates/cclab-server/src/http_server.rs` into React components under `packages/@cclab/`:

| Current server route / HTML | Proposed component |
|-----------------------------|---------------------|
| Dashboard (project list) | `@cclab/ui/dashboard` (new or extend existing) |
| Plan viewer (change review) | `@cclab/ui/plan-viewer` |
| Spec viewer (`handle_spec_viewer`) | `@cclab/spec-viewer` (already exists, extend) |
| Knowledge viewer | `@cclab/ui/knowledge-viewer` (new) |
| Project.md viewer | `@cclab/spec-viewer` (reuse) |
| Change viewer (`generate_change_viewer_html`) | `@cclab/ui/change-viewer` (new) |
| Project cclab tree (`/{project}/cclab`) | `@cclab/ui/project-tree` (new) |

All components should:
- Accept typed props generated from `packages/cclab-agkit/schemas/`
- Render from structured data, not from server-rendered HTML
- Work in both Conductor's React app (authenticated, multi-user) and Score's optional `score view` (local, single-user)
- Share styling with existing `@cclab/ui` primitives

## Data API

Both Conductor and Score need to expose JSON APIs that serve the Artifact data for these components to consume. The APIs should match — Conductor exposes them via FastAPI + PG-backed `ArtifactStore`; Score exposes them via a minimal HTTP server + filesystem-backed `ArtifactStore`.

Suggested endpoint shape:
```
GET /api/projects                          → project list
GET /api/projects/{id}/artifacts?kind=...  → artifact list by kind
GET /api/projects/{id}/artifacts/{aid}     → single artifact
GET /api/projects/{id}/artifacts/{aid}/lineage  → forward/reverse lineage
```

## After extraction — what happens to cclab-server?

Post-extraction, `cclab-server` becomes near-empty:
- Registry lookup (stays)
- Health check (stays)
- HTTP server bootstrap (stays, but trivial)
- All HTML-serving handlers (go away — React components serve these now)

At that point, `cclab-server` may be shrunk further or absorbed into `projects/score/mcp/` (the optional local server Score launches for `score view`).

## Depends on

- `cclab-agkit` codegen pipeline (#1182) — needs TypeScript interface codegen so components have typed props
- Unified `ArtifactDB` (#1172) — components consume `Artifact<kind>` uniformly

## Non-goals

- Full design overhaul — components should match existing visual style
- Server-side authentication / RBAC — that lives in Conductor's FastAPI layer (#1147), not in the components

## Why P2

Not P1 because current cclab-server dashboard still works for the immediate use case (viewing changes locally). But P2 because:
- Conductor's React frontend will want these components soon
- Duplicate implementations (server-rendered HTML + React) create drift
- Score's future `score view` command expects the React path

## Cross-references

- #1157 — Score project epic
- #1158 — Conductor rewires onto arsenal
- #1101 — Conductor thin shell refactor
- #1097 — @cclab/spec-viewer package (existing, will extend)
- #1098 — @cclab/pipeline package (existing, related)
