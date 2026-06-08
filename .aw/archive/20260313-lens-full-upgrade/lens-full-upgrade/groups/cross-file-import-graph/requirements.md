---
change: lens-full-upgrade
group: cross-file-import-graph
date: 2026-03-13
---

# Requirements

Build project-wide dependency graph from import/require/use statements. Track: file→file edges, circular dependency detection, unused file detection (no inbound edges from entry points), import resolution for Python (relative + absolute), TypeScript (paths + baseUrl), Rust (mod/use), Go (package paths). Expose via MCP tool: lens_import_graph.
