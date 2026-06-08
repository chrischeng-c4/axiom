---
change_id: mamba-p1
type: spec_context
created_at: 2026-02-20T16:15:35.786986+00:00
updated_at: 2026-02-20T16:15:35.786986+00:00
iteration: 2
complexity: high
stage: spec
scanned_groups:
  - cclab-mamba
  - cclab-orbit
---

# Spec Context

## Relevant Specs

- **mamba-oop-model**
  - relevance: high
- **mamba-codegen-logic**
  - relevance: high
- **mamba-import-system**
  - relevance: high
- **mamba-stdlib-core**
  - relevance: high
- **mamba-string-runtime**
  - relevance: high
- **mamba-iteration-protocol**
  - relevance: high
- **mamba-type-system**
  - relevance: medium
- **mamba-async-runtime**
  - relevance: medium
- **mamba-gc-runtime**
  - relevance: medium
- **mamba-jit-backend**
  - relevance: medium
- **mamba-llvm-backend**
  - relevance: medium

## Dependencies

- mamba-codegen-logic depends on mamba-iteration-protocol for comprehension and generator lowering.
- mamba-stdlib-core depends on mamba-import-system to expose built-in modules to Mamba code.
- mamba-oop-model is the foundation for super(), isinstance, decorators, descriptors, and metaclasses.
- mamba-async-runtime depends on cclab-orbit/architecture for the underlying event loop integration.
- mamba-jit-backend and mamba-llvm-backend depend on the MIR output of mamba-codegen-logic.

## Gaps

- GAP-01: Bytes and bytearray types (#405) are not defined in any existing specification. [high]
- GAP-02: The context manager protocol (__enter__/__exit__ and with-statement codegen, #385) is missing from codegen specs. [high]
- GAP-03: Descriptor protocol details (__get__/__set__/__delete__, #406) are not explicitly documented in the OOP model. [medium]
- GAP-04: Set type and its operations (#386) are not explicitly covered in iteration or stdlib specs. [medium]
- GAP-05: Runtime implementation details for super() (#383) and metaclasses (#407) need expansion in the OOP spec. [medium]
