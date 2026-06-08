---
change: spec-consolidation
group: spec-consolidation-enforcement
date: 2026-03-23
status: answered
---

# Pre-Clarifications

### Q1: General
- **Question**: Migration tool scope — include existing scattered specs migration?
- **Answer**: Include all. Migration for all scattered crates (cclab-agent, cclab-runtime, cclab-jet, cclab-lens, cclab-qc, cclab-server, etc.) is in scope for this change, including the 4 loose root files in cclab-sdd.

### Q2: General
- **Question**: Validation failure mode — hard error or warning?
- **Answer**: Hard error. validate-spec-structure, spec_plan path validation, and merge-time validation all block the pipeline on failure. No lenient mode.

### Q3: General
- **Question**: scaffold-spec output format?
- **Answer**: Create directory structure + README.md at spec root. No .gitkeep or stub files.

### Q4: General
- **Question**: Spec directory listing format in reference_context prompt?
- **Answer**: ASCII tree format (like `tree` output), paths only — no content summaries. Human and agent readable.

