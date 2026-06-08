---
id: improve-quasar-maturity
type: proposal
version: 1
created_at: 2026-01-28T07:31:44.180378+00:00
updated_at: 2026-01-28T07:31:44.180378+00:00
author: mcp
status: proposed
iteration: 1
summary: "Upgrade cclab-quasar to 95% maturity with automated DI, interactive docs, and robust testing utilities."
history:
  - timestamp: 2026-01-28T07:31:44.180378+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
  - timestamp: 2026-01-28T07:35:41.748059+00:00
    agent: "gemini-3-flash-preview"
    tool: "revise_proposal"
    action: "revised"
    duration_secs: 510.95
  - timestamp: 2026-01-28T07:36:55.315267+00:00
    agent: "gemini-3-flash-preview"
    tool: "review_proposal"
    action: "reviewed"
    duration_secs: 73.56
impact:
  scope: minor
  affected_files: 15
  new_files: 1
affected_specs:
  - id: quasar-di
    path: specs/quasar-di.md
    depends: []
  - id: quasar-docs
    path: specs/quasar-docs.md
    depends: []
  - id: quasar-lifespan
    path: specs/quasar-lifespan.md
    depends: []
  - id: quasar-test-client
    path: specs/quasar-test-client.md
    depends: []
  - id: quasar-test-expansion
    path: specs/quasar-test-expansion.md
    depends: []
  - id: quasar-maturity-upgrade
    path: specs/quasar-maturity-upgrade.md
    depends: []---

<proposal>

# Change: improve-quasar-maturity

## Summary

Upgrade cclab-quasar to 95% maturity with automated DI, interactive docs, and robust testing utilities.

## Why

To reach 95% maturity as a FastAPI alternative, cclab-quasar needs key features like automated DI, interactive docs, and robust testing utilities that developers expect from a modern API framework. The current implementation lacks automated dependency resolution for handlers and doesn't provide built-in interactive docs or an in-process test client, which are critical for developer productivity.

## What Changes

- Implement graph-resolving Dependency Injection for route handlers (FastAPI-style Depends) in dependency.rs and handler.rs
- Add interactive documentation serving for Swagger UI and ReDoc in openapi.rs and server.rs
- Integrate Lifespan Events (startup/shutdown) into the Server run loop in lifecycle.rs and server.rs
- Create a comprehensive in-process TestClient in src/testing.rs for integration testing without TCP binding
- Expand test suite coverage for middleware chains, WebSockets, and SSE keep-alive
- Export new maturity features in lib.rs and add FastAPI migration guide to documentation

## Impact

- **Scope**: minor
- **Affected Files**: ~15
- **New Files**: ~1
- Affected specs:
  - `quasar-di` (no dependencies)
  - `quasar-docs` (no dependencies)
  - `quasar-lifespan` (no dependencies)
  - `quasar-test-client` (no dependencies)
  - `quasar-test-expansion` (no dependencies)
  - `quasar-maturity-upgrade` (no dependencies)
- Affected code: `crates/cclab-quasar/src/dependency.rs`, `crates/cclab-quasar/src/openapi.rs`, `crates/cclab-quasar/src/lifecycle.rs`, `crates/cclab-quasar/src/server.rs`, `crates/cclab-quasar/src/handler.rs`, `crates/cclab-quasar/src/router.rs`, `crates/cclab-quasar/src/testing.rs`, `crates/cclab-quasar/src/lib.rs`
- **Breaking Changes**: Minor changes to internal handler traits might be required to support automated DI resolution.

</proposal>

<review iteration="1" reviewer="gemini-agent" status="needs_revision">
## Summary
The proposal is well-motivated and the high-level goals are clear. However, the technical design (specs) and implementation plan (tasks) have major deficiencies.

## Issues

### HIGH: Incorrect Task Structure and File Paths
- `tasks.md` uses generic paths like `src/logic/` and `src/api/` which do not exist in the project. The correct path is `crates/cclab-quasar/src/`.
- Tasks propose to `CREATE` new files for logic that clearly belongs in existing files (e.g., DI should be in `dependency.rs` and `handler.rs`, lifespan in `server.rs` and `lifecycle.rs`).
- The task list seems auto-generated without regard for the "Impact" section or the actual codebase structure.

### MEDIUM: Missing Technical Detail in Specs
- All specifications lack the "Interfaces" section (pseudo-code `FUNCTION` definitions).
- All specifications lack "Data Model" sections (JSON Schemas for configuration or state).
- Requirements are very brief and lack the specificity needed for implementation.

### LOW: Consistency
- The `impact` section of the proposal correctly identifies the files to be modified, but the `tasks.md` ignores this information.

## Verdict
NEEDS_REVISION

## Next Steps
1. **Regenerate Tasks**: Update `tasks.md` to use the correct file paths (`crates/cclab-quasar/src/...`) and correctly distinguish between `MODIFY`ing existing files and `CREATE`ing new ones (like `testing.rs`).
2. **Enhance Specifications**: Add "Interfaces" and "Data Model" sections to all specs. Provide specific pseudo-code for the new APIs (e.g., `TestClient` methods, `Depends` resolution logic, `Lifespan` registration).
3. **Align Requirements**: Ensure the requirements in the specs are detailed enough to be testable and actionable.
</review>
