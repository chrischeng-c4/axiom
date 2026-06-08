---
id: sdd-generate-generators-sequence-plus-gen
fill_sections: [overview, schema, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# SequencePlusGenerator Type

## Overview
<!-- type: overview lang: markdown -->

Unit-struct generator in
`projects/agentic-workflow/src/generate/generators/sequence_plus_gen.rs`. One shape:

- `SequencePlusGenerator` — unit struct with no derives.

Codegen replaces only the unit struct declaration. Module imports,
the `impl SequencePlusGenerator { new, ... }` block, the
`impl Generator for SequencePlusGenerator` (or similar trait impls),
and tests are owned by sibling source templates that replace the legacy
HANDWRITE gaps.

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  SequencePlusGenerator:
    type: object
    required: []
    properties: {}
    description: SequencePlus generator (unit struct).
    x-rust-struct:
      derive: []
      unit: true
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/generators/sequence_plus_gen.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - SequencePlusGenerator
    description: |
      Codegen replaces the unit struct declaration only.
```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [overview] Single unit struct.
- [schema] Standard unit-struct shape.
- [changes] Standard split.
