# lumen — generated clients

This directory holds the **OpenAPI contract** for lumen and the tooling to
regenerate Go / TypeScript / Python clients from it.

## Contract

`openapi.json` is the **committed reference spec** and is the single source of
truth that downstream client consumers integrate against. It is produced by:

```bash
cargo run -q -p lumen --bin lumen-openapi-dump > clients/openapi.json
```

…which serializes the `utoipa` schema attached to the live `lumen::api`
router. Because the router and spec share the same source, the spec cannot
silently drift from the implementation **inside** the crate; the only drift
risk is the committed `openapi.json` getting out of date relative to the
current build.

## Drift guard

CI (`.github/workflows/lumen.yml`, job `clients-drift`) runs
`lumen-openapi-dump` and compares its output to `clients/openapi.json`. **Any
difference fails the job.** When you make a public-API change you must:

1. Update the relevant `utoipa::ToSchema` / `utoipa::path` annotations.
2. Run `make spec` from this directory.
3. Commit the updated `openapi.json` in the same PR as the API change.

## Generating language clients

Language clients are **not checked in** — they are regenerated on demand
(see `.gitignore`). To produce them locally:

| Command       | Output                | Generator             |
|---------------|-----------------------|-----------------------|
| `make go`     | `clients/go/`         | `go`                  |
| `make ts`     | `clients/ts/`         | `typescript-fetch`    |
| `make python` | `clients/python/`     | `python`              |
| `make all`    | spec + all three      | —                     |
| `make clean`  | wipes the three above | —                     |

All three use the npm-packaged `@openapitools/openapi-generator-cli` invoked
through `npx`, so the only host requirement beyond the Rust toolchain is
`node` (any recent LTS version).

## Why not commit the language clients?

* They are deterministic functions of `openapi.json`; storing them just
  duplicates state and creates a review burden every time the schema moves.
* Each generator's output is large and stylistically idiomatic to its
  ecosystem — much of the diff noise is unrelated to the actual contract
  change.
* Consumers who want a pinned snapshot can vendor the generator output into
  their own repo and gate it against the same `openapi.json`.

The **spec** is the artifact we promise to keep stable; the clients are a
convenience build.
