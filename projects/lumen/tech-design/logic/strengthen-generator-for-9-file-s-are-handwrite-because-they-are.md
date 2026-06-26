---
id: lumen-rust-source-unit-generator-gap
summary: >
  Promote the remaining lumen HANDWRITE-owned source/test/build artifacts into
  td_ast rust-source-unit ownership by creating per-source TDs and regenerating
  the files with `aw td gen-source`.
capability_refs:
  - id: "cli-interface"
    role: primary
    claim: "service-process-interface"
    coverage: partial
    rationale: >
      Removes the remaining tracked HANDWRITE source ownership that blocks
      generator request readiness for #39's lumen migration slice.
fill_sections: [logic]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: lumen-rust-source-unit-promotion
entry: start
nodes:
  start: { kind: start, label: "generator gap: 9 lumen files are HANDWRITE, not td_ast codegen" }
  inventory: { kind: process, label: "Collect HANDWRITE markers in lumen source/test/build files" }
  spec: { kind: process, label: "Create or refresh rust-source-unit TD for each target file" }
  generate: { kind: process, label: "Run aw td gen-source --spec <source-unit-td> --target <file>" }
  verify: { kind: process, label: "Run gen-source --dry-run, aw generator check, and TD checks" }
  done: { kind: terminal, label: "Lumen gap moves from HANDWRITE source ownership to td_ast rust-source-unit ownership" }
edges:
  - { from: start, to: inventory }
  - { from: inventory, to: spec }
  - { from: spec, to: generate }
  - { from: generate, to: verify }
  - { from: verify, to: done }
---
flowchart TD
    start([9 lumen HANDWRITE files]) --> inventory[collect target files]
    inventory --> spec[create or refresh per-source rust-source-unit TDs]
    spec --> generate[aw td gen-source per target]
    generate --> verify[dry-run + generator check + TD checks]
    verify --> done([td_ast rust-source-unit ownership])
```
