---
id: lumen-spec-gen-client
summary: >
  Add a `gen` subcommand under `lumen spec`: feed lumen's own offline OpenAPI
  document (openapi_json()) into the cclab-openapi-codegen crate and write a typed
  client (TypeScript / Python / Rust, by --lang) into --out. `lumen spec` with no
  subcommand keeps printing the spec exactly as today.
capability_refs:
  - id: "competitor-feature-parity"
    role: primary
    claim: "schema-and-metadata-breadth"
    coverage: partial
    rationale: >
      Turnkey client generation from lumen's own self-complete OpenAPI, so
      adopters need no external codegen step.
fill_sections: [logic, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: spec-gen-dispatch
entry: start
nodes:
  start:    { kind: start,    label: "lumen spec [gen]" }
  has_gen:  { kind: decision, label: "gen subcommand?" }
  print:    { kind: terminal, label: "print spec (unchanged)" }
  spec:     { kind: process,  label: "openapi_json() (lumen's own spec)" }
  gen:      { kind: process,  label: "cclab_openapi_codegen::generate(spec, GenOptions{lang})" }
  write:    { kind: process,  label: "create --out; write each GeneratedFile" }
  done:     { kind: terminal, label: "client written to --out" }
edges:
  - { from: start,   to: has_gen }
  - { from: has_gen, to: print, label: "no" }
  - { from: has_gen, to: spec,  label: "yes" }
  - { from: spec,    to: gen }
  - { from: gen,     to: write }
  - { from: write,   to: done }
---
flowchart TD
    start([lumen spec]) --> has_gen{gen subcommand?}
    has_gen -->|no| print([print spec, unchanged])
    has_gen -->|yes| spec[openapi_json]
    spec --> gen[openapi-codegen generate]
    gen --> write[create out; write files]
    write --> done([client written])
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: spec-gen-verification
requirements:
  gen_py_writes_client:
    id: R1
    text: "spec gen --lang py --out <dir> writes models.py/client.py/__init__.py with a typed pydantic model"
    kind: functional
    risk: high
    verify: test
  gen_lang_selects_emitter:
    id: R2
    text: "--lang ts and --lang rust write the TypeScript and Rust file sets respectively"
    kind: functional
    risk: medium
    verify: test
  plain_spec_unchanged:
    id: R3
    text: "lumen spec with no subcommand still prints the OpenAPI document unchanged"
    kind: functional
    risk: medium
    verify: test
elements:
  test_gen_py_writes_client:
    kind: test
    type: "rs/#[test]"
  test_gen_lang_selects_emitter:
    kind: test
    type: "rs/#[test]"
  test_plain_spec_unchanged:
    kind: test
    type: "rs/#[test]"
relations:
  - { from: test_gen_py_writes_client,     verifies: gen_py_writes_client }
  - { from: test_gen_lang_selects_emitter, verifies: gen_lang_selects_emitter }
  - { from: test_plain_spec_unchanged,     verifies: plain_spec_unchanged }
---
requirementDiagram
    requirement R1 {
      id: R1
      text: "gen --lang py writes a typed client"
      risk: high
      verifymethod: test
    }
    requirement R2 {
      id: R2
      text: "--lang selects the emitter"
      risk: medium
      verifymethod: test
    }
    requirement R3 {
      id: R3
      text: "plain spec unchanged"
      risk: medium
      verifymethod: test
    }
    element test_gen_py_writes_client {
      type: "rs/#[test]"
    }
    test_gen_py_writes_client - verifies -> R1
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/lumen/src/bin/lumen.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Wire `lumen spec gen` language selection and offline typed-client generation dispatch."
  - path: projects/lumen/tests/spec_gen_e2e.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    description: "Exercise Python, TypeScript, Rust emitter selection and plain `lumen spec` output."
```
