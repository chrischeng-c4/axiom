---
change: 1140
group: check-alignment-phase1
date: 2026-04-03
status: answered
---

# Pre-Clarifications

### Q1: General
- **Question**: Should check-alignment be a new CLI subcommand or extend existing validate-spec-structure?
- **Answer**: New subcommand: `cclab sdd check-alignment`. Independent command — Phase 2/3 extensions don't affect validate-spec-structure.

### Q2: General
- **Question**: For format_priority_violation, what's the threshold for 'too much prose'?
- **Answer**: Not about prose quantity. Prose is fine for human readability. The check is: does the section have the required spec-lang block for codegen? Section type determines required block: config→JSON Schema, logic→mermaid, rpc-api→JSON, state-machine→mermaid, cli→YAML, changes→YAML. If the block is missing, it's a violation. overview/requirements/scenarios don't require code blocks.

