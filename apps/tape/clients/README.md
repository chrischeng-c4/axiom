<!-- HANDWRITE-BEGIN gap="missing-generator:logic:a48eafff" tracker="pending-tracker" reason="Usage doc for the clients/ scaffold: what openapi.json is, how to regenerate it and the per-language clients via the Makefile, mirroring lumen's clients/README.md." -->
# tape — generated clients

This directory holds the **OpenAPI contract** for tape and the tooling to
regenerate TypeScript / Python / Rust clients from it (WI #1329).

## Contract

`openapi.json` is a **committed snapshot** of `tape spec --format openapi` —
the same document `GET /openapi.json` serves — so client generation works
fully offline, without a running server:

```bash
make refresh-openapi
```

## Generating language clients

Unlike lumen's `clients/` (which shells out to the npm-packaged
`@openapitools/openapi-generator-cli`), tape's clients are generated
**in-binary** via `tape spec gen` and the shared `libs/openapi-codegen`
crate — no external tool, no `node`/`npx` requirement.

| Command         | Output           | Backing verb                          |
|------------------|------------------|----------------------------------------|
| `make gen-ts`    | `clients/ts/`    | `tape spec gen --lang ts --out ...`   |
| `make gen-py`    | `clients/py/`    | `tape spec gen --lang py --out ...`   |
| `make gen-rust`  | `clients/rust/`  | `tape spec gen --lang rust --out ...` |
| `make gen-all`   | all three        | —                                       |
| `make clean`     | wipes the three above | —                                  |

Requirements: a `cargo`/rustup toolchain only.

## Why not commit the language clients?

They are a deterministic function of `openapi.json`; committing them just
duplicates state and creates review churn every time the contract moves.
Consumers who want a pinned snapshot can vendor the generator output into
their own repo, gated against this `openapi.json`.

## Keeping `openapi.json` current

When tape's public HTTP contract changes (new route, new schema field), run
`make refresh-openapi` from this directory and commit the updated
`openapi.json` in the same change.
<!-- HANDWRITE-END -->
