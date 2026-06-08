---
change: sdd-tdd-gate
group: tdd-workflow-gate
date: 2026-04-08
status: answered
---

# Pre-Clarifications

### Q1: General
- **Question**: What format should requirement markers in test files use — comments like // REQ: REQ-001 or test function naming convention like test_req_001_*?
- **Answer**: Use comments: `// REQ: REQ-001` (Rust) or `# REQ: REQ-001` (Python/shell). This is more flexible than naming conventions — a single test can cover multiple requirements by listing multiple REQ comments. The scanner should regex-match `REQ:\s*(REQ-\w+)` patterns.

### Q2: General
- **Question**: Should the gate be a hard block (cannot advance at all) or a soft warning that can be overridden with a flag?
- **Answer**: Hard block by default. Add a `--skip-tests` flag to `score run-change` for escape hatch in emergencies, but it should log a warning and mark the change as 'tests-skipped' in STATE.yaml.

### Q3: General
- **Question**: For Gate 2 glob matching, should it use the same glob library as .gitignore or a simpler glob crate?
- **Answer**: Use the `globset` crate (same as Gate 1 / test-config group). It supports gitignore-style patterns and is already a common dependency in the Rust ecosystem. Consistent with the changes patterns in TestScope.

