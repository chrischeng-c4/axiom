---
id: sdd-generate-generators-rpc-api-types
fill_sections: [overview, schema, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# RpcApiGenerator

## Overview
<!-- type: overview lang: markdown -->

Single RpcApiGenerator unit struct. A companion source template owns the module
preamble, `Generator` impl, and regression tests that previously lived in a
managed HANDWRITE gap.

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  RpcApiGenerator:
    type: object
    description: RpcApiGenerator unit struct.
    properties: {}
    x-rust-struct:
      derive: []
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generators/rpc_api.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - RpcApiGenerator
    description: Codegen replaces RpcApiGenerator.
```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->

**Verdict:** approved

- ok.
