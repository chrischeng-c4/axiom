---
id: projects-jet-logic-jet-lib-dts-isolateddeclarations-false-positive-on-arrow-functio-md
fill_sections: [logic, unit-test, changes]
capability_refs:
  - id: library-build-publishing
    role: primary
    gap: library-publishing-readiness
    claim: type-declaration-emission
    coverage: partial
    rationale: "isolatedDeclarations-style declaration emission must accept const arrow functions whose parameter and return types are explicit on the arrow signature."
---

# jet --lib --dts: Const Arrow Function Declaration Type Synthesis

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
(fill)
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/jet/src/bundler/dts.rs"
    action: modify
    section: logic
    description: |
      Teach infer_variable_declarator_type to recognize arrow_function
      initializers with explicit parameter types and a return_type, then
      synthesize the exported const declaration type as `(params) => Return`.
    impl_mode: hand-written
  - path: "projects/jet/src/bundler/dts.rs"
    action: modify
    section: unit-test
    description: |
      Add emitter-level coverage for exported const arrow functions with
      explicit return annotations and for untyped arrow params remaining
      fail-loud.
    impl_mode: hand-written
  - path: "projects/jet/tests/build/library_dts.rs"
    action: modify
    section: unit-test
    description: |
      Add library-build regression coverage matching the reported delay
      function shape and asserting the emitted .d.ts matches TypeScript's
      isolatedDeclarations output.
    impl_mode: hand-written
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: jet-lib-dts-arrow-const-tests
requirements:
  R1:
    text: "An exported const arrow function with typed parameters and explicit return type emits a callable declaration type."
    risk: high
    verify: unit
  R2:
    text: "A library build with the reported delay arrow function emits index.d.ts instead of failing isolatedDeclarations."
    risk: high
    verify: unit
  R3:
    text: "Arrow consts without typed parameters or return annotations still fail loudly instead of emitting implicit any."
    risk: high
    verify: unit
---
requirementDiagram
requirement R1 {
  id: R1
  text: "Typed arrow const emits"
  risk: High
  verifymethod: Test
}
requirement R2 {
  id: R2
  text: "Library build accepts delay arrow"
  risk: High
  verifymethod: Test
}
requirement R3 {
  id: R3
  text: "Untyped arrow still fails"
  risk: High
  verifymethod: Test
}
```
