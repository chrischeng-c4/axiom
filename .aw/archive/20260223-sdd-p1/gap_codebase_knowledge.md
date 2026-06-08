---
change_id: sdd-p1
type: gap_codebase_knowledge
created_at: 2026-02-23T14:24:40.108959+00:00
updated_at: 2026-02-23T14:24:40.108959+00:00
---

# Gap Analysis: Codebase vs Knowledge

## High Severity

### Hardcoded workflow logic in `dag_loop.rs`
- **Gap:** The `dag_loop.rs` file contains hardcoded routing for `explore_codebase` and hardcoded phase definitions.
- **Violation:** This contradicts the "Pure state machine + prompt provider" pattern intended for the unified workflow orchestration in `sdd_run_change`.
- **Source:** `knowledge:40-mcp/dynamic-config.md`, Unified Workflow Tool (sdd_run_change) pattern.

## Medium Severity

### Inconsistent Review Checklist Implementation
- **Gap:** Review tool implementations (e.g., `explore_knowledge.rs`, `explore_codebase.rs`) show evidence of manual checklist consistency management.
- **Violation:** Mismatch with the "Unified Artifact Management" pattern which expects a centralized or unified review logic rather than per-tool manual consistency.
- **Source:** Unified Artifact Management pattern, `knowledge:40-mcp/dynamic-config.md`.

### Missing HTTP Server/Header Integration
- **Gap:** Current tool symbols do not show explicit integration for the `X-SDD-Project` header or multi-project isolation required by the HTTP server architecture.
- **Violation:** Violates the "HTTP MCP Server" architecture pattern designed for pipe safety and isolation.
- **Source:** `knowledge:40-mcp/http-server.md`, HTTP MCP Server pattern.

### Legacy Spec Feedback Fields
- **Gap:** Implementation notes for gap analysis and proposal tools still reference legacy feedback fields (verdict, checklist, missing fields).
- **Violation:** Does not align with the "Migration strategy to YAML-based spec IR" intended to eliminate token relay overhead.
- **Source:** `knowledge:genesis-372-impact.md`.
