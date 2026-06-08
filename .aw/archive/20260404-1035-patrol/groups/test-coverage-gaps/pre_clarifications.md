---
change: 1035-patrol
group: test-coverage-gaps
date: 2026-04-04
status: answered
---

# Pre-Clarifications

### Q1: General
- **Question**: Focus area for test coverage?
- **Answer**: Core modules first. Prioritize ffi/c_types.rs (0%), driver/mod.rs (33%), codegen/cranelift/mod.rs (45%) over stdlib stubs.

### Q2: General
- **Question**: Scope for this single SDD change?
- **Answer**: Top 5 worst files only: ffi/c_types.rs (0%), queue_mod.rs (4%), statistics_mod.rs (5%), shlex_mod.rs (7%), calendar_mod.rs (8%). Bring each to at least 50% line coverage.

