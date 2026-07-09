---
id: jet-dts-as-type-explicit-annotation
summary: "jet --lib --dts isolatedDeclarations: `expr as Type` assertions (const initializer or function return) count as an explicit type annotation, closing the WI #937 false-positive family."
capability_refs:
  - id: "library-build-publishing"
    role: primary
    gap: "type-declaration-emission"
    claim: "type-declaration-emission"
    coverage: partial
    rationale: "Pins WI #937 regression coverage for the `expr as Type` assertion variant of the isolatedDeclarations false-positive family inside the Type Declaration Emission work root (jet --lib --dts .d.ts emission)."
fill_sections: [logic, unit-test]
---

# jet --lib --dts isolatedDeclarations: `expr as Type` explicit-annotation inference

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: jet-dts-as-type-explicit-annotation
entry: value_or_return_expr
nodes:
  value_or_return_expr:   { kind: start,    label: "const/let/var declarator value,\nor function/method return expr" }
  is_as_or_satisfies:     { kind: decision, label: "expr kind is as_expression\nor satisfies_expression?" }
  annotated_expr_type:    { kind: process,  label: "annotated_expression_type(node, source):\nunwrap parenthesized_expression,\nread child_by_field_name(type)\n(fallback last_named_child)" }
  emit_asserted_type:     { kind: terminal, label: "emit asserted type text verbatim;\ninitializer/return body dropped" }
  existing_inference:     { kind: process,  label: "existing chain unchanged:\ninfer_variable_declarator_type /\ninfer_return_statement_type ->\ninfer_expression_type /\ninfer_object_literal_type /\ninfer_arrow_function_type" }
  is_locally_inferable:   { kind: decision, label: "locally inferable\nwithout the cast?" }
  emit_inferred_type:     { kind: terminal, label: "emit inferred type" }
  isolated_decl_error:    { kind: terminal, label: "isolatedDeclarations error\n(DtsDiagnostic, fail-loud)" }
edges:
  - { from: value_or_return_expr, to: is_as_or_satisfies }
  - { from: is_as_or_satisfies,   to: annotated_expr_type,  label: "yes" }
  - { from: is_as_or_satisfies,   to: existing_inference,   label: "no"  }
  - { from: annotated_expr_type,  to: emit_asserted_type }
  - { from: existing_inference,   to: is_locally_inferable }
  - { from: is_locally_inferable, to: emit_inferred_type,   label: "yes" }
  - { from: is_locally_inferable, to: isolated_decl_error,  label: "no"  }
---
flowchart TD
    value_or_return_expr(["const/let/var declarator value,\nor function/method return expr"]) --> is_as_or_satisfies{"expr kind is as_expression\nor satisfies_expression?"}
    is_as_or_satisfies -->|yes| annotated_expr_type["annotated_expression_type(node, source):\nunwrap parenthesized_expression,\nread child_by_field_name(type)\n(fallback last_named_child)"]
    is_as_or_satisfies -->|no| existing_inference["existing chain unchanged:\ninfer_variable_declarator_type /\ninfer_return_statement_type ->\ninfer_expression_type /\ninfer_object_literal_type /\ninfer_arrow_function_type"]
    annotated_expr_type --> emit_asserted_type(["emit asserted type text verbatim;\ninitializer/return body dropped"])
    existing_inference --> is_locally_inferable{"locally inferable\nwithout the cast?"}
    is_locally_inferable -->|yes| emit_inferred_type(["emit inferred type"])
    is_locally_inferable -->|no| isolated_decl_error(["isolatedDeclarations error\n(DtsDiagnostic, fail-loud)"])
```

Scope for WI #937 (`projects/jet/src/bundler/dts.rs`): `annotated_expression_type` already recognizes `as_expression` / `satisfies_expression` nodes and returns the asserted type's source text regardless of the wrapped expression's shape (object literal, identifier, call expression, generic function type). It is wired into both call sites the WI's acceptance criteria name — `infer_variable_declarator_type` (const/let/var initializer position, tried before arrow-function / object-literal inference) and `infer_return_statement_type` (function/method return position, tried per `return_statement` before the text-based `infer_expression_type` fallback, so a local-variable-then-`return x as Type` shape resolves through the cast, not the variable). Verified end-to-end: `jet build --lib --format esm --dts` against the WI's exact minimal repro (`asCastConst`, `asCastReturn`) and its three cited real-code shapes (`useRouterConfig`'s local-var-then-cast return, `SpAlert`'s identifier cast, `SpTable`'s generic function-type cast) all emit correctly-typed `.d.ts` output with 0 isolatedDeclarations errors on the current source tree.

This closes the `expr as Type` false-positive family without intersecting the sibling variants tracked separately in #1238 (Object.assign+computed-key body), #1263 (nested object literals), and #1264 (arrow function without explicit return type returning a typed object literal) — none of those source shapes pass through an `as_expression`/`satisfies_expression` node, so this logic path leaves their still-open false positives unchanged.

The `unit-test` section pins deterministic regression coverage for both positions plus a negative case: prior tests exercised an `as`-cast on an object-literal-with-async-method const shape and a local-object-then-cast function return, but none pinned the WI's exact plain-object-literal `{ a: 1 } as Foo` const-initializer shape, and no test pairs the two `as`-cast regressions with an explicit negative case proving a genuinely untyped/uninferrable expression (no `as`/`satisfies` wrapper, no explicit annotation) still fails loud once this inference path exists.

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: jet-dts-as-type-explicit-annotation-verification
requirements:
  const_identifier_as_expression_real_code_shape:
    id: R3
    text: "An exported const initialized to a bare identifier cast via `as Type` (WI #937's cited real-code `SpAlert` shape: `export const SpAlert = Alert as AlertInterface;`) emits the asserted interface type instead of an isolatedDeclarations error, confirming the fix is not limited to object-literal initializers."
    kind: regression
    risk: low
    verify: cargo test -p jet --lib bundler::dts::tests::infers_exported_const_identifier_as_expression_signature
  const_initializer_as_expression:
    id: R1
    text: "An exported const whose initializer is a plain object-literal `expr as Type` assertion (WI #937 minimal repro: `export const asCastConst = { a: 1 } as Foo;`) emits `export declare const asCastConst: Foo;` with the initializer dropped, instead of an isolatedDeclarations error."
    kind: functional
    risk: medium
    verify: cargo test -p jet --lib bundler::dts::tests::infers_exported_const_plain_object_literal_as_expression_signature
  function_return_local_variable_as_expression:
    id: R2
    text: "An exported function whose body assigns to a local variable and returns it cast via `x as Type` (WI #937 minimal repro: `export function asCastReturn() { const x: unknown = { a: 1 }; return x as Foo; }`) emits `export declare function asCastReturn(): Foo;` instead of an isolatedDeclarations error."
    kind: functional
    risk: medium
    verify: cargo test -p jet --lib bundler::dts::tests::infers_exported_function_return_via_local_variable_as_expression
  function_return_local_variable_without_as_expression_still_errors:
    id: R4
    text: "Negative control: the same local-variable-return shape as R2 but with the `as Foo` cast removed (`export function notCast() { const x: unknown = { a: 1 }; return x; }`) must still raise an isolatedDeclarations error, proving the `annotated_expression_type` fix is scoped to explicit `as`/`satisfies` casts and does not broaden return-expression inference to uncast local variables."
    kind: regression
    risk: high
    verify: cargo test -p jet --lib bundler::dts::tests::uninferrable_exported_function_return_of_local_variable_without_as_expression_errors
---
flowchart TD
    r1[R1 const initializer as expression] --> cargo_test_p_jet_lib_bundler_dts_tests_infers_exported_const_plain_object_literal_as_expression_signature[cargo test -p jet --lib bundler::dts::tests::infers_exported_const_plain_object_literal_as_expression_signature]
    r2[R2 function return local variable as expression] --> cargo_test_p_jet_lib_bundler_dts_tests_infers_exported_function_return_via_local_variable_as_expression[cargo test -p jet --lib bundler::dts::tests::infers_exported_function_return_via_local_variable_as_expression]
    r3[R3 const identifier as expression real code shape] --> cargo_test_p_jet_lib_bundler_dts_tests_infers_exported_const_identifier_as_expression_signature[cargo test -p jet --lib bundler::dts::tests::infers_exported_const_identifier_as_expression_signature]
    r4[R4 function return local variable without as expression still errors] --> cargo_test_p_jet_lib_bundler_dts_tests_uninferrable_exported_function_return_of_local_variable_without_as_expression_errors[cargo test -p jet --lib bundler::dts::tests::uninferrable_exported_function_return_of_local_variable_without_as_expression_errors]
```
