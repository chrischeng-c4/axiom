---
id: sdd-services-platform-sync-github
fill_sections: [overview, schema, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: client-boundary-model
    claim: client-boundary-model
    coverage: full
    rationale: "Service interfaces expose AW Core project, issue, and platform boundary behavior to clients."
---

# GitHubProvider Type

## Overview
<!-- type: overview lang: markdown -->

GitHub platform-sync provider in
`projects/agentic-workflow/src/services/platform_sync/github.rs`. One shape:

- `GitHubProvider` — `config: PlatformConfig`, `token: Option<String>`,
  no derives. Both fields private.

Codegen replaces the struct declaration. Companion source templates own module
documentation, imports, provider construction, auth checks, sync orchestration,
API and CLI upserts, error sanitization, URL parsing, and regression tests.

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  GitHubProvider:
    type: object
    required: [config, token]
    description: |
      GitHub platform-sync provider.
    properties:
      config:
        type: object
        x-rust-type: "PlatformConfig"
        x-rust-visibility: private
        description: "Platform configuration."
      token:
        type: string
        x-rust-type: "Option<String>"
        x-rust-visibility: private
        description: "Optional auth token."
    x-rust-struct:
      derive: []
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/services/platform_sync/github.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - GitHubProvider
    description: |
      Codegen replaces the struct declaration only.
```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [overview] Single struct with no derives + 2 private fields.
- [schema] Both fields via x-rust-type with private visibility.
- [changes] Standard split.
