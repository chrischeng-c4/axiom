---
change_id: genesis-fetch-issues
type: gap_codebase_knowledge
created_at: 2026-02-12T02:36:23.662054+00:00
updated_at: 2026-02-12T02:36:23.662054+00:00
---

# Gap Analysis: Codebase vs Knowledge

## Convention Violations

- **File**: `crates/cclab-genesis/src/services/platform_sync/payload.rs`, `config.rs`
- **Knowledge Ref**: Project Convention RS101 (Robustness)
- **Gap**: Prism diagnostics identified multiple `.unwrap()` calls in these files. This violates the project's robustness convention which favors explicit error handling over panics in core logic.
- **Severity**: Medium

## Pattern Mismatches

### 1. Stage-Specific MCP Tool Filtering
- **File**: `crates/cclab-genesis/src/mcp/tools/platform_sync.rs`
- **Knowledge Ref**: `40-mcp/dynamic-config.md` (Tool Filtering by Stage)
- **Gap**: The tool is currently implemented without integration into the stage-specific toolsets defined in the knowledge base. This mismatch increases LLM cognitive load and deviates from the strategy of providing specialized tool sets for Plan, Implement, etc.
- **Severity**: Medium

### 2. HTTP Transport Adoption
- **File**: `crates/cclab-genesis/src/mcp/tools/platform_sync.rs`
- **Knowledge Ref**: `40-mcp/http-server.md` (Architecture)
- **Gap**: The tool implementation relies on standard stdio transport, whereas the knowledge base documentation promotes the move to HTTP transport to resolve transport-level buffering and hanging issues.
- **Severity**: Low

### 3. Project Isolation and Registry Integration
- **File**: `crates/cclab-genesis/src/services/platform_sync/config.rs`
- **Knowledge Ref**: `40-mcp/http-server.md` (Key Features / Global Project Registry)
- **Gap**: The `PlatformConfig` logic does not currently utilize the `Global Project Registry` or project isolation headers (`X-Genesis-Project`). This mismatch prevents the platform sync service from correctly identifying repository context in multi-project environments as documented in the MCP server architecture.
- **Severity**: High
