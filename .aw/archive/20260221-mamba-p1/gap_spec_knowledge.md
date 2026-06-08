---
change_id: mamba-p1
type: gap_spec_knowledge
created_at: 2026-02-20T17:24:24.184396+00:00
updated_at: 2026-02-20T17:24:24.184396+00:00
---

# Gap Analysis: Spec vs Knowledge

## Overview
This analysis identifies discrepancies between the planned specifications (`spec_context.md`) and the established knowledge base (`knowledge_context.md`) for the `mamba-p1` change. This revision addresses gaps in classification, repair actions, and strengthens evidence for identified omissions.

## Identified Gaps

### GAP-SK-01: Missing Core Data Types (Bytes/Bytearray) [high]
- **Description**: Bytes and bytearray types are missing from both the specification list and the detailed knowledge summaries, despite being foundational Python types for the Mamba runtime.
- **Type**: knowledge_not_in_spec
- **Spec Status**: Identified as a high-priority gap in `spec_context` (GAP-01 #405).
- **Knowledge Status**: Not mentioned in any main spec summary. Direct inspection of `mamba-type-system.md` and `mamba-stdlib-core.md` confirms absence.
- **Action Needed**: YES
- **Repair Action**: Define `Bytes` and `Bytearray` data models and runtime operations in `mamba-type-system.md` or `mamba-stdlib-core.md`.

### GAP-SK-02: Context Manager Protocol Omission [high]
- **Description**: The context manager protocol (`__enter__`/`__exit__`) and 'with' statement lowering are identified as missing in the spec context and are notably absent from the knowledge summary for `mamba-codegen-logic`.
- **Type**: knowledge_not_in_spec
- **Spec Status**: Explicitly listed as a gap in `spec_context` (GAP-02 #385).
- **Knowledge Status**: Knowledge for codegen focuses on comprehensions and generators but omits context managers.
- **Action Needed**: YES
- **Repair Action**: Update `mamba-codegen-logic.md` to include `with` statement lowering rules and protocol requirements for `__enter__` and `__exit__`.

### GAP-SK-03: Incomplete OOP Mechanics (Descriptors/Metaclasses/Super) [medium]
- **Description**: Advanced object-oriented features—specifically the descriptor protocol, metaclasses, and the runtime implementation details for `super()`—are noted as missing or needing expansion in specs, and the corresponding knowledge base does not reflect their full implementation details.
- **Type**: boundary_misalignment
- **Spec Status**: Expansion required in `mamba-oop-model` for descriptors (GAP-03 #406), super() runtime (GAP-05 #383), and metaclasses (GAP-05 #407).
- **Knowledge Status**: Knowledge for the OOP model covers attribute access and high-level `super()` dispatch but lacks descriptor/metaclass depth and specific runtime implementation details for `super()`.
- **Action Needed**: YES
- **Repair Action**: Expand `mamba-oop-model.md` with descriptor protocol rules, metaclass instantiation logic, and internal `super()` dispatch mechanisms.

### GAP-SK-04: Orbit Bridge Integration Boundary [low]
- **Description**: Reclassified as a low-severity risk. High-level integration is already specified; further internals are appropriately placed in knowledge docs.
- **Type**: boundary_misalignment
- **Spec Status**: Dependency noted, but depth is lacking at the spec layer.
- **Knowledge Status**: Significant documentation exists in `knowledge:orbit/bridge-internals.md` that is not fully reflected in the spec's dependency scope. Direct inspection of `mamba-async-runtime.md` reveals that high-level integration is already specified.
- **Action Needed**: NO
- **Repair Action**: None. The high-level integration in `mamba-async-runtime.md` is sufficient for the spec layer.

### GAP-SK-05: Missing Set Type Specification [medium]
- **Description**: The Set type and its specific operations are not explicitly covered in iteration or stdlib specs/knowledge, despite being a core Python collection.
- **Type**: knowledge_not_in_spec
- **Spec Status**: Identified as a gap in iteration/stdlib specs (GAP-04 #386).
- **Knowledge Status**: Direct inspection of `mamba-stdlib-core.md`, `mamba-iteration-protocol.md`, and `mamba-type-system.md` confirms that `Set` is not specified.
- **Action Needed**: YES
- **Repair Action**: Add `Set` type definition and its iteration/membership operations to `mamba-stdlib-core.md`.
