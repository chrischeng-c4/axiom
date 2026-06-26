---
id: vat-remaining-rust-source-unit-promotion
summary: >
  Promote remaining vat source/test/build files from group-level semantic
  ownership to per-file rust-source-unit TD AST codegen.
capability_refs:
  - id: "agent-native-gpu-native-dev-containers"
    role: primary
    claim: "local-agent-test-runner-protocol"
    coverage: partial
    rationale: >
      Removes the remaining grouped source ownership that blocks #39's vat
      migration to td_ast rust-source-unit replay.
fill_sections: [logic]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: vat-rust-source-unit-promotion
entry: start
nodes:
  start: { kind: start, label: "vat grouped source/test/build files are not per-file rust-source-unit TD AST" }
  inventory: { kind: process, label: "Collect group-level semantic schema markers in vat source/test/build files" }
  spec: { kind: process, label: "Create or refresh rust-source-unit TD for each remaining target file" }
  generate: { kind: process, label: "Run aw td gen-source --spec <source-unit-td> --target <file>" }
  verify: { kind: process, label: "Run gen-source dry-run, aw generator check, TD lock, and TD checks" }
  done: { kind: terminal, label: "vat grouped source units are promoted to td_ast rust-source-unit ownership" }
edges:
  - { from: start, to: inventory }
  - { from: inventory, to: spec }
  - { from: spec, to: generate }
  - { from: generate, to: verify }
  - { from: verify, to: done }
---
flowchart TD
    start([vat grouped source units]) --> inventory[collect group-level markers]
    inventory --> spec[create or refresh per-file rust-source-unit TDs]
    spec --> generate[aw td gen-source per target]
    generate --> verify[dry-run + generator check + TD checks]
    verify --> done([td_ast rust-source-unit ownership])
```
