---
id: jet-dts-nested-object-literal-member-inference
summary: "jet --lib --dts isolatedDeclarations: an object literal whose member value is itself a plain nested object literal (nesting depth >= 2, all leaves plain literals) is inferred instead of raising a false-positive isolatedDeclarations error, closing WI #1263."
capability_refs:
  - id: "library-build-publishing"
    role: primary
    gap: "type-declaration-emission"
    claim: "type-declaration-emission"
    coverage: partial
    rationale: "Pins WI #1263 regression coverage for the nested-plain-object-literal variant of the isolatedDeclarations false-positive family inside the Type Declaration Emission work root (jet --lib --dts .d.ts emission)."
fill_sections: [logic, unit-test, changes]
---

# jet --lib --dts isolatedDeclarations: nested plain-literal object literal member inference

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: jet-dts-nested-object-literal-member-inference
entry: member_value
nodes:
  member_value:            { kind: start,    label: "object literal member value text\n(split_top_level substring, no Node context)" }
  is_arrow_typed_value:    { kind: decision, label: "infer_arrow_function_type_from_text:\nvalue matches (params) => ret\nwith an explicit arrow return type?" }
  emit_arrow_member_type:  { kind: terminal, label: "emit member: (params) => ret;\n(existing path, unchanged)" }
  is_nested_object_literal: { kind: decision, label: "NEW: value.trim() starts_with '{'\nand ends_with '}'\n(bare nested object literal,\nnot Object.assign/call/arrow text)?" }
  infer_nested_members:    { kind: process,  label: "NEW infer_object_literal_type_from_text(value):\nrecurse using the SAME member-splitting\nand member-typing routine, factored out of\ninfer_object_literal_type's Node entry point\nso it also runs on plain text (no Node needed)" }
  nested_members_all_typed: { kind: decision, label: "every nested member resolved\nto a concrete type (recursively)?" }
  emit_nested_member_type: { kind: terminal, label: "emit member: { nested: Type; ... };\n(reuses the exact `{\n    ...\n}` formatting\nthe top-level object-literal path already emits)" }
  fallback_expression_type: { kind: process,  label: "existing fallback: infer_expression_type(value, {}):\nstring/number/boolean/null/undefined/\nidentifier/binary-expression literals only\n(never recurses into '{' text today)" }
  member_typed:            { kind: terminal, label: "member type resolved,\nappended to the enclosing object-literal signature" }
  member_untyped:          { kind: terminal, label: "member unresolved -> `?` short-circuits\ninfer_object_literal_type to None\n-> caller falls through to isolatedDeclarations error\n(today's bug: this fires for EVERY nested-object member,\nrejecting the whole outer const, matching the\nissue's exact observed nestedLiteral rejection)" }
edges:
  - { from: member_value,             to: is_arrow_typed_value }
  - { from: is_arrow_typed_value,     to: emit_arrow_member_type,   label: "yes" }
  - { from: is_arrow_typed_value,     to: is_nested_object_literal, label: "no" }
  - { from: is_nested_object_literal, to: infer_nested_members,     label: "yes" }
  - { from: is_nested_object_literal, to: fallback_expression_type, label: "no (Object.assign(...), other\ncall expressions, template\nliterals, etc. -- unchanged)" }
  - { from: infer_nested_members,     to: nested_members_all_typed }
  - { from: nested_members_all_typed, to: emit_nested_member_type,  label: "yes" }
  - { from: nested_members_all_typed, to: member_untyped,           label: "no" }
  - { from: fallback_expression_type, to: member_typed,             label: "typed" }
  - { from: fallback_expression_type, to: member_untyped,           label: "untyped" }
  - { from: emit_arrow_member_type,   to: member_typed }
  - { from: emit_nested_member_type,  to: member_typed }
---
flowchart TD
    member_value(["object literal member value text\n(split_top_level substring, no Node context)"]) --> is_arrow_typed_value{"infer_arrow_function_type_from_text:\nvalue matches (params) => ret\nwith an explicit arrow return type?"}
    is_arrow_typed_value -->|yes| emit_arrow_member_type(["emit member: (params) => ret;\n(existing path, unchanged)"])
    is_arrow_typed_value -->|no| is_nested_object_literal{"NEW: value.trim() starts_with '{'\nand ends_with '}'\n(bare nested object literal,\nnot Object.assign/call/arrow text)?"}
    is_nested_object_literal -->|yes| infer_nested_members["NEW infer_object_literal_type_from_text(value):\nrecurse using the SAME member-splitting\nand member-typing routine"]
    is_nested_object_literal -->|no, Object.assign/call/template etc.| fallback_expression_type["existing fallback: infer_expression_type(value, {}):\nstring/number/boolean/null/undefined/\nidentifier/binary-expression literals only"]
    infer_nested_members --> nested_members_all_typed{"every nested member resolved\nto a concrete type (recursively)?"}
    nested_members_all_typed -->|yes| emit_nested_member_type(["emit member: { nested: Type; ... };\n(reuses existing object-literal formatting)"])
    nested_members_all_typed -->|no| member_untyped(["member unresolved -> `?` short-circuits\ninfer_object_literal_type to None\n-> caller falls through to isolatedDeclarations error"])
    fallback_expression_type -->|typed| member_typed(["member type resolved,\nappended to enclosing object-literal signature"])
    fallback_expression_type -->|untyped| member_untyped
    emit_arrow_member_type --> member_typed
    emit_nested_member_type --> member_typed
```

Scope for WI #1263 (`projects/jet/src/bundler/dts.rs`): empirically re-verified against the issue's exact minimal repro on the current `app/jet` source tree (post-#1264, commit `75cb5e5ca`, 38 passing `bundler::dts` unit tests) -- `cargo build -p jet --bin jet` then `jet build --lib --format esm --dts` against a standalone `export const flatLiteral = { ltr: 'ltr', rtl: 'rtl' }; export const nestedLiteral = { ltr: 'ltr', heading: { h1: 'editor-heading--h1' } };` still fails with `isolatedDeclarations error — exported \`const nestedLiteral\` lacks an explicit type annotation`, confirming this is a real, still-open analyzer gap (not an already-implemented case pinned only by tests, contrast WI #937). `flatLiteral` (depth 1) passes today because #796 already wired flat plain-literal member typing through `infer_object_literal_type`; only nesting depth >= 2 reproduces.

Root cause: `infer_object_literal_type` (the routine used for both a `const`'s own object-literal initializer and, since #1264, an arrow function's single-statement `return { ... }` body) resolves each member's value type via `infer_arrow_function_type_from_text(value).or_else(|| infer_expression_type(value, &empty_param_types))?` (dts.rs, member loop). `infer_expression_type` operates purely on trimmed text and only recognizes string/number/boolean/`null`/`undefined`/bare-identifier/binary-expression literals -- it never inspects whether `value` is itself a bare `{ ... }` object-literal substring, so it always returns `None` for a nested-object-valued member. Neither of the two member-typing calls ever recurses, so a member whose value is `{ h1: 'editor-heading--h1' }` fails to type, the `?` on that failed `Option` short-circuits the whole `for raw_property in split_top_level(...)` loop, and `infer_object_literal_type` returns `None` for the ENTIRE outer object literal -- not just the nested member. This exactly matches the empirical observation: `nestedLiteral` is rejected wholesale (a single isolatedDeclarations error naming the whole `const`), not silently missing one field. This is a distinct mechanism from #1262 (still open): #1262's silent truncation comes from `Object.assign({}, ...arr.map(cb))`-shaped values already text-matching a *different* branch and a bracket-depth/`split_top_level` interaction with call-expression argument lists, whereas #1263's bare `{ ... }`-valued members never text-match any existing branch at all and cause total rejection, not truncation. This TD stays scoped to plain nested object-literal member values only and does not touch the `Object.assign`/computed-key code paths #1238 and #1262 exercise.

Fix: because `infer_object_literal_type`'s member-splitting and member-typing logic already operates purely on the `text` derived from `node_text(node, source)` (the `node.kind() != "object"` check and `node_text` call are the only two places the function actually needs a tree-sitter `Node` -- everything after that is plain `&str` manipulation via `split_top_level`/`split_once_top_level`), factor the body of `infer_object_literal_type` (from the `strip_prefix('{')`/`strip_suffix('}')` step through the final `members`-join) out into a new `infer_object_literal_type_from_text(text: &str) -> Option<String>` that takes only the already-stripped `{ ... }` text and needs no `Node` at all. `infer_object_literal_type(node, source)` becomes a thin Node-to-text adapter that calls it. In the member-value-typing chain, add a new branch tried after the existing arrow-type-from-text check and before the `infer_expression_type` fallback: if `value.trim()` starts with `'{'` and ends with `'}'`, call `infer_object_literal_type_from_text(value.trim())` recursively (recursion terminates because each nested `{ ... }` substring is strictly shorter than its parent); on `Some(nested_type)`, emit `{key}: {nested_type};` using the same member-formatting the top-level path already uses. Any value text that does not text-match a bare `{ ... }` object literal (e.g. `Object.assign(...)`, other call expressions, template literals) falls through unchanged to the existing `infer_expression_type` fallback, leaving #1238/#1262's still-open false positives untouched. Nesting depth is unbounded by construction (the recursion has no depth cap), matching the real-world `fe-shared` `lexicalTheme` repro's 3+ levels.

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: jet-dts-nested-object-literal-member-inference-verification
requirements:
  deeply_nested_plain_object_literal:
    id: R2
    text: "Real-world shape control: nesting depth 3+ (mirroring the issue's fe-shared `lexicalTheme` hit, e.g. `export const theme = { list: { nested: { listitem: 'x' } } };`), all leaves plain string literals, infers correctly end to end, proving the new recursive branch has no hard-coded depth cap."
    kind: functional
    risk: medium
    verify: cargo test -p jet --lib bundler::dts::tests::infers_exported_const_deeply_nested_plain_object_literal_signature
  flat_object_literal_unchanged:
    id: R4
    text: "No-regression control on the pre-existing #796 flat-object path: a depth-1 plain object literal (`export const flatLiteral = { a: 'x', b: 'y' };`) must keep inferring exactly as before — this pre-existing test must keep passing unchanged after `infer_object_literal_type` is refactored into a Node-adapter plus a new `infer_object_literal_type_from_text` text-only routine, proving the refactor is behavior-preserving for the depth-1 case."
    kind: regression
    risk: low
    verify: cargo test -p jet --lib bundler::dts::tests::infers_plain_object_literal_const_signature
  nested_object_literal_with_untyped_member_still_errors:
    id: R3
    text: "Negative control: the same nested-object-literal member shape as R1, but the nested object's own member value is itself uninferrable (e.g. `heading: { h1: someUntypedImport }` where `someUntypedImport` is a bare identifier with no locally resolvable type), must still raise an isolatedDeclarations error, proving the new recursive branch does not silently widen to accept a nested object literal with a genuinely untyped leaf."
    kind: regression
    risk: high
    verify: cargo test -p jet --lib bundler::dts::tests::uninferrable_exported_const_nested_object_literal_with_untyped_member_errors
  nested_plain_object_literal_minimal_repro:
    id: R1
    text: "WI #1263 minimal repro: an exported const object literal with a member whose value is itself a plain object literal, nesting depth 2, all leaves plain string literals (`export const nestedLiteral = { ltr: 'ltr', heading: { h1: 'editor-heading--h1' } };`), emits `export declare const nestedLiteral: { ltr: string; heading: { h1: string; }; };` instead of an isolatedDeclarations error."
    kind: functional
    risk: medium
    verify: cargo test -p jet --lib bundler::dts::tests::infers_exported_const_nested_plain_object_literal_signature
  object_assign_computed_key_member_unaffected:
    id: R5
    text: "No-regression control proving no entanglement with #1262's still-open Object.assign truncation bug: a pre-existing object-literal member whose value is an `Object.assign({}, ...arr.map(cb))` call expression (not a bare `{ ... }` literal) must keep resolving through the existing call-expression/method-typing path unchanged, proving the new `value.trim().starts_with('{') && ends_with('}')` bare-nested-object-literal branch never text-matches a call-expression-valued member."
    kind: regression
    risk: medium
    verify: cargo test -p jet --lib bundler::dts::tests::infers_object_literal_method_with_object_assign_computed_key_body
---
flowchart TD
    r1[R1 nested plain object literal minimal repro] --> cargo_test_p_jet_lib_bundler_dts_tests_infers_exported_const_nested_plain_object_literal_signature[cargo test -p jet --lib bundler::dts::tests::infers_exported_const_nested_plain_object_literal_signature]
    r2[R2 deeply nested plain object literal] --> cargo_test_p_jet_lib_bundler_dts_tests_infers_exported_const_deeply_nested_plain_object_literal_signature[cargo test -p jet --lib bundler::dts::tests::infers_exported_const_deeply_nested_plain_object_literal_signature]
    r3[R3 nested object literal with untyped member still errors] --> cargo_test_p_jet_lib_bundler_dts_tests_uninferrable_exported_const_nested_object_literal_with_untyped_member_errors[cargo test -p jet --lib bundler::dts::tests::uninferrable_exported_const_nested_object_literal_with_untyped_member_errors]
    r4[R4 flat object literal unchanged] --> cargo_test_p_jet_lib_bundler_dts_tests_infers_plain_object_literal_const_signature[cargo test -p jet --lib bundler::dts::tests::infers_plain_object_literal_const_signature]
    r5[R5 object assign computed key member unaffected] --> cargo_test_p_jet_lib_bundler_dts_tests_infers_object_literal_method_with_object_assign_computed_key_body[cargo test -p jet --lib bundler::dts::tests::infers_object_literal_method_with_object_assign_computed_key_body]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/jet/src/bundler/dts.rs
    action: update
    section: logic
    impl_mode: hand-written
    reason: "Empirically re-verified against WI #1263's exact minimal repro (`jet build --lib --format esm --dts` on a standalone `export const flatLiteral = { ltr: 'ltr', rtl: 'rtl' }; export const nestedLiteral = { ltr: 'ltr', heading: { h1: 'editor-heading--h1' } };`) still fails on the current app/jet source tree (post-#1264, commit 75cb5e5ca), so this is a genuine, still-open analyzer gap distinct from #1264's arrow-body-return shape (not an already-implemented case pinned only by tests, contrast WI #937). Factor infer_object_literal_type's body (everything after the Node-to-text extraction step, which is already pure &str manipulation via split_top_level/split_once_top_level) out into a new infer_object_literal_type_from_text(text: &str) -> Option<String> that needs no tree-sitter Node; infer_object_literal_type(node, source) becomes a thin Node-to-text adapter delegating to it. In the member-value-typing chain (after the existing infer_arrow_function_type_from_text check, before the infer_expression_type fallback), add a branch: if value.trim() starts_with('{') and ends_with('}'), recurse via infer_object_literal_type_from_text(value.trim()) and emit `{key}: { nested members }` on success. Recursion terminates because each nested `{ ... }` substring is strictly shorter than its parent, so nesting depth is unbounded by construction. Any value text that is not a bare `{ ... }` literal (Object.assign(...) calls, other call expressions, template literals) falls through unchanged to the existing infer_expression_type fallback, leaving #1238's and #1262's still-open false positives (Object.assign+computed-key body shapes, post-Object.assign-spread property truncation) untouched -- this TD is scoped to plain nested object-literal member values only."
  - path: projects/jet/src/bundler/dts.rs
    action: update
    section: unit-test
    impl_mode: hand-written
    reason: "Add the R1-R5 regression tests specified in the unit-test section to the existing mod tests block: R1 pins the WI #1263 minimal-repro positive case (nesting depth 2, all leaves plain string literals); R2 is a positive control at nesting depth 3+ mirroring the issue's real-world fe-shared lexicalTheme shape, proving the recursion has no hard-coded depth cap; R3 is a negative control proving a nested object literal with a genuinely untyped leaf still raises isolatedDeclarations error; R4 and R5 re-assert two pre-existing tests (flat depth-1 object literal from #796, and the Object.assign+computed-key member from the #1262/#1238 family) keep passing unchanged, proving the infer_object_literal_type refactor and the new bare-nested-object-literal branch are behavior-preserving outside the exact new shape."
```
