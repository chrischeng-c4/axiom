---
change: 1145
group: sdd-docs-phase
date: 2026-04-05
status: answered
---

# Pre-Clarifications

### Q1: General
- **Question**: Should the doc-writer agent use a dedicated agent type (sdd-doc-writer) or reuse existing agent types?
- **Answer**: Dedicated sdd-doc-writer agent type with doc-specific prompt template and tools.

### Q2: General
- **Question**: For the review checklist: should doc accuracy be verified against actual CLI output (run commands), or just against spec content?
- **Answer**: Run CLI commands to verify — actually execute commands and compare output for accuracy.

### Q3: General
- **Question**: Should docs be committed as part of the change archive, or written directly to output_dir in the repo?
- **Answer**: Directly to output_dir. Default is project_root/docs, configurable via [sdd.docs].output_dir in config.toml.

