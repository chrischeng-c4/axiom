---
change: mamba-p1
date: 2026-02-21
---

# Spec Clarifications

## Questions

### Q1: Gap Alignment
- **Question**: Do the gap analysis findings contradict any original clarifications?
- **Answer**: No contradictions. All identified gaps (bytes/bytearray, context managers, descriptors/metaclasses/super, set type, f-string format specifiers) directly correspond to the 17 P1 issues being addressed. The gaps confirm the scope is correctly defined.
- **Rationale**: GAP-SK-01 maps to #405, GAP-SK-02 to #385, GAP-SK-03 to #383/#406/#407, GAP-SK-05 to #386. Codebase gaps (string-based type system, PEP 701 f-strings) map to #382 and #388.

### Q2: Spec Updates Needed
- **Question**: Should main specs be updated before implementation?
- **Answer**: Spec updates (mamba-type-system, mamba-codegen-logic, mamba-oop-model, mamba-stdlib-core) should happen during implementation as part of each issue's spec creation, not as a prerequisite.
- **Rationale**: The SDD workflow generates per-issue specs that will address these gaps naturally.

### Q3: Risk: Async/Generator Architecture
- **Question**: Gap analysis flagged async runtime and generator lowering as high-severity. Do these block P1 work?
- **Answer**: No. Async runtime (#not in P1 scope) and generator lowering (#not in P1 scope) are pre-existing architectural gaps. The 17 P1 issues can be implemented independently of these.
- **Rationale**: None of the 17 P1 issues depend on async/generator functionality. These gaps exist but are orthogonal.

### Q4: Implementation Order Confirmation
- **Question**: Does gap analysis suggest a different implementation order than the topological DAG?
- **Answer**: No. All 17 issues are independent (no inter-dependencies). The topological order (#382-#388, #405-#409, #420-#424) remains valid. The string-based type system gap (codebase-knowledge) suggests #382 (isinstance/type narrowing) is correctly prioritized first.
- **Rationale**: Fixing isinstance/issubclass early (#382) establishes proper type identity infrastructure that other features can leverage.

