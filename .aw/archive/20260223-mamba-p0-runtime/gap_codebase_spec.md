---
change_id: mamba-p0-runtime
type: gap_codebase_spec
created_at: 2026-02-15T17:21:09.304692+00:00
updated_at: 2026-02-15T17:21:09.304692+00:00
---

# Gap Analysis: Codebase vs Specs

## Code Without Matching Spec (implementation exists, no spec coverage)

### HIGH severity

1. **builtins.rs: mb_print, mb_len, mb_abs, mb_range** — These 4 builtins exist in code but mamba-stdlib-core only covers sys/os/math/json modules, not the global built-in function namespace. No spec defines which Python builtins Mamba must support or their exact signatures.

2. **string_ops.rs: mb_string_concat, mb_string_repeat, mb_string_getitem, mb_string_slice** — Basic string operations exist in code but mamba-string-runtime R3 only mentions "String Operations/Methods" at a high level without specifying which methods or their dispatch mechanism.

3. **rc.rs: ObjData enum (11 variants)** — The core tagged-union value representation exists but no spec defines the NaN-boxing scheme, variant layout, or rules for adding new variants (File, Set).

4. **exception.rs: MbException struct** — A basic exception type exists with type_name/message/cause/traceback fields, but no spec defines the exception class hierarchy (ValueError, TypeError, etc.) as Python classes inheriting from BaseException.

### MEDIUM severity

5. **class.rs: compute_c3_mro** — C3 MRO implementation exists; mamba-oop-model R1 covers this but doesn't specify integration with built-in type method resolution (str/list/dict methods are not class-based yet).

6. **iter.rs: RangeIterator, ListIterator, DictKeyIterator, StringCharIterator** — Iterator implementations exist; mamba-iteration-protocol R3 mentions built-in iterators but doesn't specify enumerate/zip/reversed iterator types.

7. **symbols.rs: SYMBOL_TABLE** — Symbol registration exists but no spec defines the registration protocol or naming conventions for runtime functions.

## Specs Without Matching Implementation (spec exists, code missing)

### HIGH severity

1. **mamba-oop-model R3: Magic Method Dispatch** — Spec mentions dunder methods (__add__, __str__, __eq__, etc.) but code has no dispatch mechanism. mb_getattr does instance+MRO lookup but doesn't intercept operator syntax to route to __add__/__mul__/etc.

2. **mamba-stdlib-core R1-R4: sys/os/math/json modules** — Spec defines 4 stdlib modules. Code has `runtime/stdlib/` directory but modules are stubs — no working implementations.

3. **mamba-string-runtime R3: String Methods** — Spec mentions string methods but code only has concat/repeat/index/slice/format/len. Missing: split, join, strip, replace, find, startswith, endswith, upper, lower, count, isdigit, isalpha.

4. **mamba-iteration-protocol R3: Built-in Iterators** — Spec mentions built-in iterators but code lacks enumerate, zip, reversed, filter, map iterator types.

### MEDIUM severity

5. **mamba-gc-runtime R1: Track Container Objects** — Spec defines GC tracking for containers. Code has gc.rs stub but not integrated with list/dict creation paths.

6. **mamba-codegen-logic R1: Comprehension Lowering** — Spec mentions comprehension codegen but list/dict methods needed by comprehensions (list.append for list comp) are not implemented.

## Summary

| Category | HIGH | MEDIUM | LOW |
|----------|------|--------|-----|
| Code without spec | 4 | 3 | 0 |
| Spec without code | 4 | 2 | 0 |
| **Total** | **8** | **5** | **0** |

Critical path: The 8 HIGH-severity gaps all relate directly to the 7 P0 issues. This change must address all of them either by implementing code to match specs or by creating new specs where none exist.