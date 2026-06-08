---
change: lens-full-upgrade
group: auto-fix
date: 2026-03-13
---

# Requirements

Add quick-fix capability to diagnostics. Each Diagnostic can carry optional Fix (text edits). Implement auto-fix for common patterns: unused imports (remove), var→const/let (JS), == to === (JS), trailing whitespace (remove), missing semicolons. LSP code action integration. MCP tool: lens_fix for applying fixes.
