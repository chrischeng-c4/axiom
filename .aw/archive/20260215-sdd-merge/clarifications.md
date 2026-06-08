---
change: sdd-merge
date: 2026-02-15
---

# Clarifications

## Q1: Rename scope
- **Question**: Crate rename or module-level merge?
- **Answer**: Full crate rename: cclab-genesis → cclab-sdd. Merge cclab-aurora into the renamed crate.
- **Rationale**: Clean break — the crate name reflects its new purpose as the unified SDD (Spec-Driven Development) system.

## Q2: MCP tool prefix
- **Question**: aurora_generate_* tool naming after migration?
- **Answer**: sdd_generate_* — rename all 21 Aurora tools from aurora_generate_* to sdd_generate_*.
- **Rationale**: Consistent with the new crate identity. Breaking change for agent prompts but cleaner long-term.

## Q3: Git workflow
- **Question**: Which git workflow?
- **Answer**: in_place — work on current sdd branch.
- **Rationale**: Already on sdd branch, no need for a separate branch.

## Q4: Affected modules
- **Question**: Which crates/paths affected?
- **Answer**: cclab-aurora (remove), cclab-genesis (rename to cclab-sdd), cclab-cli (update deps), cclab-server (update deps), Cargo.toml workspace, CLAUDE.md, all crates importing cclab-genesis or cclab-aurora.
- **Rationale**: Crate rename + merge affects the entire dependency graph.

