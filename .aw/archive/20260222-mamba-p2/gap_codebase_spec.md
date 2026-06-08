---
change_id: mamba-p2
type: gap_codebase_spec
created_at: 2026-02-22T11:06:54.328604+00:00
updated_at: 2026-02-22T11:06:54.328604+00:00
---

# Gap Analysis: Codebase vs Spec

## Specs Without Code (High)

1. **frozenset-type**: No frozenset implementation. Needs ObjData::FrozenSet variant and frozenset_ops.rs.
2. **stdlib-re**: No re module. Needs re_mod.rs wrapping regex crate.
3. **stdlib-datetime**: No datetime module. Needs datetime_mod.rs.
4. **stdlib-collections**: No collections module (defaultdict, Counter, deque, OrderedDict).
5. **stdlib-itertools**: No itertools module. Needs iterator combinators.
6. **stdlib-functools**: No functools module (partial, lru_cache, reduce).
7. **dict-list-unpacking**: No {**d} or [*a] unpacking in parser/codegen.

## Specs Without Code (Medium)

8. **slots-support**: No __slots__ support in class.rs.
9. **stdlib-pathlib**: No pathlib module.
10. **stdlib-random**: No random module.
11. **exception-groups**: No except* syntax or ExceptionGroup class.
12. **format-protocol**: No __format__ dunder or f'{x=}' debug syntax.
13. **enum-module**: No enum module runtime.
14. **dataclasses**: No dataclasses module.
15. **contextlib**: No contextlib module.
16. **copy-module**: No copy/deepcopy module.
17. **del-finalizer**: No __del__ finalizer in GC.
18. **weakref**: No weakref module.

## Specs Without Code (Low - Utility Stdlib)

19-29. Missing stdlib modules: io/struct, hashlib, shutil, tempfile, glob, traceback, warnings, decimal/fractions, operator, inspect, base64.

## Code Without Spec

1. **class-rs-oversize**: class.rs ~1179 lines exceeds 1000 line limit. Needs split before P2 additions.