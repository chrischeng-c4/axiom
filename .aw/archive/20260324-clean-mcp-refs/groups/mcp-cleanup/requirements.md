---
change: clean-mcp-refs
group: mcp-cleanup
date: 2026-03-24
---

# Requirements

Bulk refactor of 28 spec files under cclab/specs/crates/cclab-sdd/. Three categories of changes:

1. **files: frontmatter** — update mcp/tools/*.rs paths to actual source paths (tools/*.rs, services/*.rs)
2. **MCP terminology** — replace 'MCP tool' with 'CLI command' or 'tool', remove 'MCP server' references
3. **Obsolete spec** — evaluate generate/template-mcp-configs.md for archival

No code changes. Spec-only refactor.
