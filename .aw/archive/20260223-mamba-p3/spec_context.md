---
change_id: mamba-p3
type: spec_context
created_at: 2026-02-23T01:07:52.438576+00:00
updated_at: 2026-02-23T01:07:52.438576+00:00
iteration: 1
complexity: high
stage: spec
scanned_groups:
  - cclab-mamba
---

# Spec Context

## Relevant Specs

- **mamba-stdlib-core**
  - relevance: high
- **mamba-oop-model**
  - relevance: medium
- **mamba-jit-backend**
  - relevance: medium
- **mamba-string-runtime**
  - relevance: medium
- **mamba-gc-runtime**
  - relevance: low
- **mamba-import-system**
  - relevance: high

## Gaps

- No main spec for bytes/binary data (#405 P1). Dependency for socket, pickle, compression, array.
- No main spec for metaclasses/ABC (#407 P1). Dependency for unittest.
- No spec for complex numbers ObjData variant impacts ~7 match exhaustiveness sites.
- No spec for external crate dependencies (rusqlite, flate2, zip, tar).
