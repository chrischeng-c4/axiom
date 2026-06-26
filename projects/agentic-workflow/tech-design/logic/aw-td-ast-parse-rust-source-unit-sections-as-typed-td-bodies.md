---
id: aw-td-ast-parse-rust-source-unit-sections-as-typed-td-bodies
summary: Parse rust-source-unit TD sections into typed TD AST bodies and dispatch them as structural generator input.
fill_sections: [logic, unit-test]
---

# TD: aw td_ast parse rust-source-unit sections as typed TD bodies

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: aw-td-ast-rust-source-unit-typed-body
entry: parse_section
nodes:
  parse_section: { kind: start, label: "parse_td_str walks TD sections" }
  registry: { kind: process, label: "SectionKind::for_section_type(RustSourceUnit) => RustSourceUnitFamily" }
  fence: { kind: decision, label: "section has fenced Rust body?" }
  placeholder: { kind: terminal, label: "empty or missing fence => TypedBody::Placeholder" }
  parse_rust: { kind: process, label: "rust_source_unit::parse(content)" }
  parse_ok: { kind: decision, label: "ra_ap_syntax parse clean?" }
  typed_error: { kind: terminal, label: "return TypedPayloadParse with rust-source-unit parse error" }
  typed_body: { kind: terminal, label: "TypedBody::RustSourceUnit(unit) + content_hash(unit)" }
  dispatch: { kind: start, label: "dispatch_from_tdast sees TypedBody::RustSourceUnit" }
  structural: { kind: terminal, label: "DispatchOutcome emitted by rust-source-unit; StructuralGenerator; source_backed=false" }
edges:
  - { from: parse_section, to: registry }
  - { from: registry, to: fence }
  - { from: fence, to: placeholder, label: "no" }
  - { from: fence, to: parse_rust, label: "yes" }
  - { from: parse_rust, to: parse_ok }
  - { from: parse_ok, to: typed_error, label: "errors" }
  - { from: parse_ok, to: typed_body, label: "clean" }
  - { from: dispatch, to: structural }
---
flowchart TD
  parse_section([parse_td_str walks TD sections]) --> registry[SectionKind RustSourceUnit maps to RustSourceUnitFamily]
  registry --> fence{fenced Rust body?}
  fence -->|no| placeholder([TypedBody::Placeholder])
  fence -->|yes| parse_rust[rust_source_unit::parse content]
  parse_rust --> parse_ok{clean parse?}
  parse_ok -->|errors| typed_error([TypedPayloadParse rust-source-unit parse error])
  parse_ok -->|clean| typed_body([TypedBody::RustSourceUnit plus content_hash])
  dispatch([dispatch_from_tdast]) --> structural([rust-source-unit StructuralGenerator source_backed=false])
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: aw-td-ast-rust-source-unit-unit-tests
requirements:
  typed_parse:
    id: R1
    text: "A valid rust-source-unit TD section parses to TypedBody::RustSourceUnit and carries a content hash."
    kind: functional
    risk: high
    verify: test
  parse_error:
    id: R2
    text: "Invalid Rust in a rust-source-unit TD section returns a typed-payload parse error instead of an unsupported body."
    kind: functional
    risk: high
    verify: test
  dispatch_route:
    id: R3
    text: "dispatch_from_tdast classifies TypedBody::RustSourceUnit as an emitted rust-source-unit structural generator with source_backed=false."
    kind: functional
    risk: high
    verify: test
elements:
  parse_td_str_parses_rust_source_unit_body:
    kind: test
    type: "rs/#[test]"
  parse_td_str_rejects_invalid_rust_source_unit:
    kind: test
    type: "rs/#[test]"
  rust_source_unit_dispatch_routes_as_structural_generator:
    kind: test
    type: "rs/#[test]"
relations:
  - { from: parse_td_str_parses_rust_source_unit_body, verifies: typed_parse }
  - { from: parse_td_str_rejects_invalid_rust_source_unit, verifies: parse_error }
  - { from: rust_source_unit_dispatch_routes_as_structural_generator, verifies: dispatch_route }
---
requirementDiagram
    requirement R1 {
      id: R1
      text: "valid rust-source-unit parses as typed body"
      risk: high
      verifymethod: test
    }
    requirement R2 {
      id: R2
      text: "invalid rust-source-unit returns typed parse error"
      risk: high
      verifymethod: test
    }
    requirement R3 {
      id: R3
      text: "dispatch reports structural rust-source-unit generator"
      risk: high
      verifymethod: test
    }
    element parse_td_str_parses_rust_source_unit_body {
      type: "rs/#[test]"
    }
    element parse_td_str_rejects_invalid_rust_source_unit {
      type: "rs/#[test]"
    }
    element rust_source_unit_dispatch_routes_as_structural_generator {
      type: "rs/#[test]"
    }
```
