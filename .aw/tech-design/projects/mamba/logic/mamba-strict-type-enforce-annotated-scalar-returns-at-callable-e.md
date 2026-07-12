---
id: mamba-strict-type-enforce-annotated-scalar-returns-at-callable-e
summary: Enforce retained scalar return annotations at synchronous user-function egress without changing the raw or boxed ABI.
fill_sections: [logic, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
flowchart TD
    A["Synchronous user function has a return annotation"] --> B["Lower explicit, bare, or implicit return"]
    B --> C["Resolve retained scalar contract and source spelling"]
    C --> D{"Definite scalar contract?"}
    D -- "unannotated, Any, or unsupported" --> E["Use existing return lowering and ABI"]
    D -- "int, bool, float, str, bytes, or None" --> F["Validate and adapt before ABI crossing"]
    F -- "mismatch" --> G["Set catchable TypeError and return existing error sentinel"]
    F -- "accepted" --> H["Preserve or adapt the existing raw, boxed, or F64 return ABI"]
    H --> E
    E --> I["Direct and Any-erased dynamic callers observe one result contract"]
```
