---
id: health
fill_sections: [overview, schema, manifest, tests, changes]
---

# Health

## Overview
<!-- type: overview lang: markdown -->

Health check primitives for the mamba-native `api` framework:

- `HealthStatus` — three-state enum (`healthy` / `degraded` / `unhealthy`)
- `HealthCheck` — a single named check with a status + optional description
- `HealthManager` — a list of checks with an aggregate status derivation

This slice exercises the codegen pipeline's support for (1) multiple types
in one spec via JSON Schema `definitions: {}`, (2) string-enum codegen
(Rust unit variants + `as_str` + `FromStr`), and (3) struct fields whose
type is another spec-defined type (via `$ref`) or a collection of them
(`Vec<HealthCheck>`).

No App / Router dependency — the health surface is self-contained data
plus aggregation logic.

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  HealthStatus:
    type: string
    enum: [healthy, degraded, unhealthy]
    description: "Aggregate health of a component. Ordered most- to least-healthy."

  HealthCheck:
    type: object
    required: [name, status]
    properties:
      name:
        type: string
        description: "Identifier for the check (e.g. \"db\", \"cache\")."
      status:
        $ref: "#/definitions/HealthStatus"
      description:
        type: [string, null]
        description: "Free-form detail shown alongside the status."
    x-mamba-binding:
      module: mambalibs.http
      symbol: HealthCheck
      extern_fn: health_check_new
      signature: "HealthCheck(name: str, status: HealthStatus, description: str | None = None) -> HealthCheck"
    x-constructor:
      args:
        - { name: name,        mb_type: str,  rust_type: String }
        - { name: status,      mb_type: enum, rust_type: "crate::health::HealthStatus", default: "crate::health::HealthStatus::Healthy" }
        - { name: description, mb_type: str,  rust_type: String, nullable: true }
    x-mamba-attributes:
      - name: name
        doc: "Identifier for the check."
      - name: status
        rust_expr: "self_.status.as_str().to_string()"
        doc: "Current status as canonical string."
      - name: description
        doc: "Optional free-form detail."

  HealthManager:
    type: object
    required: [checks]
    properties:
      checks:
        type: array
        items:
          $ref: "#/definitions/HealthCheck"
        description: "Registered checks, aggregated by `aggregate_status()` (lowest-health wins)."
    x-mamba-binding:
      module: mambalibs.http
      symbol: HealthManager
      extern_fn: health_manager_new
      signature: "HealthManager(checks: list | None = None) -> HealthManager"
    x-constructor:
      args:
        - { name: checks, mb_type: list, rust_type: "Vec<crate::health::HealthCheck>", default: "Vec::new()" }
    x-mamba-attributes:
      # Intentionally omit `checks` getter for now — list-of-struct attribute
      # surfacing needs IntoMbValue<Vec<T>> which lands with mamba PR-2 alongside
      # the dict reader. Track as a follow-up once PR-2 is in.
```

## Manifest
<!-- type: manifest lang: yaml -->

```yaml
dependencies:
  - { name: cclab-mamba-registry, spec: path, path: "../../crates/cclab-mamba-registry" }
  - { name: linkme, spec: workspace }
  - { name: thiserror, spec: workspace }
  - { name: serde, spec: workspace, features: [derive] }
  - { name: serde_json, spec: workspace }
```

## Tests
<!-- type: tests lang: yaml -->

```yaml
imports:
  - "use mambalibs_http::health::{HealthStatus, HealthCheck};"
  - "use std::str::FromStr;"

tests:
  - name: status_serializes_to_canonical_string
    setup: |
      let s = HealthStatus::Degraded;
    assertions:
      - 'assert_eq!(s.as_str(), "degraded")'

  - name: status_round_trips_via_from_str
    setup: |
      let parsed: HealthStatus = FromStr::from_str("unhealthy").unwrap();
    assertions:
      - "assert_eq!(parsed, HealthStatus::Unhealthy)"
      - 'assert_eq!(parsed.as_str(), "unhealthy")'

  - name: status_from_str_rejects_unknown
    setup: |
      let result = HealthStatus::from_str("spicy");
    assertions:
      - "assert!(result.is_err())"

  - name: check_preserves_description_when_provided
    setup: |
      let c = HealthCheck::new(
          "db".to_string(),
          HealthStatus::Healthy,
          Some("primary replica online".to_string()),
      ).unwrap();
    assertions:
      - 'assert_eq!(c.name, "db")'
      - "assert_eq!(c.status, HealthStatus::Healthy)"
      - 'assert_eq!(c.description.as_deref(), Some("primary replica online"))'

  - name: check_accepts_omitted_description
    setup: |
      let c = HealthCheck::new(
          "cache".to_string(),
          HealthStatus::Degraded,
          None,
      ).unwrap();
    assertions:
      - "assert!(c.description.is_none())"
      - "assert_eq!(c.status, HealthStatus::Degraded)"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/mamba/mambalibs/httpkit/src/health.rs
    action: create
    section: schema
    description: |
      Generated from the schema section — HealthStatus enum + HealthCheck
      struct + HealthManager struct + their constructors, FFI shims,
      attribute getters, and register() fns.
  - path: projects/mamba/mambalibs/httpkit/Cargo.toml
    action: modify
    section: manifest
    description: |
      Same dep list as http-exception (shared httpkit crate). The CODEGEN block
      under `[dependencies]` carries the union — last spec to regenerate
      wins, but the contents are identical today.
  - path: projects/mamba/mambalibs/httpkit/tests/health_test.rs
    action: create
    section: tests
    description: |
      Runnable tests exercising HealthStatus's enum contract (as_str,
      FromStr, rejection) and HealthCheck's constructor (description
      Some/None paths).
  # projects/mamba/mambalibs/httpkit/src/lib.rs — auto-wired by apply.rs post-pass
  # (`pub mod health;` + `health::register(r)` in HttpkitModule::register).
  # projects/mamba/mambalibs/httpkit/README.md — auto-aggregated Registered symbols table.
```
