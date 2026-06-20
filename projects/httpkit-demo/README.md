# httpkit-demo

## Brief

httpkit-demo is a generated demo consumer of the `mambalibs.http` framework.

It shows the pydantic-like BaseModel pattern for user-defined request payload
models in Mamba: a TD-authored request schema becomes a Rust struct,
constructor validation, serde support, Mamba FFI entrypoints, attribute getters,
and a `MambaModule` registrar.

Every type here is produced 100% by TD v2 codegen from specs under
`.score/tech_design/projects/httpkit-demo/`. The same pipeline that produces
`mambalibs.http` framework types (HTTPException, Request, Response, ...) produces
user payload types — identical authoring surface, no distinction between
framework and user code from the codegen's perspective.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Mamba HTTP Request Payload Demo | - | implemented | verified | smoke | not_ready | runtime smoke passes; production readiness still depends on TD/linkage closure |

### Mamba HTTP Request Payload Demo

ID: mamba-http-request-payload-demo
Type: DeveloperTool
Surfaces: Mamba module: `mambalibs.httpkit_demo`; Rust API: `CreateUserRequest`, `CreateUserRequest::new`, `HttpkitDemoModule`; FFI: `create_user_request_new`, `create_user_request_get_name`, `create_user_request_get_email`, `create_user_request_get_age`; Tests: `cargo test -p httpkit-demo`
EC Dimensions: behavior: `cargo test -p httpkit-demo` - generated schema, validation, serde, and Mamba registration smoke
Root WI: -
Status: verified
Required Verification: smoke
Promise:
httpkit-demo proves that TD v2 codegen can produce a user-defined HTTP request payload model with constructor validation, JSON serialization, Mamba runtime registration, and generated attribute getter entrypoints using the same authoring surface as framework-owned HTTP types.
Gate Inventory: `cargo test -p httpkit-demo`; projects/httpkit-demo/tests/create_user_request_test.rs; projects/httpkit-demo/src/create_user_request.rs; projects/httpkit-demo/src/lib.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Generated request model validation contract | epic | - | implemented | verified | smoke | `cargo test -p httpkit-demo`; projects/httpkit-demo/tests/create_user_request_test.rs |
| Mamba module and FFI compile/registration contract | epic | - | implemented | verified | smoke | `cargo test -p httpkit-demo`; projects/httpkit-demo/src/create_user_request.rs; projects/httpkit-demo/src/lib.rs |

## Registered symbols

<!-- SPEC-MANAGED: generated/readme#mamba-symbols -->
<!-- CODEGEN-BEGIN -->
| Symbol | Spec |
| --- | --- |
| `CreateUserRequest` | [.score/tech_design/projects/httpkit-demo/create-user-request.md](.score/tech_design/projects/httpkit-demo/create-user-request.md) |
<!-- CODEGEN-END -->
