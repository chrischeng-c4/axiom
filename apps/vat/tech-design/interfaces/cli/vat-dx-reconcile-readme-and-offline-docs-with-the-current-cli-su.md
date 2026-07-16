---
id: '1817'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: vat-dx-readme-surface-logic
entry: start
nodes:
  start: { kind: start, label: "VAT documentation reconciliation begins" }
  inspect_cli: { kind: process, label: "derive agent-facing top-level command inventory from the real Cmd clap variants and vat --help" }
  compare_readme: { kind: decision, label: "README inventory and product boundary match the shipped command surface" }
  update_docs: { kind: process, label: "document build, compose, opt-in docker shim, and Apple Container k8s with their supported boundaries" }
  update_llm: { kind: process, label: "align the offline guide with the same command and boundary vocabulary" }
  regression: { kind: process, label: "test the built binary help output against the documented command inventory" }
  done: { kind: terminal, label: "docs teach only supported VAT paths and drift fails deterministically" }
edges:
  - { from: start, to: inspect_cli }
  - { from: inspect_cli, to: compare_readme }
  - { from: compare_readme, to: update_docs, label: "drift" }
  - { from: compare_readme, to: regression, label: "aligned" }
  - { from: update_docs, to: update_llm }
  - { from: update_llm, to: regression }
  - { from: regression, to: done }
---
```

The canonical agent-facing command inventory is the executable VAT clap surface. README and `vat llm` explain that `vat build` and bounded `vat compose` use Apple Container, `vat docker` is an opt-in limited command shim, and `vat k8s` supports bounded ephemeral Apple-Container-backed local Kubernetes flows. They explicitly exclude Docker Engine/API compatibility, unbounded generic Compose support, and persistent Kubernetes lifecycle claims. A built-binary regression test reads `vat --help` and asserts that every documented agent-facing command is present, so a future CLI or documentation edit cannot silently create an obsolete onboarding path.
