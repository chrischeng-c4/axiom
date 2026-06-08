---
change_id: sdd-merge
type: gap_codebase_knowledge
created_at: 2026-02-15T03:41:12.569155+00:00
updated_at: 2026-02-15T03:41:12.569155+00:00
---

# Gap Analysis: Codebase vs Knowledge for sdd-merge

## Convention Violations

- **Redundant Registry Implementation**
  - **File:** `crates/cclab-genesis/src/mcp/registry.rs`
  - **Ref:** `knowledge:40-mcp/http-server.md#registry-file`
  - **Description:** The `cclab-genesis` crate retains a full implementation of `Registry`, `ProjectInfo`, and `ServerInfo` that duplicates the intended source of truth in `crates/cclab-server/src/registry.rs`.
  - **Severity:** HIGH

- **Duplicate HTTP Server Logic**
  - **File:** `crates/cclab-genesis/src/mcp/http_server.rs`
  - **Ref:** `knowledge:40-mcp/http-server.md#architecture`
  - **Description:** `cclab-genesis` maintains its own `start_server` and `handle_mcp_request` logic, which should be merged into the unified implementation in `crates/cclab-server/src/http_server.rs`.
  - **Severity:** HIGH

- **Crate Naming Inconsistency**
  - **File:** `crates/cclab-genesis/`
  - **Ref:** `knowledge:index.md` (Knowledge Pitfalls)
  - **Description:** The codebase has not yet executed the planned rename to `cclab-sdd`, which will require recursive dependency updates as noted in the project conventions.
  - **Severity:** LOW

## Pattern Mismatches

- **Incomplete Unified MCP Routing**
  - **File:** `crates/cclab-server/src/mcp/router.rs`
  - **Pattern:** Unified MCP tool router
  - **Source:** `knowledge:40-mcp/dynamic-config.md`
  - **Description:** The unified router fails to expose the PDG (Program Dependence Graph) tools defined in `crates/cclab-prism/src/mcp/tools.rs`, creating a functional gap for downstream clients.
  - **Severity:** HIGH

- **SpecIR Pipeline Non-Compliance**
  - **File:** `crates/cclab-aurora/src/lib.rs` (and related generators)
  - **Pattern:** Agnostic SpecIR pipeline
  - **Source:** `knowledge:spec-to-code/index.md`
  - **Description:** Legacy generators remain coupled to specific formats (JSON Schema/OpenAPI) rather than consuming the Agnostic SpecIR as documented in the generator contract.
  - **Severity:** MEDIUM

- **Merge Logic Manifest Ignoring**
  - **Area:** Server/Registry loading
  - **Pattern:** Direct file-based YAML IR generation
  - **Source:** `knowledge:genesis-372-impact.md`
  - **Description:** Current merge logic focuses on markdown artifacts, ignoring the `spec_ir/*.yaml` manifests required for main spec consistency.
  - **Severity:** MEDIUM
