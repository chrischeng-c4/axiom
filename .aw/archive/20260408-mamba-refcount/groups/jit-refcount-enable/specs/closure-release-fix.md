---
id: closure-release-fix
type: spec
title: "Closures and Free Variables"
version: 1
spec_type: algorithm
files:
  - runtime/closure.rs
main_spec_ref: crates/mamba/runtime/closure.md
merge_strategy: extend
fill_sections: [overview, changes]
filled_sections: [overview, changes]
create_complete: true
---

# Closures and Free Variables

## Overview

<!-- type: overview lang: markdown -->

Fix the asymmetric ownership bug in `mb_closure_release` that blocks enabling CPython 3.12 reference counting in Mamba's JIT codegen (#1129). The current implementation at `runtime/closure.rs:137-140` removes the closure from the thread-local `CLOSURES` HashMap but does **not** release the captured `MbValue` references held in `MbClosure.captures`, `MbClosure.func`, or `MbClosure.defaults`. When `EMIT_REFCOUNT_CALLS=true`, the JIT emits `mb_release_value` for all local variables at function return. Captured heap objects (strings, lists, instances) that were retained via `mb_closure_get_capture` (which correctly calls `retain_if_ptr`) become dangling because the closure's captures are dropped without decrementing their refcounts. This causes use-after-free (SIGBUS) in sequential tests that share thread-local runtime state.

**Root cause**: `mb_closure_release` calls `closures.borrow_mut().remove(&id)` which drops the `MbClosure` struct. Rust's `Drop` for `Vec<MbValue>` deallocates the vector memory but does NOT call `release_if_ptr` on each `MbValue` element because `MbValue` is `Copy` (a NaN-boxed `u64`). The heap objects pointed to by those `MbValue`s are leaked -- their refcount never reaches zero, preventing deallocation. Worse, under `EMIT_REFCOUNT_CALLS=true`, the JIT-emitted release for a local variable that held the same object can drive the refcount to zero and free the object while the closure's captures Vec still holds a (now-dangling) pointer, triggering use-after-free on the next test run when `cleanup_all_closures` iterates the same thread-local state.

**Fix**: Before removing from the HashMap, iterate `closure.captures` and call `rc::release_if_ptr(val)` for each captured value. Also release `closure.func` if it is a heap pointer. Also release each value in `closure.defaults`. Additionally, fix `mb_closure_set_capture` to release the old value before overwriting (prevents leak on mutable closure captures). Similarly, fix `cleanup_all_closures` to cascade-release all contained MbValues before clearing the HashMaps.

**Symmetry invariant**: Every `mb_closure_new` that stores MbValues into the closure must be balanced by `mb_closure_release` cascade-releasing those same MbValues. This mirrors CPython's `func_dealloc` which calls `Py_XDECREF` on every captured cell in `func_closure`.

Issue: #1129. Phase ordering: this fix must land BEFORE `EMIT_REFCOUNT_CALLS=true` and BEFORE GC re-enable.
## Source Files

| File | LOC | Responsibility |
|------|-----|----------------|
| `runtime/closure.rs` | 291 | Closure creation, free variable capture, cell variables |

## Requirements

### R1 - Free variable capture from enclosing function scopes

```yaml
id: R1
priority: high
```

During name resolution, variables referenced in an inner function but defined in an outer function are marked as free variables. At function definition time, these variables are captured from the enclosing scope into the closure's environment (`Vec<MbValue>` or similar). The closure carries both the code pointer and the captured environment.

### R2 - Cell variable indirection for mutable shared state

```yaml
id: R2
priority: high
```

When a captured variable is mutated by either the inner or outer scope, it is stored as a cell variable (a heap-allocated single-slot container). Both the enclosing scope and the closure reference the same cell, enabling shared mutable state. This matches Python's `nonlocal` semantics.

### R3 - Closure creation at function definition time

```yaml
id: R3
priority: high
```

When a `def` or `lambda` statement is executed and the function has free variables:
1. Resolve each free variable from the enclosing scope
2. Package them into a capture environment
3. Create `ObjData::Function` with code pointer + captured environment

The closure is created each time the `def` statement executes (closures in loops get fresh captures per iteration).

### R4 - Nested closure support

```yaml
id: R4
priority: high
```

Closures can capture variables from any enclosing scope, not just the immediate parent. A closure inside a closure re-captures from its parent's environment. The compiler chains cell variable references through multiple nesting levels.

### R5 - Lambda as anonymous closure

```yaml
id: R5
priority: high
```

`lambda args: expr` compiles to an anonymous closure with the same capture semantics as `def`. The lambda body is a single expression that is returned. Lambda supports default argument values and captures free variables identically to named functions.

## Acceptance Criteria

### Scenario: Basic closure capture

- **GIVEN** `def outer(): x = 10; def inner(): return x; return inner`
- **WHEN** `outer()()` is called
- **THEN** Returns 10

### Scenario: Mutable capture with nonlocal

- **GIVEN** `def counter(): n = 0; def inc(): nonlocal n; n += 1; return n; return inc`
- **WHEN** `c = counter(); c(); c()`
- **THEN** Returns 1, then 2

### Scenario: Closure in loop

- **GIVEN** `funcs = [lambda i=i: i for i in range(3)]`
- **WHEN** `[f() for f in funcs]`
- **THEN** Returns `[0, 1, 2]` (each closure captures its own `i`)

### Scenario: Nested closure

- **GIVEN** `def a(): x=1; def b(): def c(): return x; return c; return b`
- **WHEN** `a()()()` is called
- **THEN** Returns 1

### Scenario: Lambda capture

- **GIVEN** `def make_adder(n): return lambda x: x + n`
- **WHEN** `make_adder(5)(3)` is called
- **THEN** Returns 8


## Changes

<!-- type: changes lang: yaml -->

```yaml
files:
  - path: crates/mamba/src/runtime/closure.rs
    action: MODIFY
    targets:
      - type: function
        name: mb_closure_release
        change: |
          Before removing the closure from the CLOSURES HashMap, cascade-release
          all contained MbValues. The fix replaces the current one-liner:
            closures.borrow_mut().remove(&(id as u64));
          with:
            if let Some(closure) = closures.borrow_mut().remove(&(id as u64)) {
                unsafe {
                    for val in &closure.captures {
                        super::rc::release_if_ptr(*val);
                    }
                    super::rc::release_if_ptr(closure.func);
                    for val in &closure.defaults {
                        super::rc::release_if_ptr(*val);
                    }
                }
            }
          This ensures every MbValue stored during mb_closure_new (captures vec,
          func pointer, defaults vec) has its refcount decremented when the
          closure is released. Mirrors CPython func_dealloc calling Py_XDECREF
          on func_closure cells.
      - type: function
        name: mb_closure_set_capture
        change: |
          Release the old value at captures[idx] before overwriting with the
          new value. Currently line 117 does `c.captures[idx] = value;` without
          releasing the previous MbValue. Add:
            if idx < c.captures.len() {
                unsafe { super::rc::release_if_ptr(c.captures[idx]); }
            }
          before the assignment. This prevents leaking heap objects when
          mutable closures (nonlocal) reassign captured variables.
      - type: function
        name: mb_cell_set
        change: |
          Release the old cell value before overwriting. Currently line 289
          does `cells.borrow_mut().insert(id as u64, value);` which drops the
          old MbValue without releasing the pointed-to heap object. Change to:
            if let Some(old) = cells.borrow_mut().insert(id as u64, value) {
                unsafe { super::rc::release_if_ptr(old); }
            }
          This prevents leaking when nonlocal variables are reassigned via
          mb_cell_set.
      - type: function
        name: mb_global_set
        change: |
          Release the old global value before overwriting. Currently line 318
          does `ns.borrow_mut().insert(var_name, value);` which drops the old
          MbValue without releasing. Change to:
            if let Some(old) = ns.borrow_mut().insert(var_name, value) {
                unsafe { super::rc::release_if_ptr(old); }
            }
      - type: function
        name: mb_global_set_id
        change: |
          Release the old global value before overwriting. Same pattern as
          mb_global_set — capture the return of HashMap::insert and release
          the old value if present.
      - type: function
        name: cleanup_all_closures
        change: |
          Before clearing each HashMap, iterate and release all contained
          MbValues. Currently line 396-402 just calls .clear() on each map.
          The fix must:
          1. CLOSURES: for each MbClosure, release captures, func, defaults
          2. CELLS: for each cell value, release_if_ptr
          3. GLOBAL_NAMESPACE: for each global value, release_if_ptr
          4. GLOBAL_ID_NAMESPACE: for each global value, release_if_ptr
          Then proceed with .clear() and counter resets as before.
          This prevents leaking when test cleanup runs between test executions.
    do_not_touch:
      - mb_closure_new
      - mb_closure_get_capture
      - mb_closure_get_func
      - mb_closure_set_defaults
      - closure_defaults
      - mb_apply_decorator
      - mb_apply_decorators
      - mb_property_new
      - mb_func_set_name
      - mb_func_get_name
      - mb_cell_new
      - mb_cell_get
      - mb_global_get
      - mb_global_get_id
      - extract_str
      - extract_list
      - save_and_clear_global_id_namespace
      - restore_global_id_namespace
      - snapshot_global_id_namespace
```

# Reviews
