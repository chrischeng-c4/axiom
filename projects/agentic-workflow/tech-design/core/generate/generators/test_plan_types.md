---
id: sdd-generate-generators-test-plan-types
fill_sections: [overview, schema, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# TestPlanGenerator

## Overview
<!-- type: overview lang: markdown -->

Single TestPlanGenerator unit struct. A companion source template owns the
module preamble, `Generator` impl, and regression tests that previously lived in
a managed HANDWRITE gap.

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  TestPlanGenerator:
    type: object
    description: TestPlanGenerator unit struct.
    properties: {}
    x-rust-struct:
      derive: []
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generators/test_plan.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - TestPlanGenerator
    description: Codegen replaces TestPlanGenerator.
```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->

**Verdict:** approved

- ok.
