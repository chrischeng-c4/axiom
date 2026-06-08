# `httpkit-demo`

Demo consumer of the `mambalibs.http` framework — shows the pydantic-like BaseModel
pattern for user-defined request payload models.

Every type here is produced 100% by TD v2 codegen from specs under
`.score/tech_design/projects/httpkit-demo/`. The same pipeline that produces
`mambalibs.http` framework types (HTTPException, Request, Response, ...) produces
user payload types — identical authoring surface, no distinction between
framework and user code from the codegen's perspective.

## Registered symbols

<!-- SPEC-MANAGED: generated/readme#mamba-symbols -->
<!-- CODEGEN-BEGIN -->
| Symbol | Spec |
| --- | --- |
| `CreateUserRequest` | [.score/tech_design/projects/httpkit-demo/create-user-request.md](.score/tech_design/projects/httpkit-demo/create-user-request.md) |
<!-- CODEGEN-END -->