---
id: closure
type: spec
title: "Closures and Free Variables"
version: 1
spec_type: algorithm
files:
  - runtime/closure.rs
---

# Closures and Free Variables

## Overview

Implements closure support for Mamba. When a function references variables from an enclosing scope, the compiler detects these as free variables and captures them into the function's closure environment at definition time. Mutable shared state uses cell variable indirection so that inner and outer scopes share the same storage. Lambdas are compiled as anonymous closures.

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
