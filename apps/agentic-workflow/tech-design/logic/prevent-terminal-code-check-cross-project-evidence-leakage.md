---
id: '1679'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: terminal-code-check-exact-spec-evidence-scope
entry: code_check
nodes:
  code_check: { kind: start, label: "aw td code-check <slug>" }
  resolve: { kind: process, label: "resolve exact Issue.implements TD spec paths" }
  target_evidence: { kind: process, label: "collect hand-written create/modify paths only from resolved specs" }
  unrelated_spec: { kind: process, label: "unrelated project TD with incomplete paths" }
  complete: { kind: decision, label: "every target path has a Td-Init-to-HEAD diff?" }
  refuse: { kind: terminal, label: "error names only target paths" }
  close: { kind: terminal, label: "complete terminal lifecycle" }
edges:
  - { from: code_check, to: resolve }
  - { from: resolve, to: target_evidence }
  - { from: unrelated_spec, to: target_evidence, label: "must not contribute" }
  - { from: target_evidence, to: complete }
  - { from: complete, to: refuse, label: "no" }
  - { from: complete, to: close, label: "yes" }
---
flowchart TD
  code_check([aw td code-check]) --> resolve[resolve Issue.implements]
  unrelated_spec[unrelated Mamba TD] -. ignored .-> target_evidence[target spec evidence only]
  resolve --> target_evidence
  target_evidence --> complete{every target diff present?}
  complete -->|no| refuse([error: target paths only])
  complete -->|yes| close([terminal lifecycle closes])
```
