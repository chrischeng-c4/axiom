---
id: sdd-generate-generators-mod
fill_sections: [overview, schema, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# GeneratorArgs Type

## Overview
<!-- type: overview lang: markdown -->

Generator arguments type in `projects/agentic-workflow/src/generators/mod.rs`. One shape:

- `GeneratorArgs` — `section_type: SectionType`,
  `sdd_id: Option<String>`, `sdd_refs: Vec<String>`. Derives `[Debug, Clone]`.

Codegen replaces the struct declaration. Companion source templates own the
module preamble/imports and the runtime helper/dispatch/test block that
previously lived in managed HANDWRITE gaps.

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  GeneratorArgs:
    type: object
    required: [section_type, sdd_id, sdd_refs]
    description: Arguments for invoking a structural generator.
    properties:
      section_type:
        type: string
        x-rust-type: "SectionType"
        description: "Target section type."
      sdd_id:
        type: string
        x-rust-type: "Option<String>"
        description: "Change ID providing context (from --sdd-id)."
      sdd_refs:
        type: array
        items: { type: string }
        x-rust-type: "Vec<String>"
        description: "Related spec references (from --sdd-refs)."
    x-rust-struct:
      derive: [Debug, Clone]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generators/mod.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - GeneratorArgs
    description: |
      Codegen replaces the struct declaration only.
```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->

**Verdict:** approved

- [overview] Single struct with foreign-type fields.
- [schema] Standard pattern.
- [changes] Standard split.
