---
change_id: pylibs-refactor
type: gap_codebase_spec
created_at: 2026-02-24T10:00:59.955255+00:00
updated_at: 2026-02-24T10:00:59.955255+00:00
---

# Gap Analysis: Codebase vs Specs

| Component | Target Change | Existing Spec(s) | Gap | Severity | Action Needed |
|-----------|---------------|------------------|-----|----------|---------------|
| **Binding Architecture** | Decentralization (#81, #82, #85) | `nucleus-architecture` | Specs point to `cclab-nucleus` but code is moving/moved to individual crates (pg, queue, mongo). | Medium | Update specs to reflect decentralized path. |
| **cclab-pg** | Split crud.rs (#81) | Issue #81 | Spec is 'open' and task-oriented, but implementation is already complete and decentralized. | Low | Close spec #81. |
| **cclab-queue** | Split tasks.rs (#82) | Issue #82 | `mod.rs` is still 924 lines; splitting implementation is missing. | High | Perform split. |
| **cclab-mongo** | Split document.rs (#85) | Issue #85 | `document.rs` is still 728 lines; splitting implementation is missing. | High | Perform split. |
| **cclab-titan** | P0 Test Coverage (#135) | Issue #135 | 0 tests for pool, constraints, cascade, upsert despite detailed test plan. | High | Implement P0 tests. |
| **cclab-quasar** | FastAPI Parity (#138) | Issue #138 | Router, Middleware, and WS server plumbing are specified but missing from code. | Medium | Implement missing features. |
| **cclab-fetch** | Migration (#455) | Issue #455 | Both `cclab-http` and `cclab-fetch` exist; cleanup and dependency updates are incomplete. | High | Complete migration/cleanup. |
| **cclab-shield** | Performance (#189) | Issue #189 | JSON-to-model bottleneck persists; suggested optimizations are not implemented. | Medium | Implement optimizations. |
