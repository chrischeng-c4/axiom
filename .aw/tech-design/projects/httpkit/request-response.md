---
id: request-response
fill_sections: [overview, schema, manifest, tests, changes]
---

# Request / Response

## Overview
<!-- type: overview lang: markdown -->

> **Status (Phase 1a, 2026-05):** the canonical, unified wire-level types
> now live at `mambalibs_http::http::{Request, Response, Cookie, HttpMethod,
> HttpStatus, RequestBody, Auth}` (hand-written Rust, server + client share
> one shape). The pydantic-style codegen surface described below is **kept
> as a compatibility shim** for the existing `mambalibs-http-binding` FFI exports
> (`Cookie`, `Request`, `Response` in the `mambalibs.http` namespace) and
> will be retired once Phase 1b (DI engine) and Phase 1c
> (`httpkit.BaseModel` Python helper, mamba-native) land. Rationale: with
> mamba being strongly typed and capable of running Python validators at
> native speed, the JSON-Schema → Rust struct codegen no longer earns its
> keep — payload validation belongs in user Python compiled by mamba, not
> in a Rust-side mirror.

Core HTTP request/response data model for the native httpkit framework:

- `Cookie` — one cookie value with web-framework attributes (path, domain,
  secure, http_only, max_age). Used on both the request and response sides.
- `Request` — inbound HTTP request: method, path, query_params, headers,
  cookies, body bytes, and path_params captured by the router.
- `Response` — outbound HTTP response: status_code, body bytes, headers,
  cookies, and media_type.

All three types are "pydantic-like" validated data models under this
codegen pipeline — every field is declared in JSON Schema with typed
defaults + validators (`x-constructor.validations`), and every public
surface goes through `x-mamba-binding` for Python interop. The same
pattern applies to user-defined payload models (`class UserRequest(BaseModel)`
in Python becomes a TD spec with `x-constructor` + `x-mamba-attributes`).

Follow-ups blocked on mamba PR-2 (dict / list `FromMbValue`):

- Real header + cookie round-trip from a Python-side caller (today the
  FFI shim uses the literal defaults because `mb_type: dict` / `list`
  have no primitive reader yet).
- `IntoMbValue<Vec<Cookie>>` for the cookies attribute getter.

Follow-ups blocked on mamba PR-5 (getattr dispatch):

- Python-side `request.headers["x-user"]` access — attributes are
  registered today but not dispatched.

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  Cookie:
    type: object
    # `secure`, `http_only` are required-with-default — they're always present
    # on a constructed Cookie, just default to false. Keeping them out of
    # `required` would make the struct field `Option<bool>`, which is clumsy.
    required: [name, value, secure, http_only]
    properties:
      name:
        type: string
        description: "Cookie name. Validated non-empty at construction."
      value:
        type: string
        description: "Cookie value. Opaque to the framework."
      path:
        type: [string, null]
        description: "URL path prefix the cookie applies to (`Set-Cookie; Path=...`)."
      domain:
        type: [string, null]
        description: "Domain scope (`Set-Cookie; Domain=...`)."
      secure:
        type: boolean
        description: "True → `Set-Cookie; Secure`. Transmission only over HTTPS."
      http_only:
        type: boolean
        description: "True → `Set-Cookie; HttpOnly`. Inaccessible to JavaScript."
      max_age:
        type: [integer, null]
        description: "Lifetime in seconds. None → session cookie."
    x-mamba-binding:
      module: mambalibs.http
      symbol: Cookie
      extern_fn: cookie_new
      signature: "Cookie(name: str, value: str, path: str | None = None, domain: str | None = None, secure: bool = False, http_only: bool = False, max_age: int | None = None) -> Cookie"
    x-constructor:
      args:
        - { name: name,      mb_type: str,   rust_type: String }
        - { name: value,     mb_type: str,   rust_type: String }
        - { name: path,      mb_type: str,   rust_type: String, nullable: true }
        - { name: domain,    mb_type: str,   rust_type: String, nullable: true }
        - { name: secure,    mb_type: bool,  rust_type: bool,   default: "false" }
        - { name: http_only, mb_type: bool,  rust_type: bool,   default: "false" }
        - { name: max_age,   mb_type: int,   rust_type: i64,    nullable: true }
    x-mamba-attributes:
      - name: name
      - name: value
      - name: path
      - name: domain
      - name: secure
      - name: http_only
      - name: max_age

  Request:
    type: object
    required: [method, path]
    properties:
      method:
        type: string
        description: "HTTP method (GET, POST, PUT, PATCH, DELETE, HEAD, OPTIONS)."
      path:
        type: string
        description: "URL path of the request (no query string)."
      query_params:
        type: object
        x-rust-type: "std::collections::HashMap<String, String>"
        description: "Parsed query string. Multi-value params join by comma."
      headers:
        type: object
        x-rust-type: "std::collections::HashMap<String, String>"
        description: "Request headers, lowercase-keyed by convention."
      cookies:
        type: array
        items:
          $ref: "#/definitions/Cookie"
        description: "Parsed from the `Cookie` header."
      body:
        type: string
        x-rust-type: "Vec<u8>"
        description: "Raw body bytes. Handler deserializes into a payload model."
      path_params:
        type: object
        x-rust-type: "std::collections::HashMap<String, String>"
        description: "URL path captures populated by the router (e.g. `/users/{id}`)."
    x-mamba-binding:
      module: mambalibs.http
      symbol: Request
      extern_fn: request_new
      signature: "Request(method: str, path: str) -> Request"
    x-constructor:
      args:
        - { name: method, mb_type: str, rust_type: String }
        - { name: path,   mb_type: str, rust_type: String }
        - { name: query_params, mb_type: dict, rust_type: "std::collections::HashMap<String, String>", default: "std::collections::HashMap::new()" }
        - { name: headers,      mb_type: dict, rust_type: "std::collections::HashMap<String, String>", default: "std::collections::HashMap::new()" }
        - { name: cookies,      mb_type: list, rust_type: "Vec<crate::request_response::Cookie>", default: "Vec::new()" }
        - { name: body,         mb_type: list, rust_type: "Vec<u8>", default: "Vec::new()" }
        - { name: path_params,  mb_type: dict, rust_type: "std::collections::HashMap<String, String>", default: "std::collections::HashMap::new()" }
    x-mamba-attributes:
      - name: method
      - name: path

  Response:
    type: object
    required: [status_code, media_type]
    properties:
      status_code:
        type: integer
        x-rust-type: u16
        minimum: 100
        maximum: 599
        description: "HTTP status code. Validated at construction."
      body:
        type: string
        x-rust-type: "Vec<u8>"
        description: "Response body bytes. Handlers set this via JSONResponse / HTMLResponse / PlainTextResponse wrappers (future slice)."
      headers:
        type: object
        x-rust-type: "std::collections::HashMap<String, String>"
        description: "Response headers, lowercase-keyed by convention."
      cookies:
        type: array
        items:
          $ref: "#/definitions/Cookie"
        description: "Emitted as multiple `Set-Cookie` headers."
      media_type:
        type: string
        description: "Content-Type header value (e.g. `application/json`, `text/html`)."
    x-mamba-binding:
      module: mambalibs.http
      symbol: Response
      extern_fn: response_new
      signature: "Response(status_code: int = 200, media_type: str = 'application/json') -> Response"
    x-constructor:
      args:
        - { name: status_code, mb_type: int,  rust_type: u16,    default: "200" }
        - { name: body,        mb_type: list, rust_type: "Vec<u8>", default: "Vec::new()" }
        - { name: headers,     mb_type: dict, rust_type: "std::collections::HashMap<String, String>", default: "std::collections::HashMap::new()" }
        - { name: cookies,     mb_type: list, rust_type: "Vec<crate::request_response::Cookie>", default: "Vec::new()" }
        - { name: media_type,  mb_type: str,  rust_type: String, default: "\"application/json\".to_string()" }
      validations:
        - { field: status_code, rule: range, min: 100, max: 599, message: "status_code must be in [100, 599]" }
    x-mamba-attributes:
      - name: status_code
        rust_expr: "self_.status_code as i64"
      - name: media_type
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
  - "use mambalibs_http::request_response::{Cookie, Request, Response};"
  - "use std::collections::HashMap;"

tests:
  - name: cookie_with_secure_http_only_defaults_to_false
    setup: |
      let c = Cookie::new(
          "session".to_string(),
          "abc123".to_string(),
          None, None, false, false, None,
      ).unwrap();
    assertions:
      - 'assert_eq!(c.name, "session")'
      - 'assert_eq!(c.value, "abc123")'
      - "assert!(!c.secure)"
      - "assert!(!c.http_only)"
      - "assert!(c.max_age.is_none())"

  - name: cookie_secure_flag_roundtrips
    setup: |
      let c = Cookie::new(
          "sid".to_string(),
          "xyz".to_string(),
          Some("/".to_string()), Some("example.com".to_string()),
          true, true, Some(3600),
      ).unwrap();
    assertions:
      - "assert!(c.secure)"
      - "assert!(c.http_only)"
      - "assert_eq!(c.max_age, Some(3600))"
      - 'assert_eq!(c.path.as_deref(), Some("/"))'

  - name: request_defaults_have_empty_collections
    setup: |
      let req = Request::new(
          "GET".to_string(), "/health".to_string(),
          HashMap::new(), HashMap::new(), Vec::new(),
          Vec::new(), HashMap::new(),
      ).unwrap();
    assertions:
      - 'assert_eq!(req.method, "GET")'
      - 'assert_eq!(req.path, "/health")'
      - "assert!(req.query_params.as_ref().map_or(true, |m| m.is_empty()))"
      - "assert!(req.body.as_ref().map_or(true, |b| b.is_empty()))"

  - name: response_rejects_out_of_range_status
    setup: |
      let result = Response::new(
          700, Vec::new(), HashMap::new(),
          Vec::new(), "application/json".to_string(),
      );
    assertions:
      - "assert!(result.is_err())"

  - name: response_accepts_default_json_media_type
    setup: |
      let resp = Response::new(
          200, br#"{"ok":true}"#.to_vec(), HashMap::new(),
          Vec::new(), "application/json".to_string(),
      ).unwrap();
    assertions:
      - "assert_eq!(resp.status_code, 200)"
      - 'assert_eq!(resp.media_type, "application/json")'
      - "assert!(resp.body.as_ref().is_some_and(|b| !b.is_empty()))"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/mamba/mambalibs/httpkit/src/request_response.rs
    action: create
    section: schema
    description: |
      Generated Cookie + Request + Response structs, constructors, FFI
      shims, attribute getters, and per-type register fns + aggregate
      register() from the schema section.
  - path: projects/mamba/mambalibs/httpkit/Cargo.toml
    action: modify
    section: manifest
    description: |
      Dep list unchanged from http-exception / health specs — same api
      crate. Last spec regenerates wins; contents identical.
  - path: projects/mamba/mambalibs/httpkit/tests/request_response_test.rs
    action: create
    section: tests
    description: |
      Runnable tests covering Cookie flag defaults + round-trip,
      Request default collections, Response status-code validation,
      Response default media type.
  # projects/mamba/mambalibs/httpkit/src/lib.rs — auto-wired by apply.rs post-pass.
  # projects/mamba/mambalibs/httpkit/README.md — auto-aggregated Registered symbols table
  # will list Cookie + Request + Response.
```
