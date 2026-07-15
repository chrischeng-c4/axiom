---
id: jsonl-str-primitives
fill_sections: [overview, schema, logic, tests, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# JSONL String Primitives

## Overview
<!-- type: overview lang: markdown -->

This spec introduces two new primitive vocabulary entries to the Mermaid Plus
flowchart code generation system: `parse_jsonl_str` and `serialize_jsonl_line`.

`parse_jsonl_str` reads a JSONL string (for example, an event log already
loaded into memory) and deserialises each non-empty line via
`serde_json::from_str`, silently dropping malformed lines. Output is
`Vec<T>` where T is the target struct. This is infallible at the primitive
level — parse errors are isolated to their individual records and skipped.

`serialize_jsonl_line` serialises a single value to a JSON string with a
trailing newline character, suitable for POSIX-atomic `O_APPEND` writes to a
JSONL log file. It is fallible (`serde_json::to_string` returns `Result`)
and the `?` operator propagates `serde_json::Error` to the enclosing function.

These entries are transport-neutral generator vocabulary for any structured
JSONL record stream. They remain canonical even when an individual consumer is
removed because Mermaid Plus diagrams can bind their own record type through
`type_args`.

The PrimitiveKind enum in `flowchart_plus/schema.rs` gains two new variants
(`ParseJsonlStr`, `SerializeJsonlLine`) and the static `REGISTRY` in
`primitive_registry.rs` gains two new `PrimitiveEntry` rows. The registry
size assertion in tests is updated from 15 to 17.
## Schema: ParseJsonlStr and SerializeJsonlLine
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: jsonl-str-primitives#schema
title: JSONL String Primitive Entries
description: >
  Schema additions for two new PrimitiveKind variants and their PrimitiveEntry
  rows in the static REGISTRY. They support generated parsing and serialization
  for typed JSONL record streams without coupling the vocabulary to a transport.

definitions:
  ParseJsonlStrEntry:
    $ref: "mermaid-plus-primitive-vocabulary#/definitions/PrimitiveEntry"
    description: >
      parse_jsonl_str: splits a &str on newlines, deserialises each non-empty
      line via serde_json::from_str::<T>(), and collects successes into Vec<T>.
      Malformed lines are silently dropped (infallible at primitive level).
    x-entry:
      name: parse_jsonl_str
      category: serde
      inputs:
        - name: content
          field_type: string
      output_type: "Vec<T>"
      generic_params: [T]
      fallible: false
      emit_template_id: parse_jsonl_str
      emit_template: >-
        let {out}: Vec<{T}> = {content}.lines()
          .filter(|l| !l.trim().is_empty())
          .filter_map(|l| serde_json::from_str(l.trim()).ok())
          .collect();

  SerializeJsonlLineEntry:
    $ref: "mermaid-plus-primitive-vocabulary#/definitions/PrimitiveEntry"
    description: >
      serialize_jsonl_line: serialises a &T: Serialize to a JSON string and
      appends a newline, producing a single JSONL line ready for O_APPEND write.
      Fallible: serde_json::to_string returns Result; the ? operator propagates
      serde_json::Error to the enclosing function's Result return type.
    x-entry:
      name: serialize_jsonl_line
      category: serde
      inputs:
        - name: value
          field_type: T
      output_type: string
      generic_params: [T]
      fallible: true
      emit_template_id: serialize_jsonl_line
      emit_template: >-
        let {out} = format!("{}\n", serde_json::to_string(&{value})?);

  PrimitiveKindExtension:
    description: >
      Two new variants appended to the PrimitiveKind enum in
      apps/agentic-workflow/src/generate/diagrams/flowchart_plus/schema.rs.
      Placed in the JSONL stream IO group alongside existing variants.
    type: object
    properties:
      variants:
        type: array
        items:
          type: string
        x-values:
          - ParseJsonlStr
          - SerializeJsonlLine
```
## Logic: parse and serialize JSONL records
<!-- type: logic lang: mermaid -->

```mermaid
---
id: parse-serialize-jsonl-primitives
entry: start
nodes:
  start:
    kind: start
    label: "Begin"
  parse_lines:
    kind: process
    label: "parse_jsonl_str(content)"
    primitive: parse_jsonl_str
    inputs:
      content: content
    output_binding: records
    type_args:
      T: EventRecord
  parse_done:
    kind: terminal
    label: "Return Vec<EventRecord>"
  serialize_start:
    kind: start
    label: "Begin"
  serialize_line:
    kind: process
    label: "serialize_jsonl_line(record)"
    primitive: serialize_jsonl_line
    inputs:
      value: record
    output_binding: line
    type_args:
      T: EventRecord
  serialize_done:
    kind: terminal
    label: "Return line: String"
edges:
  - from: start
    to: parse_lines
  - from: parse_lines
    to: parse_done
  - from: serialize_start
    to: serialize_line
  - from: serialize_line
    to: serialize_done
---
flowchart TD
    start([Begin]) --> parse_lines["parse_jsonl_str(content)\nprimitive: parse_jsonl_str\noutput: records: Vec<EventRecord>"]
    parse_lines --> parse_done([Return Vec<EventRecord>])

    serialize_start([Begin]) --> serialize_line["serialize_jsonl_line(record)\nprimitive: serialize_jsonl_line\noutput: line: String"]
    serialize_line --> serialize_done([Return line: String])
```
## Tests: primitive registry coverage
<!-- type: tests lang: yaml -->

```yaml
tests:
  - id: T1
    name: test_lookup_parse_jsonl_str_returns_entry
    kind: unit
    description: >
      parse_jsonl_str is registered in REGISTRY with infallible=false,
      output_type Vec<T>, and an emit_template containing both
      serde_json::from_str and .lines().
    setup:
      import: crate::generate::generators::primitive_registry
    assertions:
      - expr: "lookup(&PrimitiveKind::ParseJsonlStr).is_some()"
        expect: true
      - expr: "lookup(&PrimitiveKind::ParseJsonlStr).unwrap().name"
        expect: '"parse_jsonl_str"'
      - expr: "lookup(&PrimitiveKind::ParseJsonlStr).unwrap().fallible"
        expect: false
      - expr: "lookup(&PrimitiveKind::ParseJsonlStr).unwrap().output_type"
        expect: '"Vec<T>"'
      - expr: "lookup(&PrimitiveKind::ParseJsonlStr).unwrap().emit_template.contains(\"serde_json::from_str\")"
        expect: true
      - expr: "lookup(&PrimitiveKind::ParseJsonlStr).unwrap().emit_template.contains(\".lines()\")"
        expect: true

  - id: T2
    name: test_lookup_serialize_jsonl_line_returns_entry
    kind: unit
    description: >
      serialize_jsonl_line is registered in REGISTRY with fallible=true,
      output_type string, and an emit_template containing serde_json::to_string
      and a trailing newline.
    setup:
      import: crate::generate::generators::primitive_registry
    assertions:
      - expr: "lookup(&PrimitiveKind::SerializeJsonlLine).is_some()"
        expect: true
      - expr: "lookup(&PrimitiveKind::SerializeJsonlLine).unwrap().name"
        expect: '"serialize_jsonl_line"'
      - expr: "lookup(&PrimitiveKind::SerializeJsonlLine).unwrap().fallible"
        expect: true
      - expr: "lookup(&PrimitiveKind::SerializeJsonlLine).unwrap().output_type"
        expect: '"string"'
      - expr: "lookup(&PrimitiveKind::SerializeJsonlLine).unwrap().emit_template.contains(\"serde_json::to_string\")"
        expect: true
      - expr: "lookup(&PrimitiveKind::SerializeJsonlLine).unwrap().emit_template.contains(\"\\\\n\")"
        expect: true

  - id: T3
    name: test_registry_has_seventeen_entries
    kind: unit
    description: >
      REGISTRY has exactly 17 canonical entries, including parse_jsonl_str and
      serialize_jsonl_line.
    setup:
      import: crate::generate::generators::primitive_registry
    assertions:
      - expr: "REGISTRY.len()"
        expect: 17
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/agentic-workflow/src/generate/diagrams/flowchart_plus/schema.rs
    action: modify
    section: schema
    impl_mode: hand-written
    description: >
      Add two PrimitiveKind variants: ParseJsonlStr and SerializeJsonlLine.
      Place them in the JSONL stream IO group alongside ParseJsonlStream and
      AppendLineAtomic. The additions are inside the existing CODEGEN-BEGIN
      (primitive-vocabulary) block since the file is spec-managed.

  - path: apps/agentic-workflow/src/generate/generators/primitive_registry.rs
    action: modify
    section: schema
    impl_mode: hand-written
    description: >
      Add two PrimitiveEntry rows to the static REGISTRY constant for
      parse_jsonl_str and serialize_jsonl_line, using the emit_template strings
      defined in this spec. Add two arms to kind_to_name for ParseJsonlStr and
      SerializeJsonlLine. Update the registry-size assertion in tests from 15
      to 17. Add tests T1 (test_lookup_parse_jsonl_str_returns_entry),
      T2 (test_lookup_serialize_jsonl_line_returns_entry), and T3
      (test_registry_has_seventeen_entries) in the existing tests module.

  - path: apps/agentic-workflow/tech-design/surface/specs/mermaid-plus-primitive-vocabulary.md
    action: modify
    section: schema
    impl_mode: hand-written
    description: >
      Add parse_jsonl_str and serialize_jsonl_line to the PrimitiveKind enum
      list in the schema section, grouped with the JSONL stream IO comment
      alongside parse_jsonl_stream, append_line_atomic, and run_subprocess.
      Add full PrimitiveEntry rows for both new primitives in the serde
      category x-entries block.

  - action: annotate
    section: unit-test
    impl_mode: hand-written
    description: "Traceability metadata edge for the unit-test section."

```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->

**Verdict:** approved

- [schema] `serialize_jsonl_line` correctly has `fallible: true` and the emit template uses `format!("{}\n", serde_json::to_string(&{value})?)` — semantically equivalent to the R4 formulation and unambiguous for codegen.
- [schema] `parse_jsonl_str` emit template correctly chains `.lines()`, blank-line filter, and `.filter_map(...ok())` into `Vec<{T}>` — matches the infallible contract.
- [changes] The primitive registry and vocabulary spec remain the canonical owners of the transport-neutral JSONL entries.
- [tests] T1–T3 cover registration, fallibility flag, output_type, and template content for both primitives plus the registry-size assertion bump to 17 — sufficient for R9/R10 gate.
- [logic] Two-flow Mermaid Plus diagram correctly encodes primitive-bound nodes with `type_args: T: EventRecord` for both operations.
