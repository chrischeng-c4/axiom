---
id: create-user-request
fill_sections: [overview, schema, manifest, tests, changes]
---

# CreateUserRequest — pydantic-like user payload model

## Overview
<!-- type: overview lang: markdown -->

Demonstration of the `pydantic`-equivalent BaseModel pattern for the
mamba-native `mambalibs.http` framework. A handler-side request payload that a
user-land app would declare to validate inbound JSON:

```python
# Python consumer (post-mamba-binding):
from httpkit_demo import CreateUserRequest

@app.post("/users")
def create_user(payload: CreateUserRequest) -> Response:
    # payload.name, payload.email, payload.age all typed + validated.
    ...
```

The spec below is functionally the same shape the framework's own
`HTTPException` / `Cookie` / `Request` / `Response` specs use — there
is no separate "BaseModel" trait. Every generated struct in this
codegen pipeline IS a validated data model:

- JSON Schema describes the field shape
- `x-constructor.validations` enforces pydantic-style rules
  (`range`, `min_length`, `max_length`, `expr`)
- `x-mamba-binding` exposes it to Python as a constructible class
- `x-mamba-attributes` emits per-field getters

Framework types and user types follow the same authoring surface. The
only thing that makes this spec "user-facing" is that it lives under
`projects/httpkit-demo/` instead of `projects/mamba/mambalibs/httpkit/`.

## Schema
<!-- type: schema lang: yaml -->

```yaml
title: CreateUserRequest
type: object
required: [name, email, age]
description: "Request body for `POST /users`. Validated pydantic-like."
properties:
  name:
    type: string
    description: "Display name. 1–64 chars."
  email:
    type: string
    description: "User email. Must contain '@'. No further RFC 5322 check in this demo."
  age:
    type: integer
    x-rust-type: i64
    minimum: 0
    maximum: 150
    description: "Age in years. Sanity-bounded to avoid obvious bogus inputs."
x-mamba-binding:
  module: mambalibs.httpkit_demo
  symbol: CreateUserRequest
  extern_fn: create_user_request_new
  signature: "CreateUserRequest(name: str, email: str, age: int) -> CreateUserRequest"
x-constructor:
  args:
    - { name: name,  mb_type: str, rust_type: String }
    - { name: email, mb_type: str, rust_type: String }
    - { name: age,   mb_type: int, rust_type: i64, default: "0" }
  validations:
    - { field: name,  rule: min_length, min: 1,  message: "name must be non-empty" }
    - { field: name,  rule: max_length, max: 64, message: "name too long (max 64 chars)" }
    - { field: email, rule: expr, expr: "email.contains('@')", message: "email must contain '@'" }
    - { field: age,   rule: range, min: 0, max: 150 }
x-mamba-attributes:
  - name: name
  - name: email
  - name: age
    rust_expr: "self_.age"
```

## Manifest
<!-- type: manifest lang: yaml -->

```yaml
dependencies:
  - { name: mambalibs-http,        spec: path, path: "../mamba/mambalibs/httpkit" }
  - { name: cclab-mamba-registry, spec: path, path: "../../crates/cclab-mamba-registry" }
  - { name: linkme,               spec: workspace }
  - { name: serde,                spec: workspace, features: [derive] }
  - { name: serde_json,           spec: workspace }
```

## Tests
<!-- type: tests lang: yaml -->

```yaml
imports:
  - "use httpkit_demo::create_user_request::CreateUserRequest;"

tests:
  - name: valid_payload_constructs
    setup: |
      let r = CreateUserRequest::new(
          "alice".to_string(),
          "alice@example.com".to_string(),
          30,
      ).unwrap();
    assertions:
      - 'assert_eq!(r.name, "alice")'
      - 'assert_eq!(r.email, "alice@example.com")'
      - "assert_eq!(r.age, 30)"

  - name: empty_name_rejected_by_min_length
    setup: |
      let result = CreateUserRequest::new(
          "".to_string(),
          "alice@example.com".to_string(),
          30,
      );
    assertions:
      - "assert!(result.is_err())"
      - 'assert!(result.unwrap_err().contains("non-empty"))'

  - name: long_name_rejected_by_max_length
    setup: |
      let long = "a".repeat(65);
      let result = CreateUserRequest::new(
          long,
          "alice@example.com".to_string(),
          30,
      );
    assertions:
      - "assert!(result.is_err())"
      - 'assert!(result.unwrap_err().contains("too long"))'

  - name: email_without_at_rejected_by_expr_rule
    setup: |
      let result = CreateUserRequest::new(
          "alice".to_string(),
          "alice-at-example.com".to_string(),
          30,
      );
    assertions:
      - "assert!(result.is_err())"
      - 'assert!(result.unwrap_err().contains("@"))'

  - name: out_of_range_age_rejected
    setup: |
      let result = CreateUserRequest::new(
          "alice".to_string(),
          "alice@example.com".to_string(),
          999,
      );
    assertions:
      - "assert!(result.is_err())"

  - name: serde_json_roundtrip
    setup: |
      let r = CreateUserRequest::new(
          "alice".to_string(),
          "alice@example.com".to_string(),
          30,
      ).unwrap();
      let json = serde_json::to_string(&r).unwrap();
      let parsed: CreateUserRequest = serde_json::from_str(&json).unwrap();
    assertions:
      - "assert_eq!(r, parsed)"
      - 'assert!(json.contains("alice@example.com"))'
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/httpkit-demo/src/create_user_request.rs
    action: create
    section: schema
    description: |
      Generated from schema — CreateUserRequest struct + constructor (with
      min_length + max_length + expr + range validators) + extern "C" FFI
      shim + register_create_user_request + 3 attribute getters.
  - path: projects/httpkit-demo/Cargo.toml
    action: modify
    section: manifest
    description: |
      Dep list for the demo crate. Depends on `mambalibs-http` (path), mamba
      registry, linkme, serde/serde_json.
  - path: projects/httpkit-demo/tests/create_user_request_test.rs
    action: create
    section: tests
    description: |
      Runnable tests exercising the four validator flavors + happy path
      + serde round-trip. Proves a user-land type gets the same
      validation + serialization surface framework types do.
  # projects/httpkit-demo/src/lib.rs — auto-wired (`pub mod create_user_request;`).
  # projects/httpkit-demo/README.md — Registered symbols table gets
  # `CreateUserRequest`.
```
