---
change_id: mamba-p1b
type: gap_spec_knowledge
created_at: 2026-02-21T15:08:07.184985+00:00
updated_at: 2026-02-21T15:08:07.184985+00:00
---

---
change_id: mamba-p1b
type: gap_spec_knowledge
created_at: 2026-02-21T15:15:00.000000+00:00
updated_at: 2026-02-21T15:15:00.000000+00:00
---

# Gap Analysis: Spec vs Knowledge

## Change ID: mamba-p1b

| Gap ID | Type | Severity | Source | Description |
| :--- | :--- | :--- | :--- | :--- |
| **oop-super-contradiction** | contradiction | High | `spec:cclab-mamba/mamba-oop-model.md` vs `issue_383` | The `mamba-oop-model` specification (Knowledge) defines `super()` support as a requirement (R2) and provides an acceptance scenario. However, `issue_383` (Spec) lists it as a new feature to be implemented because it is "not wired" in `class.rs`. This indicates a contradiction where the established spec claims it's part of the model, but the current change request treats it as a missing feature. |
| **attribute-lookup-descriptors-omission** | contradiction | High | `spec:cclab-mamba/mamba-oop-model.md` vs `issue_406` | The `mamba-oop-model` (Knowledge) provides a simplified flowchart for attribute lookup that does not account for the Descriptor Protocol (`__get__`, `__set__`, `__delete__`) or `__getattribute__`. However, `issue_406` (Spec) introduces the descriptor protocol. This creates a logical contradiction/omission in the established knowledge about how attribute access works in Mamba. |
| **os-module-scope-overlap** | boundary_misalignment | Medium | `spec:cclab-mamba/mamba-stdlib-core.md` vs `issue_424` | The `mamba-stdlib-core` specification (Knowledge) includes R2 for "file system operations" in the `os` module. However, `issue_424` (Spec) lists these same operations (join, split, exists, etc.) as missing features to be implemented. This is a boundary misalignment where the knowledge claims a capability that the new spec treats as missing. |
| **module-system-complexity-gap** | missing_pattern | Medium | `spec:cclab-mamba/mamba-import-system.md` vs `issue_421` | The `mamba-import-system` specification (Knowledge) defines core module resolution and caching. However, `issue_421` (Spec) adds complex requirements like `__init__.py` support, `sys.path` handling, and relative imports. There is a risk of conflict between the existing simplified resolution logic and the new package system requirements. |
| **metaclass-abc-omission** | missing_pattern | Medium | `spec:cclab-mamba/mamba-oop-model.md` vs `issue_407` | `mamba-oop-model` (Knowledge) defines the inheritance and MRO model but completely omits Metaclasses and Abstract Base Classes (ABCs). `issue_407` (Spec) introduces these features, which represent a significant extension to the OOP architecture not captured in the core knowledge. |
| **context-manager-protocol-pattern** | missing_pattern | Medium | `issue_385` vs missing knowledge | `issue_385` (Spec) introduces the Context Manager Protocol (`with` statement). While `mamba-iteration-protocol` (Knowledge) exists, there is no general "Runtime Protocols" knowledge document that defines the pattern for magic-method based protocols (e.g., MIR lowering, runtime trait mapping). |
| **type-narrowing-knowledge-gap** | missing_pattern | Medium | `spec:cclab-mamba/mamba-type-system.md` vs `issue_382` | `issue_382` (Spec) introduces type narrowing for `isinstance`. While `mamba-type-system` (Knowledge) covers type checking, it does not mention flow-sensitive narrowing or how assertions influence the type environment. |
