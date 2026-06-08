---
id: type-checker
title: Type Checker — Constraint Solver and Inference
crate: mamba
files:
  - crates/mamba/src/types/check.rs
  - crates/mamba/src/types/check_stmt.rs
  - crates/mamba/src/types/check_expr.rs
  - crates/mamba/src/types/builtins.rs
status: source-of-truth
last_synced_at: 2026-04-27
last_synced_commit: a47e76722
---

# Type Checker

Mamba's type checker walks `parser::ast::Module` building a
`TypeContext` of inferred / annotated types per AST node. Statements
are dispatched in `check_stmt.rs`; expressions in `check_expr.rs`;
shared utilities (subtype, unify, common-supertype) in `check.rs`;
builtin function signatures registered in `builtins.rs`.

The checker is **gradual** — `Ty::Any` makes any check succeed; this
matches Python's optional-typing model where unannotated code falls
back to dynamic.

Three load-bearing invariants:

1. **`Ty::Any` short-circuits subtype / unify** — every check that
   sees `Any` on either side returns success. Errors propagate via
   `Ty::Error` rather than panic so a single bad annotation doesn't
   cascade.
2. **Comparison chains type-check element-wise** — `1 < x < 10` is
   not `(int < int) < int`; the checker recognises `Expr::Compare`
   and verifies each adjacent pair.
3. **Generic type variables are unified post-call** — `def f[T](x: T) -> T`
   when called with `f(5)` first checks `T = int`, then sets the
   return as `int`. Out-of-order unification breaks `T` resolution.

## Type model
<!-- type: dependency lang: mermaid -->

```mermaid
---
id: type-checker-types
types:
  TypeChecker:    { kind: struct, label: "stateful walker; carries TypeContext + symbols" }
  CheckStmt:      { kind: struct, label: "check_stmt — per-Stmt rule dispatch" }
  CheckExpr:      { kind: struct, label: "check_expr — per-Expr rule dispatch" }
  CheckCore:      { kind: struct, label: "check.rs — subtype / unify / common_supertype" }
  Builtins:       { kind: struct, label: "register builtin fn signatures (print, len, range, ...)" }
  TypeContext:    { kind: struct, label: "from types/type-representations" }
  AST:            { kind: struct, label: "parser::ast (input)" }
  HirBuilder:     { kind: struct, label: "lower::ast_to_hir (consumer)" }
edges:
  - { from: TypeChecker, to: CheckStmt,   kind: owns }
  - { from: TypeChecker, to: CheckExpr,   kind: owns }
  - { from: TypeChecker, to: CheckCore,   kind: owns }
  - { from: TypeChecker, to: Builtins,    kind: owns }
  - { from: TypeChecker, to: TypeContext, kind: references }
  - { from: TypeChecker, to: AST,         kind: references, label: "input" }
  - { from: HirBuilder,  to: TypeChecker, kind: references, label: "consumes annotations" }
---
classDiagram
    class TypeChecker
    class CheckStmt
    class CheckExpr
    class CheckCore
    class Builtins
    class TypeContext
    class AST
    class HirBuilder
    TypeChecker --> CheckStmt : owns
    TypeChecker --> CheckExpr : owns
    TypeChecker --> CheckCore : owns
    TypeChecker --> Builtins : owns
    TypeChecker --> TypeContext : refs
    TypeChecker --> AST : input
    HirBuilder --> TypeChecker : consumes
```

## Check-result shape
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "type-checker-types"
$defs:
  CheckResult:
    type: object
    properties:
      ty:     { x-rust-type: TypeId, description: "result of expression / annotation" }
      errors: { type: array, items: { type: string } }
    required: [ty, errors]
  SubtypeRule:
    description: "When does `sub <: super` hold?"
    type: array
    items:
      type: object
      properties:
        rule:        { type: string }
        description: { type: string }
      required: [rule, description]
    examples:
      - - { rule: "Any-anywhere",     description: "Ty::Any <: T and T <: Ty::Any always" }
        - { rule: "Bool <: Int",      description: "True/False are valid ints" }
        - { rule: "Int <: Float",     description: "implicit numeric promotion" }
        - { rule: "Union-LR",         description: "T <: Union[U..] iff T <: any U" }
        - { rule: "Class-MRO",        description: "Sub <: Super iff Super in Sub.mro" }
        - { rule: "Generic-covariant", description: "List[T] <: List[U] iff T <: U" }
        - { rule: "Tuple-fixed",      description: "Tuple[T..] <: Tuple[U..] iff lens match and pointwise sub" }
        - { rule: "Literal-narrow",   description: "Literal[1] <: Int" }
```

## Check dispatch logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: type-checker-dispatch
entry: enter
nodes:
  enter:        { kind: start,    label: "check_module(Module)" }
  per_stmt:     { kind: process,  label: "for each Spanned<Stmt>: check_stmt" }
  is_expr_stmt: { kind: decision, label: "Stmt::Expr / Assign / Return / etc.?" }
  check_expr_:  { kind: process,  label: "check_expr — recurse on Expr" }
  is_def:       { kind: decision, label: "FnDef / AsyncFnDef / ClassDef?" }
  check_def:    { kind: process,  label: "register signature; check body in new scope" }
  is_compound:  { kind: decision, label: "If / While / For / With / Try / Match?" }
  check_compound: { kind: process, label: "check header expr; recurse body; merge scopes" }
  unify_or_err: { kind: process,  label: "unify result with annotation; produce Ty::Error on conflict" }
  done:         { kind: terminal, label: "Vec<TypeError> + populated TypeContext" }
edges:
  - { from: enter,         to: per_stmt }
  - { from: per_stmt,      to: is_expr_stmt }
  - { from: is_expr_stmt,  to: check_expr_,    label: "yes" }
  - { from: is_expr_stmt,  to: is_def,         label: "no" }
  - { from: is_def,        to: check_def,      label: "yes" }
  - { from: is_def,        to: is_compound,    label: "no" }
  - { from: is_compound,   to: check_compound, label: "yes" }
  - { from: check_expr_,   to: unify_or_err }
  - { from: check_def,     to: unify_or_err }
  - { from: check_compound, to: unify_or_err }
  - { from: unify_or_err,  to: per_stmt,       label: "next" }
  - { from: unify_or_err,  to: done,           label: "EOF" }
---
flowchart TD
    enter([check_module]) --> per_stmt[per Stmt]
    per_stmt --> is_expr_stmt{expr-stmt?}
    is_expr_stmt -->|yes| check_expr_[check_expr]
    is_expr_stmt -->|no| is_def{def/class?}
    is_def -->|yes| check_def[register sig + body]
    is_def -->|no| is_compound{compound?}
    is_compound -->|yes| check_compound[header + body]
    check_expr_ --> unify_or_err[unify with annotation]
    check_def --> unify_or_err
    check_compound --> unify_or_err
    unify_or_err --> per_stmt
    unify_or_err --> done([errors + TypeContext])
```

## Type-check / lower interaction
<!-- type: interaction lang: mermaid -->

```mermaid
---
id: type-check-flow
actors:
  - { id: AST,         kind: system, label: "parser::ast" }
  - { id: Checker,     kind: system, label: "TypeChecker" }
  - { id: TypeContext, kind: system }
  - { id: HirBuilder,  kind: system, label: "lower::ast_to_hir" }
messages:
  - { from: AST,         to: Checker,     name: "check_module(Module)" }
  - { from: Checker,     to: Checker,     name: "register builtin signatures" }
  - { from: Checker,     to: Checker,     name: "walk Stmts; check_stmt / check_expr" }
  - { from: Checker,     to: TypeContext, name: "intern_ty per encountered type" }
  - { from: Checker,     to: AST,         name: "annotate each Spanned<Expr> with TypeId" }
  - { from: AST,         to: HirBuilder,  name: "lower_module(Module, TypeChecker)" }
  - { from: HirBuilder,  to: Checker,     name: "look up TypeId per AST node" }
---
sequenceDiagram
    participant AST
    participant Checker
    participant TypeContext
    participant HirBuilder
    AST->>Checker: check_module
    Checker->>Checker: register builtins
    Checker->>Checker: walk Stmts
    Checker->>TypeContext: intern types
    Checker-->>AST: annotated
    AST->>HirBuilder: lower_module
    HirBuilder->>Checker: lookup TypeId
```

## Acceptance scenarios
<!-- type: scenarios lang: yaml -->

```yaml
scenarios:
  - id: assignment-type-match
    given: language/type_check_pass.py assigns int values to int variables
    when: Mamba checks the module
    then: no type errors are reported
  - id: assignment-type-mismatch
    given: language/type_check_fail.py assigns a str to an int variable
    when: Mamba checks the assignment
    then: a TypeError reports that str cannot be assigned to int
  - id: chained-compare
    given: language/chained_compare_check.py uses 1 < x < 10 with x typed as int
    when: comparison checking runs
    then: each adjacent pair is checked independently and succeeds
  - id: any-short-circuit
    given: a value typed Any is used through calls, indexing, and field access
    when: subtype and unification checks run
    then: Ty::Any short-circuits the checks and no errors are emitted
```

## Tests
<!-- type: test-plan lang: mermaid -->

```mermaid
---
id: type-checker-test-plan
title: Type Checker Test Plan
---
flowchart TD
    Runner["cargo test -p mamba --test type_check_tests --release -- {name} --test-threads=1"]
    Runner --> AssignMatch["test_assignment_type_match"]
    Runner --> AssignMismatch["test_assignment_type_mismatch"]
    Runner --> ChainedCompare["test_chained_compare_type_check"]
    Runner --> AnyShortCircuit["test_any_short_circuits_check"]
    Runner --> GenericCall["test_generic_fn_call_unification"]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - file: crates/mamba/src/types/check.rs
    action: modify
    impl_mode: hand-written
    description: "Subtype / unify / common_supertype core rules. Hand-written; the subtype table is the contract."
  - file: crates/mamba/src/types/check_stmt.rs
    action: modify
    impl_mode: hand-written
    description: "Per-Stmt type-check dispatch. Hand-written."
  - file: crates/mamba/src/types/check_expr.rs
    action: modify
    impl_mode: hand-written
    description: "Per-Expr type-check dispatch incl. comparison-chain element-wise + generic call unification. Hand-written."
  - file: crates/mamba/src/types/builtins.rs
    action: modify
    impl_mode: hand-written
    description: "Register builtin fn signatures (print, len, range, sorted, ...). Hand-written."
```
