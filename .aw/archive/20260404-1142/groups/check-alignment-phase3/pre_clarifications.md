---
change: 1142
group: check-alignment-phase3
date: 2026-04-04
status: answered
---

# Pre-Clarifications

### Q1: General
- **Question**: Should artifact tools validate ONLY the section being written, or the entire spec file?
- **Answer**: Entire file. After writing, run check() on the whole spec to ensure global consistency.

### Q2: General
- **Question**: For merge workflow: should alignment warnings be written to implementation.md or just printed to stdout?
- **Answer**: Write to implementation.md — append alignment warnings to the review section.

### Q3: General
- **Question**: Should the alignment_warnings in run-change response include the full violation list or just a count + summary?
- **Answer**: Full violation list — include all violations as complete JSON in the alignment_warnings field.

