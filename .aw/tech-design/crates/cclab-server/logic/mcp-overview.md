---
id: cclab-server-mcp-overview
main_spec_ref: "crates/cclab-server/src/mcp"
merge_strategy: new
fill_sections: [overview]
---

# MCP Configuration Overview

## Overview
<!-- type: overview lang: markdown -->

Model Context Protocol servers provide structured tools to LLM clients. The
cclab-server MCP surface is split into protocol/runtime behavior and stage
configuration contracts.

The old index lived at `.aw/tech-design/crates/cclab-server/mcp/index.md`.
The active overview is now under `logic/` because `mcp/` is not an allowed
top-level TD directory.

### Related Contracts

| Contract | Current path |
|----------|--------------|
| Dynamic stage tool filtering | `.aw/tech-design/crates/cclab-server/config/dynamic-mcp-config.md` |
| Claude Code runtime MCP config | `.aw/tech-design/crates/cclab-server/config/claude-code-mcp.md` |
| HTTP MCP server behavior | `.aw/tech-design/crates/cclab-server/interfaces/mcp/http-server.md` |

### Stage Tool Surface

| Stage | MCP tools needed |
|-------|------------------|
| Plan | all core and diagram tools |
| Implement | implementation read tools |
| Review | validation, review append, and file read tools |
| Archive | knowledge and file read tools |

Dynamic MCP configuration exists to reduce prompt overhead, narrow tool choice,
and prevent a stage from accidentally calling unrelated planning or archive
tools.
