---
id: projects-jet-logic-jet-build-lib-d-ts-class-member-signature-reduction-remaining-ex-md
fill_sections: [logic, changes]
capability_refs:
  - id: library-build-publishing
    role: primary
    gap: library-publishing-readiness
    claim: type-declaration-emission
    coverage: partial
    rationale: "Reducing exported class members to ambient signatures (and covering remaining export shapes) completes correct .d.ts emission for library-build-publishing."
---

# jet build --lib .d.ts: Class-Member Reduction + Remaining Export Shapes

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: jet-dts-class-reduction
entry: walk
nodes:
  walk:   { kind: start,    label: "dts emit reaches an exported class decl" }
  member: { kind: process,  label: "for each class member" }
  vis:    { kind: decision, label: "member visibility/kind" }
  drop:   { kind: process,  label: "private / # member -> drop" }
  method: { kind: process,  label: "method -> signature only (drop body)" }
  field:  { kind: process,  label: "public field -> name: type" }
  more:   { kind: decision, label: "more members?" }
  emit:   { kind: process,  label: "emit export declare class { reduced members }" }
  done:   { kind: terminal, label: "ambient class declaration" }
edges:
  - { from: walk,   to: member }
  - { from: member, to: vis }
  - { from: vis,    to: drop,   label: "private" }
  - { from: vis,    to: method, label: "method" }
  - { from: vis,    to: field,  label: "public-field" }
  - { from: drop,   to: more }
  - { from: method, to: more }
  - { from: field,  to: more }
  - { from: more,   to: member, label: "yes" }
  - { from: more,   to: emit,   label: "no" }
  - { from: emit,   to: done }
---
flowchart TD
    walk([exported class decl]) --> member[for each member]
    member --> vis{visibility/kind}
    vis -->|private| drop[drop]
    vis -->|method| method[signature only]
    vis -->|public-field| field[name: type]
    drop --> more{more members?}
    method --> more
    field --> more
    more -->|yes| member
    more -->|no| emit[emit export declare class]
    emit --> done([ambient class declaration])
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
      Reduce exported class declarations to ambient form: methods -> signatures
      (drop bodies), public fields -> name: type, drop private/#-private members,
      keep constructor signature. Cover the remaining export shapes flagged TODO
      (annotated complex default export, re-export passthrough) where statically
      determinable.
    impl_mode: hand-written
  - path: "projects/jet/tests/build/library_dts.rs"
    action: modify
    section: unit-test
    description: |
      Tests: exported class emits export declare class with method signatures,
      public field types, private members dropped; previously-deferred export
      shapes emit valid declarations; existing dts tests pass.
    impl_mode: hand-written
```

# Reviews

### Review 1
**Verdict:** approved

- [logic] Contract logic (jet-dts-class-reduction) complete + deterministic: walk class -> per-member visibility decision (private/method/public-field, all labeled) -> more loop -> emit ambient declare class -> terminal. All nodes reachable; terminal real. Extends A2.
