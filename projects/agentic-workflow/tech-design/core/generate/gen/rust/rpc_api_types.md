---
id: sdd-gen-rust-rpc_api-types
fill_sections: [overview, schema, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# RpcApiGenOutput

## Overview
<!-- type: overview lang: markdown -->

`RpcApiGenOutput` is generated in the canonical Rust codegen module at
`projects/agentic-workflow/src/generate/gen/rust/rpc_api.rs` and the legacy mirror at
`projects/agentic-workflow/src/gen/rust/rpc_api.rs`.

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  RpcApiGenOutput:
    type: object
    description: Output from RPC-API code generation.
    properties:
      code:
        type: string
        description: The generated async fn signatures with SPEC-REF body markers.
      spec_refs:
        type: array
        items: { type: string }
        description: SPEC-REF entries emitted.
    required: [code, spec_refs]
    x-rust-struct:
      derive: [Debug, Clone]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/gen/rust/rpc_api.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - RpcApiGenOutput
    description: Codegen replaces RpcApiGenOutput.
  - path: projects/agentic-workflow/src/gen/rust/rpc_api.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - RpcApiGenOutput
    description: Codegen replaces the legacy mirror of RpcApiGenOutput.
  - path: projects/agentic-workflow/src/generate/gen/rust/rpc_api.rs
    action: modify
    section: schema
    impl_mode: hand-written
    description: Module preamble, free fns, helpers, tests.
```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- ok.
