---
change: unified-server
date: 2026-01-24
---

# Clarifications

## Q1: LSP Transport and Port
- **Question**: Should the unified server listen for LSP connections on a separate TCP port (defaulting to 5007) instead of using WebSocket or stdio?
- **Answer**: Yes, listening on a separate TCP port is the most standard and compatible way for an LSP server to operate in a multi-project environment. Port 5007 is a reasonable default.
- **Rationale**: TCP provides better compatibility with existing LSP clients (VS Code, Vim, Emacs) when the server is already running as a background daemon.

