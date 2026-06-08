---
id: sdd-issues-backends-gitlab
fill_sections: [overview, schema, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: client-boundary-model
    claim: client-boundary-model
    coverage: full
    rationale: "Issue backend interfaces implement the AW Core client boundary for projecting workflow state to configured issue platforms."
---

# GitLabBackend Type

## Overview
<!-- type: overview lang: markdown -->

Issue backend struct in `projects/agentic-workflow/src/issues/backends/gitlab.rs`.
One shape:

- `GitLabBackend` — single private `repo: Option<String>` field with
  no derives.

Codegen replaces the struct declaration. Companion source templates own module
documentation/imports, GitLab CLI plumbing, `IssueBackend` behavior, parsing,
subprocess execution, and regression tests.

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  GitLabBackend:
    type: object
    required: [repo]
    description: |
      Issue backend that calls the `glab` CLI.
    properties:
      repo:
        type: string
        x-rust-type: "Option<String>"
        x-rust-visibility: private
        description: "Optional `owner/repo` slug. None = use CWD-detected repo."
    x-rust-struct:
      derive: []
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/issues/backends/gitlab.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - GitLabBackend
    description: |
      Codegen replaces the struct declaration only.
```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [overview] Single struct with no derives + private field.
- [schema] Option<String> via x-rust-type with private visibility.
- [changes] Standard split.
