---
change: fix-gen-apply-file-path
date: 2026-04-10
status: answered
---

# Pre-Clarifications

### Q1: General
- **Question**: Which files are affected?
- **Answer**: crates/sdd/src/generate/apply.rs (primary fix: extract_change_entries) and crates/sdd/src/generate/diff.rs (if it duplicates the same path lookup logic).

### Q2: General
- **Question**: What is the canonical field name going forward?
- **Answer**: path: is canonical. file: is accepted as backward-compatible alias. A comment in the code will document this.

### Q3: General
- **Question**: What warning should be emitted for 0 entries?
- **Answer**: When Changes section has valid YAML but 0 entries extracted, emit a warning via log::warn! or eprintln! indicating that no change entries were found. This prevents silent no-op from masking bugs.

### Q4: General
- **Question**: Scope of regression tests?
- **Answer**: Unit tests in crates/sdd/src/generate/ covering: (1) YAML block with path: key works, (2) YAML block with file: key works, (3) both path: and file: in same block (path: takes precedence), (4) block with neither key produces 0 entries.

### Q5: General
- **Question**: Should the spec be updated?
- **Answer**: Add a comment in code noting that both path: and file: are accepted with path: as canonical. No separate spec file needed for this small bug fix.

