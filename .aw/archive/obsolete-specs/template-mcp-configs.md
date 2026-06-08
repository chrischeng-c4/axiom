---
id: mcp-configs-template
type: spec
title: "MCP Configuration Templates"
version: 1
spec_type: utility
spec_group: cclab-sdd
created_at: 2026-02-23T00:00:00+00:00
updated_at: 2026-02-23T00:00:00+00:00
requirements:
  total: 7
  ids:
    - R1
    - R2
    - R3
    - R4
    - R5
    - R6
    - R7
---

# MCP Configuration Templates

## Overview

Four MCP configuration files connect LLM clients (Claude Code, Gemini CLI, Codex CLI) to the `cclab-mcp` server. Each client has its own config format and location. The `.example` template files in `crates/cclab-sdd/templates/mcp-configs/` serve as documentation only — actual config content is constructed programmatically in `config.rs`.

**Implementation**: `crates/cclab-sdd/src/mcp/config.rs`

| Client | Config File | Format |
|--------|------------|--------|
| Claude Code | `.mcp.json` | JSON |
| Claude Code | `.claude/settings.local.json` | JSON |
| Gemini CLI | `.gemini/settings.json` | JSON |
| Codex CLI | `.codex/config.toml` | TOML |

## Templates

### Claude Code — `.mcp.json`

```json
{
  "mcpServers": {
    "cclab-mcp": {
      "type": "http",
      "url": "http://localhost:3456/mcp"
    }
  }
}
```

### Claude Code — `.claude/settings.local.json`

```json
{
  "enabledMcpjsonServers": [
    "cclab-mcp"
  ]
}
```

### Gemini CLI — `.gemini/settings.json`

```json
{
  "mcp": {
    "allowed": [
      "cclab-mcp"
    ]
  },
  "mcpServers": {
    "cclab-mcp": {
      "type": "http",
      "url": "http://localhost:3456/mcp",
      "excludeTools": ["sdd_delegate_agent"]
    }
  }
}
```

### Codex CLI — `.codex/config.toml`

```yaml
mcp_servers:
  cclab-mcp:
    type: http
    url: http://localhost:3456/mcp
    disabled_tools: [sdd_delegate_agent]
```

## Installation

### R1 - Programmatic Construction (Not Embedded)

```yaml
id: R1
priority: high
status: draft
```

Unlike skill templates, MCP configs are NOT embedded via `include_str!()`. Config content is constructed programmatically in `config.rs` using `serde_json` (JSON) and `toml` (TOML) crates. The `.example` files in `templates/mcp-configs/` exist solely as human-readable documentation.

**Implementation functions**:
- `ensure_claude_mcp_json()` — writes `.mcp.json`
- `ensure_claude_settings()` — writes `.claude/settings.local.json`
- `ensure_gemini_mcp_config()` — writes `.gemini/settings.json`
- `ensure_codex_mcp_config()` — writes `.codex/config.toml`

## Merge Behavior

### R2 - Deep Merge (JSON)

```yaml
id: R2
priority: high
status: draft
```

For `.mcp.json`, `.claude/settings.local.json`, and `.gemini/settings.json`:

1. Parse existing file as JSON (or start with `{}` if missing)
2. Add/overwrite the `cclab-mcp` entry under `mcpServers` — preserves all other user-configured servers
3. Write back with `serde_json::to_string_pretty()`

This ensures user-added CLI servers (e.g. `playwright`, `filesystem`) are never removed.

### R3 - Deep Merge (TOML)

```yaml
id: R3
priority: high
status: draft
```

For `.codex/config.toml`:

1. Parse existing file as TOML (or start with empty table if missing)
2. Add/overwrite `[mcp_servers.cclab-mcp]` — preserves all other user-configured servers
3. Write back as TOML

Same merge semantics as JSON but in TOML format.

## Legacy Migration

### R4 - Legacy Entry Removal

```yaml
id: R4
priority: medium
status: draft
```

On each `cclab init`, old `"cclab"` entries are removed and replaced with `"cclab-mcp"`:

| File | Migration |
|------|-----------|
| `.mcp.json` | Remove `mcpServers.cclab`, add `mcpServers.cclab-mcp` |
| `.gemini/settings.json` | Remove `mcpServers.cclab`, remove `"cclab"` from `mcp.allowed`, add `"cclab-mcp"` |
| `.codex/config.toml` | Remove `[mcp_servers.cclab]`, add `[mcp_servers.cclab-mcp]` |

### R5 - Server Name Rationale

```yaml
id: R5
priority: medium
status: draft
```

The server is named `cclab-mcp` (not `cclab`) because Claude Code blocks CLI servers that share a name with installed CLI tools. Since `cclab` is the CLI binary name, using it as the CLI server name would cause Claude Code to silently ignore the server.

## Recursion Prevention

### R6 - Agent Delegation Tool Exclusion

```yaml
id: R6
priority: high
status: draft
```

The `sdd_delegate_agent` tool is excluded from non-mainthread clients to prevent recursive agent calls (e.g. Gemini calling delegate_agent which calls Gemini again):

| Client | Mechanism | Config Key |
|--------|-----------|------------|
| Gemini | Server-level tool exclusion | `"excludeTools": ["sdd_delegate_agent"]` |
| Codex | Server-level tool disabling | `disabled_tools = ["sdd_delegate_agent"]` |
| Claude | CLI dispatch flag (handled externally) | N/A in config |

## Edge Cases

### R7 - Settings Skip for All-Enabled

```yaml
id: R7
priority: low
status: draft
```

If `.claude/settings.local.json` already has `"enableAllProjectMcpServers": true`, the `ensure_claude_settings()` function skips adding `"cclab-mcp"` to `enabledMcpjsonServers`. The global flag already enables all servers, so the specific entry is unnecessary.
