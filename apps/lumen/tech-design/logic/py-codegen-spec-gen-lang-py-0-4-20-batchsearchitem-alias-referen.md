---
id: '1766'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: python-reference-alias-order
entry: generate
nodes:
  generate: { kind: start, label: "lumen spec gen --lang py" }
  canonical: { kind: process, label: "read Lumen canonical OpenAPI components" }
  partition: { kind: process, label: "shared Python emitter partitions concrete models and reference aliases" }
  models: { kind: process, label: "emit concrete Pydantic models, including SearchRequest" }
  aliases: { kind: process, label: "emit dependency-ordered aliases, including BatchSearchItem = SearchRequest" }
  execute: { kind: process, label: "Python compiles and executes generated models.py" }
  done: { kind: terminal, label: "generated package is import-safe" }
edges:
  - { from: generate, to: canonical }
  - { from: canonical, to: partition }
  - { from: partition, to: models }
  - { from: models, to: aliases }
  - { from: aliases, to: execute }
  - { from: execute, to: done }
---
flowchart TD
    generate([lumen spec gen --lang py]) --> canonical[read canonical OpenAPI components]
    canonical --> partition[partition concrete models and reference aliases]
    partition --> models[emit concrete Pydantic models including SearchRequest]
    models --> aliases[emit dependency-ordered aliases including BatchSearchItem = SearchRequest]
    aliases --> execute[compile and execute generated models.py]
    execute --> done([generated package is import-safe])
```
