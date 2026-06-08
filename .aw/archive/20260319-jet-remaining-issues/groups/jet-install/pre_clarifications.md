---
change: jet-remaining-issues
group: jet-install
date: 2026-03-19
status: answered
---

# Pre-Clarifications

### Q1: General
- **Answer**: Keep 5min default. Add --no-cache flag (already implemented). No user-configurable TTL for now — keep it simple.

### Q2: General
- **Answer**: ALPN negotiation (already using http2_adaptive_window on reqwest). Silent fallback to HTTP/1.1. No http2_prior_knowledge.

### Q3: General
- **Answer**: Only nx.rs get_project_graph() calls nx CLI. Replace with direct project.json scanning: read nx.json for project patterns, glob for project.json files, parse root/targets. Handle plain npm workspaces (no project.json) via existing workspace.rs.

