---
id: '2197'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: workbench-context-provenance-applicability
entry: item
nodes:
  item: { kind: start, label: "provider-neutral context item" }
  classify: { kind: process, label: "classify extracted, inferred, or ambiguous" }
  resolve: { kind: process, label: "resolve confined file and optional span" }
  valid: { kind: decision, label: "canonical source exists and span is valid?" }
  canonical: { kind: process, label: "emit canonical source navigation" }
  unavailable: { kind: process, label: "emit visible non-authoritative state" }
  label: { kind: process, label: "disclose provider and derivation inputs" }
  done: { kind: terminal, label: "read-only provenance view" }
edges:
  - { from: item, to: classify }
  - { from: classify, to: resolve }
  - { from: resolve, to: valid }
  - { from: valid, to: canonical, label: "yes" }
  - { from: valid, to: unavailable, label: "no" }
  - { from: canonical, to: label }
  - { from: unavailable, to: label }
  - { from: label, to: done }
---
flowchart LR
    item([Context item]) --> classify[Classify]
    classify --> resolve[Resolve source/span]
    resolve --> valid{Canonical?}
    valid -->|Yes| canonical[Source navigation]
    valid -->|No| unavailable[Non-authoritative state]
    canonical --> label[Provider and inputs]
    unavailable --> label
    label --> done([Provenance view])
```

Create a provider-neutral provenance module below the renderer registry. It models a confined repository-relative file, optional one-based source span, provider identity, and extracted/inferred/ambiguous classification without importing any Graphify, wiki, AW, PTY, or cwd types. Graph-style EXTRACTED/INFERRED/AMBIGUOUS trust labels inform the closed classification vocabulary, while compiled wiki pages remain derived views over immutable canonical inputs.

Resolution canonicalizes the selected root and requested source, rejects traversal and symlink escape, validates ordered non-zero spans, and distinguishes canonical, missing, and invalid states. Extracted items link only when their direct source resolves. Inferred items retain every input location and always carry a visible derived label; unavailable inputs are disclosed rather than replaced with fabricated links.

The module is a pure read-only data and resolution boundary. It exposes no repository write, AW transition, provider invocation, or verification mutation; repository bytes and declared executable gates remain authoritative outside the provenance view.
