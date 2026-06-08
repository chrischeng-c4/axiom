---
change_id: sdd-merge
type: gap_codebase_spec
created_at: 2026-02-15T03:38:08.395774+00:00
updated_at: 2026-02-15T03:38:08.395774+00:00
---

# Gap Analysis: Codebase vs Spec (sdd-merge)

## Summary of Gaps

| Gap ID | Type | Description | Severity |
|--------|------|-------------|----------|
| G1 | Code without Spec | Redundant Registry implementation in both `cclab-genesis` and `cclab-server`. | High |
| G2 | Code without Spec | Unified MCP Router (`cclab-server/src/mcp/router.rs`) missing PDG tools (`prism_pdg`). | High |
| G3 | Spec without Impl | `prism-codegen-unification` (unifying Aurora tools into Prism/SDD) is not reflected in the router or crate dependencies. | High |
| G4 | Code without Spec | Duplicate HTTP server implementations in `cclab-server` and `cclab-genesis`. | Medium |
| G5 | Spec without Impl | `migration-architecture` R3/R4 (removal of Aurora relay) is not implemented; legacy calls persist. | Medium |
| G6 | Code without Spec | Redundant CLI server commands in `cclab-server` and `cclab-genesis`. | Low |

## Detailed Gaps

### G1: Redundant Registry Implementations
- **Source:** `crates/cclab-genesis/src/mcp/registry.rs` and `crates/cclab-server/src/registry.rs`
- **Issue:** Both files define identical `Registry` and `ProjectInfo` symbols.
- **Severity:** High
- **Context:** Duplicate definitions of `Registry` and `ProjectInfo` exist across multiple crates without a unifying specification.

### G2: Missing PDG Tools in Unified Router
- **Source:** `crates/cclab-server/src/mcp/router.rs`
- **Issue:** `prism_impact` results confirm `prism_pdg` is missing from the `UnifiedMcpRouter`.
- **Severity:** High
- **Context:** The `UnifiedMcpRouter` lacks integration with `prism_pdg` tools, resulting in reduced functionality compared to individual Prism components.

### G3: Unimplemented Crate Unification
- **Source:** `prism-codegen-unification` (Spec)
- **Issue:** Code still contains `call_aurora_tool` and depends on `cclab-aurora`, contradicting the spec's goal of merging these into the genesis/sdd workflow.
- **Severity:** High
- **Context:** Existing code maintains dependencies on `cclab-aurora` and uses `call_aurora_tool`, which is inconsistent with the requirement to merge these into the SDD workflow.

### G4: Duplicate HTTP Server Logic
- **Source:** `crates/cclab-server/src/http_server.rs` and `crates/cclab-genesis/src/mcp/http_server.rs`
- **Issue:** Overlapping `handle_mcp_request` and `start_server` logic.
- **Severity:** Medium
- **Context:** Shared logic for `handle_mcp_request` and `start_server` is present in both crates, indicating a split in the HTTP server implementation.

### G5: Aurora Relay Removal
- **Source:** `migration-architecture` R3/R4 (Spec)
- **Issue:** The codebase still relies on the Aurora relay logic instead of the pure YAML IR flow defined in the spec.
- **Severity:** Medium
- **Context:** Codebase continues to utilize Aurora relay logic, which deviates from the YAML IR flow specified in `migration-architecture` R3/R4.

### G6: Redundant CLI Server Commands
- **Source:** `crates/cclab-server/src/cli.rs` and `crates/cclab-genesis/src/cli/server.rs`
- **Issue:** Both files implement CLI subcommands for starting, stopping, and managing the unified server.
- **Severity:** Low
- **Context:** Redundant CLI implementations exist for server management across different crates.
