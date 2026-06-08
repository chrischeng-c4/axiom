---
change_id: mamba-p1b
type: gap_codebase_knowledge
created_at: 2026-02-21T15:05:30.450393+00:00
updated_at: 2026-02-21T15:05:30.450393+00:00
---

# Gap Analysis: Codebase vs Knowledge

## Change ID: mamba-p1b

### 1. Set Implementation Performance Mismatch (O(N) vs O(1))
- **Severity**: high
- **Description**: Requirement R1 specifies `ObjData::Set(HashSet<u64>)` and O(1) performance using raw u64 bit patterns for hashing/equality. However, `rc.rs` defines `ObjData::Set(Vec<MbValue>)` and `set_ops.rs` implements all operations (add, contains, remove, union, etc.) using O(N) linear scans via `items.iter().any(...)`.
- **Reasons**: 
  - Convention violation: `HashSet` is imported and used for `MbClass` methods and fields, but the `Set` runtime type uses a `Vec` and linear search, violating the performance guarantees of the set data structure.

### 2. Missing `mb_set_pop` Implementation and Symbol
- **Severity**: medium
- **Description**: The specification lists 14 set functions including `pop`. The codebase (`symbols.rs` and `set_ops.rs`) includes 14 set-related symbols but `pop` is missing, while `discard`, `clear`, and `copy` are present.
- **Reasons**: 
  - Missing feature: `pop` is an essential set operation specified in the requirements but not implemented or registered in the runtime symbols.
