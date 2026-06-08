---
change: sdd-workflow-cleanup
group: config-unification
date: 2026-03-17
status: answered
---

# Pre-Clarifications

### Q1: General
- **Question**: If label_prefix is defined, can it be overridden per step with an absolute label?
- **Answer**: Yes, per-step label overrides label_prefix. If label starts with a specific prefix it's absolute, otherwise label_prefix is prepended. But labels are not used by the workflow yet — this is future-proofing, keep it simple for now.

### Q2: General
- **Question**: In migrate_config, should we automatically delete the old sections or move them to a backup/commented state?
- **Answer**: Delete the old sections. We already have version tracking in config.toml, and git history preserves the old config.

