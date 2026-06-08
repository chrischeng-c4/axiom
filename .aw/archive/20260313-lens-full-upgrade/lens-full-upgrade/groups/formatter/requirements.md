---
change: lens-full-upgrade
group: formatter
date: 2026-03-13
---

# Requirements

Unified formatter interface wrapping external tools: rustfmt (Rust), prettier (JS/TS/HTML/CSS/JSON/YAML/Markdown/GraphQL), gofmt (Go), black (Python), terraform fmt (HCL). Detection: check if formatter binary is in PATH. Format-on-save in LSP. MCP tool: lens_format. Diff mode: show what would change without applying.
