---
change: lens-beyond-ide
group: rpc-mcp-wiring
date: 2026-03-13
status: answered
---

# Pre-Clarifications

### Q1: Search index lifecycle
- **Answer**: Eager on start — build index during daemon startup. Slower start but instant first query.

### Q2: Refactor preview
- **Answer**: Support both via a dry_run parameter. Default is preview (dry_run=true), apply with dry_run=false.

