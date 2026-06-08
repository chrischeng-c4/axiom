---
id: http-exception
fill_sections: [overview, schema, manifest, tests, changes]
---

# HTTPException

## Overview
<!-- type: overview lang: markdown -->

Exception type raised by handlers to return an HTTP error response. First
symbol registered in the mamba-native `api` module. Keeps the familiar
constructor surface — `HTTPException(status_code, detail=None, headers=None)`
— with default `detail` derived from the canonical HTTP status phrase when
omitted, and the status range `[100, 599]` validated at construction.

Fully codegen-produced via the `schema` section's `x-mamba-binding` +
`x-constructor` annotations. No App, Router, or handler-dispatch wiring in
this slice.

## Schema
<!-- type: schema lang: yaml -->

```yaml
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "api://schemas/http-exception",
  "title": "HTTPException",
  "type": "object",
  "required": ["status_code", "detail"],
  "properties": {
    "status_code": {
      "type": "integer",
      "x-rust-type": "u16",
      "minimum": 100,
      "maximum": 599,
      "description": "HTTP status code. Validated at construction; values outside [100,599] raise ValueError."
    },
    "detail": {
      "type": "string",
      "description": "Human-readable error detail. Filled from cclab_mamba_registry::http::status_phrase(status_code) when omitted by the caller (PR-4 canonical table)."
    },
    "headers": {
      "type": ["object", "null"],
      "x-rust-type": "std::collections::HashMap<String, String>",
      "additionalProperties": { "type": "string" },
      "description": "Optional response headers to merge into the error response. Preserved verbatim."
    }
  },
  "x-constructor": {
    "args": [
      { "name": "status_code", "mb_type": "int", "rust_type": "u16", "default": "500" },
      { "name": "detail",      "mb_type": "str", "rust_type": "String", "nullable": true, "default_expr": "cclab_mamba_registry::http::status_phrase(status_code).to_string()" },
      { "name": "headers",     "mb_type": "dict", "rust_type": "std::collections::HashMap<String, String>", "default": "std::collections::HashMap::new()" }
    ],
    "validations": [
      { "field": "status_code", "rule": "range", "min": 100, "max": 599, "message": "status_code must be in [100, 599]" }
    ]
  },
  "x-mamba-binding": {
    "module": "mambalibs.http",
    "symbol": "HTTPException",
    "extern_fn": "http_exception_new",
    "signature": "HTTPException(status_code: int, detail: str | None = None, headers: dict | None = None) -> HTTPException"
  },
  "x-mamba-attributes": [
    { "name": "status_code", "rust_expr": "self_.status_code as i64",
      "doc": "HTTP status code. Validated to be in [100, 599] at construction." },
    { "name": "detail",
      "doc": "Human-readable error detail. Filled from http_status_phrase(status_code) when omitted by the caller." },
    { "name": "headers",
      "doc": "Optional response headers to merge into the error response." }
  ]
}
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
  - "use mambalibs_http::http_exception::HTTPException;"
  - "use std::collections::HashMap;"

tests:
  - name: preserves_explicit_detail
    setup: |
      let exc = HTTPException::new(
          404, Some("not in cache".into()), HashMap::new(),
      ).unwrap();
    assertions:
      - "assert_eq!(exc.status_code, 404)"
      - 'assert_eq!(exc.detail, "not in cache")'

  - name: omitted_detail_fills_from_status_phrase
    setup: |
      let exc = HTTPException::new(500, None, HashMap::new()).unwrap();
    assertions:
      - "assert_eq!(exc.status_code, 500)"
      - 'assert_eq!(exc.detail, "Internal Server Error")'

  - name: status_code_out_of_range_low_returns_err
    setup: |
      let result = HTTPException::new(50, None, HashMap::new());
    assertions:
      - "assert!(result.is_err())"

  - name: status_code_out_of_range_high_returns_err
    setup: |
      let result = HTTPException::new(700, None, HashMap::new());
    assertions:
      - "assert!(result.is_err())"

  - name: headers_round_trip_when_provided
    setup: |
      let mut h = HashMap::new();
      h.insert("x-retry-after".to_string(), "30".to_string());
      let exc = HTTPException::new(429, None, h).unwrap();
    assertions:
      - 'assert_eq!(exc.headers.as_ref().unwrap().get("x-retry-after").map(|s| s.as_str()), Some("30"))'
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/mamba/mambalibs/httpkit/src/http_exception.rs
    action: create
    section: schema
    description: |
      Generated from the schema section — struct + constructor (with range
      validation + computed default for detail) + extern "C" FFI shim +
      register() fn + http_status_phrase helper.
  - path: projects/mamba/mambalibs/httpkit/Cargo.toml
    action: modify
    section: manifest
    description: |
      Dependencies for the httpkit crate. Managed block lives under
      `[dependencies]`; anything outside the CODEGEN markers is hand-written.
  - path: projects/mamba/mambalibs/httpkit/tests/http_exception_test.rs
    action: create
    section: tests
    description: |
      Runnable `#[test]` cases for the HTTPException contract — preserved
      detail, computed default, range validation, headers round-trip.
  # projects/mamba/mambalibs/httpkit/src/lib.rs — mamba-mod-decls + mamba-register-body CODEGEN
  # blocks are aggregated by apply.rs's auto_wire_mamba_lib post-pass.
  # projects/mamba/mambalibs/httpkit/README.md — Registered Symbols table is aggregated by
  # apply.rs's auto_wire_readme_symbols post-pass from every spec's
  # x-mamba-binding annotation.
```
