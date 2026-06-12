---
id: rust-source-unit-ir
fill_sections: [logic]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "The rust-source-unit generator captures a Rust file as a structured item-tree IR (lossless CST) and emits it byte-identically, replacing brittle source-replay for Rust units."
---

# Tech Design: rust-source-unit IR

Captures a Rust source unit as a structured, editable item-tree AST and emits it
byte-identically — the foundation for moving Rust units off source-replay to td_ast.

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: rust-source-unit-roundtrip
entry: parse
nodes:
  parse:    { kind: start,    label: "ra_ap_syntax parse(text, Edition2021)" }
  check:    { kind: decision, label: "parse_errors == 0?" }
  err:      { kind: terminal, label: "Return ParseError (not a clean unit)" }
  walk:     { kind: process,  label: "walk children_with_tokens in order" }
  classify: { kind: decision, label: "element is a Node (item)?" }
  item:     { kind: process,  label: "extract structured Item: kind/name/attrs/doc/sig-or-fields/body-CST" }
  trivia:   { kind: process,  label: "capture Trivia verbatim (whitespace + comments)" }
  ir:       { kind: process,  label: "RustSourceUnit IR = ordered [Item|Trivia] (structured, not a text blob)" }
  emit:     { kind: process,  label: "emit each segment: unchanged -> lossless CST; edited -> re-render" }
  done:     { kind: terminal, label: "byte-identical regen (td_ast origin band)" }
edges:
  - { from: parse,    to: check }
  - { from: check,    to: err,    label: "no" }
  - { from: check,    to: walk,   label: "yes" }
  - { from: walk,     to: classify }
  - { from: classify, to: item,   label: "yes" }
  - { from: classify, to: trivia, label: "no" }
  - { from: item,     to: ir }
  - { from: trivia,   to: ir }
  - { from: ir,       to: emit }
  - { from: emit,     to: done }
---
flowchart TD
    parse([ra_ap_syntax parse]) --> check{parse_errors == 0?}
    check -->|no| err([Return ParseError])
    check -->|yes| walk[walk children_with_tokens in order]
    walk --> classify{element is a Node item?}
    classify -->|yes| item[extract structured Item]
    classify -->|no| trivia[capture Trivia verbatim]
    item --> ir[RustSourceUnit IR ordered segments]
    trivia --> ir
    ir --> emit[emit: unchanged lossless / edited re-render]
    emit --> done([byte-identical regen td_ast origin])
```

# Reviews

### Review 1
**Verdict:** approved

- [logic] Logic section is a valid Mermaid Plus block (id rust-source-unit-roundtrip; nodes/edges cover parse -> classify item/trivia -> ordered RustSourceUnit IR -> segment emit -> byte-identical regen). Matches the proven POC pipeline (lossless CST capture, structured item extraction, surgical re-render). Applicability is correct for this atomic library WI: the unit owns parse->IR->emit only; dispatch and health-metric wiring are out of scope.

# Reviews

### Review 1
**Verdict:** approved

- [logic] Logic section is a valid Mermaid Plus block (id rust-source-unit-roundtrip; nodes/edges cover parse -> classify item/trivia -> ordered RustSourceUnit IR -> segment emit -> byte-identical regen). Matches the proven POC pipeline (lossless CST capture, structured item extraction, surgical re-render). Applicability is correct for this atomic library WI: the unit owns parse->IR->emit only; dispatch and health-metric wiring are out of scope.
