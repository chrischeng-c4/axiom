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
