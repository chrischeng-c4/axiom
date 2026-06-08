---
number: 484
title: "feat(sdd): session-based project isolation for shared HTTP MCP server"
state: open
labels: [enhancement, crate:sdd]
---

# #484 — feat(sdd): session-based project isolation for shared HTTP MCP server

## Problem

`cclab-mcp` HTTP server is shared across multiple projects. Each tool call relies on the `project_path` parameter for isolation. If a client passes the wrong `project_path`, change artifacts get written to the wrong project's `cclab/changes/` directory.

Real case: `280/`, `281/`, `crypto-qr/` change dirs from another project leaked into the current repo.

## Design

### Header-first, param-fallback with dynamic schema

**Server lifecycle:**

```
initialize:
  X-Cclab-Project header 存在 → 綁定 session.project_path
  不存在 → session.project_path = null

tools/list:
  session.project_path 存在 → project_path 從 required 移除（client 看不到）
  session.project_path 為 null → project_path 保持 required（現行行為）

tool call:
  優先用 session.project_path
  沒有才看 tool args 的 project_path
  都沒有 → error
```

**Client experience:**

| Config | `project_path` visible? | Behavior |
|--------|------------------------|----------|
| Has header | Hidden from tool schema | Auto-injected, zero noise |
| No header | Required param | Backwards compatible, current behavior |

### `.mcp.json` config

```json
{
  "cclab-mcp": {
    "type": "http",
    "url": "http://localhost:3456/mcp",
    "headers": {
      "X-Cclab-Project": "/path/to/project"
    }
  }
}
```

Claude Code sends `headers` on **every** HTTP request including `initialize`.

### `.mcp.json` should be gitignored

Since `.mcp.json` contains local-specific config (paths, ports, headers):

| File | Purpose | Git |
|------|---------|-----|
| `.mcp.json` | Local runtime config | `.gitignore` |
| `.mcp.json.example` | Template for reference | tracked |

### `cclab init` changes

- Generate `.mcp.json` with `headers.X-Cclab-Project` set to `$PWD`
- Add `.mcp.json` to `.gitignore`
- Track `.mcp.json.example` for onboarding

## Tasks

- [ ] Server: read `X-Cclab-Project` from request headers on `initialize`, bind to session
- [ ] Server: dynamic `tools/list` — hide `project_path` from required when session has bound project
- [ ] Server: tool call resolution — session.project_path > args.project_path > error
- [ ] `cclab init`: add `headers.X-Cclab-Project` to generated `.mcp.json`
- [ ] `cclab init`: add `.mcp.json` to `.gitignore`, track `.mcp.json.example`
- [ ] Docs: update MCP config spec
