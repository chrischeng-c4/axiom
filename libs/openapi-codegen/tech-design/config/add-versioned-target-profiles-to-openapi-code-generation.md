---
id: "1569"
summary: (fill)
fill_sections: [logic]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
flowchart LR
  O[GenOptions] --> V[Validate language target profile]
  V --> P[OpenAPI semantic IR]
  P --> E{Language emitter}
  E --> PY[Python artifact profile]
  E --> TS[TypeScript artifact profile]
  E --> RS[Rust artifact profile]
  PY --> A[Deterministic GeneratedOutput]
  TS --> A
  RS --> A
```
