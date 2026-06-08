---
id: sdd-codegen-trait-impl
fill_sections: [overview, requirements, schema, logic, source, test-plan, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# Trait-Impl Generator

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generate/generators/trait_impl.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `MatchArm` | projects/agentic-workflow/src/generate/generators/trait_impl.rs | struct | pub | 17 |  |
| `TraitImplOutput` | projects/agentic-workflow/src/generate/generators/trait_impl.rs | struct | pub | 25 |  |
| `TraitImplSpec` | projects/agentic-workflow/src/generate/generators/trait_impl.rs | struct | pub | 36 |  |
| `TraitMethod` | projects/agentic-workflow/src/generate/generators/trait_impl.rs | struct | pub | 48 |  |
| `run_trait_impl` | projects/agentic-workflow/src/generate/generators/trait_impl.rs | function | pub | 73 | run_trait_impl(spec: &TraitImplSpec, spec_ref: Option<String>) -> TraitImplOutput |
## Requirements
<!-- type: requirements lang: mermaid -->

```mermaid
---
id: sdd-codegen-trait-impl-requirements
requirements:
  emit_impl_block:
    id: R1
    text: "run_trait_impl MUST emit an impl <Trait> for <Type> { ... } block."
    kind: functional
    risk: high
    verify: test
  emit_method_match:
    id: R2
    text: "Each TraitMethod MUST emit a fn body that match-dispatches on self over body_lookup entries."
    kind: functional
    risk: high
    verify: test
  empty_methods:
    id: R3
    text: "An empty methods list MUST emit an impl block with an empty body (no panic, valid Rust)."
    kind: functional
    risk: high
    verify: test
  codegen_markers:
    id: R4
    text: "Emitted output MUST plug into CODEGEN-BEGIN/CODEGEN-END markers via the lines field."
    kind: functional
    risk: high
    verify: test
  unit_tests:
    id: R5
    text: "Generator MUST have unit tests covering empty, single-method, multi-arm, multi-method cases."
    kind: functional
    risk: high
    verify: test
elements: {}
relations: []
---
requirementDiagram
    requirement R1 {
      id: R1
      text: "run_trait_impl emits impl Trait for Type block"
      risk: High
      verifymethod: Test
    }
    requirement R2 {
      id: R2
      text: "Each method emits match-dispatch body"
      risk: High
      verifymethod: Test
    }
    requirement R3 {
      id: R3
      text: "empty methods list emits empty impl body"
      risk: High
      verifymethod: Test
    }
    requirement R4 {
      id: R4
      text: "output plugs into CODEGEN-BEGIN/END markers"
      risk: High
      verifymethod: Test
    }
    requirement R5 {
      id: R5
      text: "unit tests cover empty single multi-arm multi-method cases"
      risk: High
      verifymethod: Test
    }
```

## Schema
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: sdd-codegen-trait-impl#schema
title: Trait-Impl Generator Type Definitions
description: >
  Type declarations for the trait-impl codegen primitive in
  projects/agentic-workflow/src/generate/generators/trait_impl.rs.

definitions:
  TraitMethod:
    type: object
    $id: TraitMethod
    required: [name, signature, body_lookup]
    description: >
      One method inside a TraitImplSpec. The signature carries the full Rust
      fn header (everything from fn through the opening { brace). body_lookup
      is an ordered map from match-arm pattern (key) to expression (value);
      each entry emits one `<key> => <value>,` line in the match body.
    properties:
      name:
        type: string
        description: "Method identifier, used for documentation only (the signature carries it too)."
      signature:
        type: string
        description: "Full Rust fn signature from `fn` through the opening `{`."
      body_lookup:
        type: array
        items:
          type: object
          required: [pattern, expression]
          properties:
            pattern:
              type: string
            expression:
              type: string
        x-rust-type: "Vec<MatchArm>"
        description: "Ordered list of (pattern, expression) match arms."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize, PartialEq, Eq]

  MatchArm:
    type: object
    $id: MatchArm
    required: [pattern, expression]
    description: "One `<pattern> => <expression>,` line in a match self body."
    properties:
      pattern:
        type: string
      expression:
        type: string
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize, PartialEq, Eq]

  TraitImplSpec:
    type: object
    $id: TraitImplSpec
    required: [trait_name, type_name, methods]
    description: >
      Input descriptor for the trait-impl generator, sourced from the
      trait_impl: field of a spec change entry (R1, R2).
    properties:
      trait_name:
        type: string
        description: "Trait identifier used in `impl <trait_name> for <type_name>`."
      type_name:
        type: string
        description: "Type identifier used in `impl <trait_name> for <type_name>`."
      methods:
        type: array
        items:
          $ref: "#/definitions/TraitMethod"
        x-rust-type: "Vec<TraitMethod>"
        description: "Ordered list of methods inside the impl block (R2, R3)."
    x-rust-struct:
      derive: [Debug, Clone, Default, Serialize, Deserialize]

  TraitImplOutput:
    type: object
    $id: TraitImplOutput
    required: [lines]
    description: >
      Result of running the trait-impl generator. lines plug into
      CODEGEN-BEGIN/CODEGEN-END markers (R4).
    properties:
      lines:
        type: array
        items:
          type: string
        x-rust-type: "Vec<String>"
        description: "Generated source lines (impl block declaration + body)."
      spec_ref:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        x-serde-skip-if: "Option::is_none"
        description: "SPEC-REF anchor string for the CODEGEN marker header."
    x-rust-struct:
      derive: [Debug, Clone, Default, Serialize, Deserialize]
```

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: trait-impl-emit
entry: start
nodes:
  start:        { kind: start,    label: "run_trait_impl(spec, spec_ref)" }
  init_lines:   { kind: process,  label: "Initialize output lines Vec" }
  emit_spec:    { kind: process,  label: "If spec_ref present, append /// @spec <spec_ref>" }
  emit_open:    { kind: process,  label: "Append: impl <trait> for <type> {" }
  iter_method:  { kind: process,  label: "Take next TraitMethod" }
  check_more:   { kind: decision, label: "More methods?" }
  emit_sig:     { kind: process,  label: "Append: <signature>" }
  emit_match:   { kind: process,  label: "Append: match self {" }
  iter_arm:     { kind: process,  label: "Take next MatchArm" }
  check_arm:    { kind: decision, label: "More arms?" }
  emit_arm:     { kind: process,  label: "Append: <pattern> => <expression>," }
  emit_match_close: { kind: process, label: "Append: }" }
  emit_fn_close: { kind: process, label: "Append: }" }
  emit_close:   { kind: process,  label: "Append: }" }
  return_ok:    { kind: terminal, label: "Return TraitImplOutput { lines, spec_ref }" }
edges:
  - { from: start,        to: init_lines }
  - { from: init_lines,   to: emit_spec }
  - { from: emit_spec,    to: emit_open }
  - { from: emit_open,    to: iter_method }
  - { from: iter_method,  to: check_more }
  - { from: check_more,   to: emit_close,    label: "no" }
  - { from: check_more,   to: emit_sig,      label: "yes" }
  - { from: emit_sig,     to: emit_match }
  - { from: emit_match,   to: iter_arm }
  - { from: iter_arm,     to: check_arm }
  - { from: check_arm,    to: emit_match_close, label: "no" }
  - { from: check_arm,    to: emit_arm,         label: "yes" }
  - { from: emit_arm,     to: iter_arm }
  - { from: emit_match_close, to: emit_fn_close }
  - { from: emit_fn_close, to: iter_method }
  - { from: emit_close,   to: return_ok }
---
flowchart TD
    start([run_trait_impl spec spec_ref]) --> init_lines[Initialize output lines Vec]
    init_lines --> emit_spec[Append @spec marker when spec_ref present]
    emit_spec --> emit_open[Append impl Trait for Type open]
    emit_open --> iter_method[Take next TraitMethod]
    iter_method --> check_more{More methods?}
    check_more -->|no| emit_close[Append closing brace]
    check_more -->|yes| emit_sig[Append signature line]
    emit_sig --> emit_match[Append match self open]
    emit_match --> iter_arm[Take next MatchArm]
    iter_arm --> check_arm{More arms?}
    check_arm -->|no| emit_match_close[Append closing match brace]
    check_arm -->|yes| emit_arm[Append pattern arrow expression]
    emit_arm --> iter_arm
    emit_match_close --> emit_fn_close[Append closing fn brace]
    emit_fn_close --> iter_method
    emit_close --> return_ok([Return TraitImplOutput lines spec_ref])
```

## Test Plan
<!-- type: test-plan lang: mermaid -->

```mermaid
---
id: sdd-codegen-trait-impl-test-plan
requirements:
  emit_impl_block:
    id: R1
    text: "run_trait_impl emits impl Trait for Type block"
    kind: functional
    risk: high
    verify: test
  emit_method_match:
    id: R2
    text: "Each TraitMethod emits a fn body that match-dispatches on self"
    kind: functional
    risk: high
    verify: test
  empty_methods:
    id: R3
    text: "Empty methods list emits empty impl body"
    kind: functional
    risk: high
    verify: test
  codegen_markers:
    id: R4
    text: "Output plugs into CODEGEN-BEGIN/END markers"
    kind: functional
    risk: high
    verify: test
elements:
  test_empty_methods:
    kind: test
    type: "rs/#[test]"
  test_single_method_single_arm:
    kind: test
    type: "rs/#[test]"
  test_single_method_multi_arm:
    kind: test
    type: "rs/#[test]"
  test_multi_method:
    kind: test
    type: "rs/#[test]"
relations:
  - { from: test_empty_methods,           verifies: empty_methods }
  - { from: test_single_method_single_arm, verifies: emit_impl_block }
  - { from: test_single_method_single_arm, verifies: emit_method_match }
  - { from: test_single_method_multi_arm,  verifies: emit_method_match }
  - { from: test_multi_method,             verifies: emit_method_match }
---
requirementDiagram
    requirement R1 {
      id: R1
      text: "run_trait_impl emits impl Trait for Type block"
      risk: High
      verifymethod: Test
    }
    requirement R2 {
      id: R2
      text: "Each method emits match-dispatch body"
      risk: High
      verifymethod: Test
    }
    requirement R3 {
      id: R3
      text: "Empty methods list emits empty impl body"
      risk: High
      verifymethod: Test
    }
    requirement R4 {
      id: R4
      text: "Output plugs into CODEGEN-BEGIN/END markers"
      risk: High
      verifymethod: Test
    }
    element test_empty_methods {
      type: "rs/#[test]"
    }
    element test_single_method_single_arm {
      type: "rs/#[test]"
    }
    element test_single_method_multi_arm {
      type: "rs/#[test]"
    }
    element test_multi_method {
      type: "rs/#[test]"
    }
    test_empty_methods - verifies -> R3
    test_single_method_single_arm - verifies -> R1
    test_single_method_single_arm - verifies -> R2
    test_single_method_multi_arm - verifies -> R2
    test_multi_method - verifies -> R2
```

## Source
<!-- type: source lang: rust -->
<!-- source-from-target: handwrite-gap missing-generator:logic -->

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/generators/trait_impl.rs
    action: create
    section: schema
    impl_mode: codegen
    description: >
      New module: TraitMethod, MatchArm, TraitImplSpec, TraitImplOutput struct
      declarations generated from sdd-codegen-trait-impl#schema. CODEGEN-BEGIN/END
      blocks with @spec markers.

  - path: projects/agentic-workflow/src/generate/generators/trait_impl.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:missing-generator:logic>"
    description: >
      run_trait_impl(spec: &TraitImplSpec, spec_ref: Option<String>) ->
      TraitImplOutput implementing the logic flowchart in
      sdd-codegen-trait-impl#logic.

  - path: projects/agentic-workflow/src/generate/generators/mod.rs
    action: modify
    section: schema
    impl_mode: hand-written
    description: >
      Declare pub mod trait_impl and re-export run_trait_impl, TraitImplSpec,
      TraitMethod, MatchArm, TraitImplOutput.
  - action: annotate
    section: logic
    impl_mode: hand-written
    description: "Traceability metadata edge for the logic section."

  - action: annotate
    section: requirements
    impl_mode: hand-written
    description: "Traceability metadata edge for the requirements section."

  - action: annotate
    section: unit-test
    impl_mode: hand-written
    description: "Traceability metadata edge for the unit-test section."

```
