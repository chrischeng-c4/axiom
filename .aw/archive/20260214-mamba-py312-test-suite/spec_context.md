---
change_id: mamba-py312-test-suite
type: spec_context
created_at: 2026-02-13T10:31:35.883319+00:00
updated_at: 2026-02-13T10:31:35.883319+00:00
iteration: 1
complexity: high
stage: spec
scanned_groups:
  - cclab-mamba
  - cclab-probe
  - cclab-nucleus
---

# Spec Context

## Relevant Specs

- **mamba-jit-backend** (group: cclab-mamba)
  - relevance: high
  - key sections: Overview, Requirements (R1, R2), JIT Compilation Flow diagram
- **state-machine** (group: cclab-probe)
  - relevance: medium
  - key sections: Test Execution State Machine diagram, Single Test Lifecycle diagram
- **architecture** (group: cclab-nucleus)
  - relevance: medium
  - key sections: PyO3 Call Flow diagram

## Dependencies

- cclab-mamba/mamba-jit-backend depends on cclab-probe/state-machine for test runner context

## Gaps

- No spec for Python 3.12 syntax support in Mamba compiler
- No spec for CPython test suite integration strategy in Mamba fixtures
- No spec for datatest-stable harness configuration for multi-version syntax testing
