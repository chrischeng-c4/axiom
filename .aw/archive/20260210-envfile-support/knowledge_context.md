---
change_id: envfile-support
type: knowledge_context
created_at: 2026-02-10T02:27:44.840950+00:00
updated_at: 2026-02-10T02:27:44.840950+00:00
iteration: 1
complexity: medium
stage: knowledge
scanned_categories:
  - 05-titan
  - 178-grid-db-refactor-changelog.md
  - 30-claude
  - 40-mcp
  - changelogs
  - grid
  - orbit
---

# Knowledge Context

## Relevant Documents

- **40-mcp/dynamic-config.md**
  - summary: Describes how Genesis manages MCP tool configuration for different stages, which might be relevant if envfiles are passed as CLI args.
  - relevant sections: Claude Code Integration (MCP server command and args)

## Patterns

- **Dotenv Implementation Pattern** (source: cclab-shield/shield-settings-management spec)
  - Use of 'dotenvy' in Rust and 'python-dotenv' in Python for .env file support.

## Pitfalls

- Variable substitution needs to be handled correctly to match industry standards (dotenv).
- mainthread does not support envfiles as it runs in the host process.
