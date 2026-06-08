---
id: 484
type: proposal
version: 2
created_at: 2026-02-24T02:57:28.106725+00:00
updated_at: 2026-02-24T02:57:28.106725+00:00
iteration: 1
scope: minor
spec_plan:
  - id: mcp-session-binding
    title: "MCP Session-based Project Binding"
    depends: []
    context_refs:
    affected_code: ["crates/cclab-server/src/http_server.rs", "crates/cclab-server/src/mcp/router.rs"]
  - id: dynamic-tool-schema
    title: "Dynamic tools/list Schema Based on Session"
    depends: [mcp-session-binding]
    context_refs:
    affected_code: ["crates/cclab-server/src/mcp/router.rs", "crates/cclab-sdd/src/mcp/tools/mod.rs"]
  - id: init-mcp-json
    title: "Generate .mcp.json with Project Header in cclab init"
    depends: []
    context_refs:
    affected_code: ["crates/cclab-sdd/src/cli/init.rs"]
history:
  - timestamp: 2026-02-24T02:57:28.106725+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
---

<proposal>

# Spec Navigation Map: 484

## Scope Overview (Mindmap)

```mermaid
mindmap
  root((484))  
    Server
      session binding
      dynamic schema
      project_path resolution
    CLI
      init
      mcp.json generation
      .gitignore
```

## Spec Dependency Graph (Block Diagram)

```mermaid
block-beta
  columns 3

  mcp_session_binding["mcp-session-binding"]
  dynamic_tool_schema["dynamic-tool-schema"]
  init_mcp_json["init-mcp-json"]

  mcp_session_binding --> dynamic_tool_schema
```

## Spec Execution Order

1. **init-mcp-json** — Generate .mcp.json with Project Header in cclab init
   - code: crates/cclab-sdd/src/cli/init.rs
2. **mcp-session-binding** — MCP Session-based Project Binding
   - code: crates/cclab-server/src/http_server.rs, crates/cclab-server/src/mcp/router.rs
3. **dynamic-tool-schema** — Dynamic tools/list Schema Based on Session
   - depends: mcp-session-binding
   - code: crates/cclab-server/src/mcp/router.rs, crates/cclab-sdd/src/mcp/tools/mod.rs

</proposal>
