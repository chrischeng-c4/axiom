---
id: sdd-services-platform-sync-config
fill_sections: [overview, schema, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: client-boundary-model
    claim: client-boundary-model
    coverage: full
    rationale: "Config and platform TDs define AW Core client boundary behavior."
---

# Platform Sync Config Types

## Overview
<!-- type: overview lang: markdown -->

Platform sync configuration types in
`projects/agentic-workflow/src/services/platform_sync/config.rs`. Seven serde shapes:

- `PlatformConfig` — top-level (platform_type via `serde(rename = "type")`,
  repo, optional self-hosted host, optional auth/labels/title, legacy
  envfile/envfield).
- `AuthConfig` — optional envfile, envfield.
- `LabelConfig` — auto_create bool, optional proposal/spec/status/scope labels.
- `StatusLabels` — optional draft/review/approved/implementing/done.
- `ScopeConfig` — enabled bool, optional pattern, optional auto_detect.
- `ScopeAutoDetect` — optional path_regex.
- `TitleConfig` — optional proposal/spec format strings.

Codegen replaces all seven type declarations. Companion source templates own
module documentation, non-serde imports, loading, authentication lookup, title
and label helpers, env parsing, and regression tests. The generated schema block
continues to own the serde import.

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  PlatformConfig:
    type: object
    required: [platform_type, repo, host, auth, labels, title, envfile, envfield]
    description: Platform configuration from .aw/config.toml.
    properties:
      platform_type:
        type: string
        x-serde-rename: "type"
        description: "Platform type: github or gitlab."
      repo:
        type: string
        description: "Repository in owner/repo format."
      host:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        description: "Optional self-hosted base host (e.g. 'gitlab.example.com'). None = use platform default."
      auth:
        type: object
        x-rust-type: "Option<AuthConfig>"
        x-serde-default: true
        description: "Authentication configuration."
      labels:
        type: object
        x-rust-type: "Option<LabelConfig>"
        x-serde-default: true
        description: "Label configuration."
      title:
        type: object
        x-rust-type: "Option<TitleConfig>"
        x-serde-default: true
        description: "Title format configuration."
      envfile:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        description: "Legacy envfile field."
      envfield:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        description: "Legacy envfield field."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  AuthConfig:
    type: object
    required: [envfile, envfield]
    description: Authentication configuration.
    properties:
      envfile:
        type: string
        x-rust-type: "Option<String>"
        description: "Path to .env file."
      envfield:
        type: string
        x-rust-type: "Option<String>"
        description: "Field name in .env file."
    x-rust-struct:
      derive: [Debug, Clone, Default, Serialize, Deserialize]

  LabelConfig:
    type: object
    required: [auto_create, proposal, spec, status, scope]
    description: Label configuration.
    properties:
      auto_create:
        type: boolean
        x-serde-default: true
        description: "Auto-create labels if they don't exist."
      proposal:
        type: string
        x-rust-type: "Option<String>"
        description: "Label for proposal issues."
      spec:
        type: string
        x-rust-type: "Option<String>"
        description: "Label for spec issues."
      status:
        type: object
        x-rust-type: "Option<StatusLabels>"
        x-serde-default: true
        description: "Status labels."
      scope:
        type: object
        x-rust-type: "Option<ScopeConfig>"
        x-serde-default: true
        description: "Scope label configuration."
    x-rust-struct:
      derive: [Debug, Clone, Default, Serialize, Deserialize]

  StatusLabels:
    type: object
    required: [draft, review, approved, implementing, done]
    description: Status labels mapping.
    properties:
      draft:
        type: string
        x-rust-type: "Option<String>"
        description: "Draft status label."
      review:
        type: string
        x-rust-type: "Option<String>"
        description: "Review status label."
      approved:
        type: string
        x-rust-type: "Option<String>"
        description: "Approved status label."
      implementing:
        type: string
        x-rust-type: "Option<String>"
        description: "Implementing status label."
      done:
        type: string
        x-rust-type: "Option<String>"
        description: "Done status label."
    x-rust-struct:
      derive: [Debug, Clone, Default, Serialize, Deserialize]

  ScopeConfig:
    type: object
    required: [enabled, pattern, auto_detect]
    description: Scope label configuration (e.g., crate:xxx).
    properties:
      enabled:
        type: boolean
        description: "Enable scope labels."
      pattern:
        type: string
        x-rust-type: "Option<String>"
        description: "Label pattern (e.g., crate:{scope})."
      auto_detect:
        type: object
        x-rust-type: "Option<ScopeAutoDetect>"
        x-serde-default: true
        description: "Auto-detection configuration."
    x-rust-struct:
      derive: [Debug, Clone, Default, Serialize, Deserialize]

  ScopeAutoDetect:
    type: object
    required: [path_regex]
    description: Auto-detection for scope labels.
    properties:
      path_regex:
        type: string
        x-rust-type: "Option<String>"
        description: "Regex to extract scope from path."
    x-rust-struct:
      derive: [Debug, Clone, Default, Serialize, Deserialize]

  TitleConfig:
    type: object
    required: [proposal, spec]
    description: Title format configuration.
    properties:
      proposal:
        type: string
        x-rust-type: "Option<String>"
        description: "Format for proposal issues."
      spec:
        type: string
        x-rust-type: "Option<String>"
        description: "Format for spec issues."
    x-rust-struct:
      derive: [Debug, Clone, Default, Serialize, Deserialize]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/services/platform_sync/config.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - PlatformConfig
      - AuthConfig
      - LabelConfig
      - StatusLabels
      - ScopeConfig
      - ScopeAutoDetect
      - TitleConfig
    description: |
      Codegen replaces all seven type declarations.
```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [overview] Seven nested config structs.
- [schema] x-serde-rename for `type`; nested optional fields with x-serde-default.
- [changes] All seven in `replaces`.
