# lumen — generated clients

This directory holds lumen's committed **OpenAPI contract** and pinned client
target policy.

## Contract

`openapi.json` is the **committed reference spec** and is the single source of
truth that downstream client consumers integrate against. It is produced by:

```bash
cargo run -q -p lumen --bin lumen -- spec > projects/lumen/clients/openapi.json
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
2. Run the direct `lumen spec` command above.
3. Commit the updated `openapi.json` in the same PR as the API change.

## Generating language clients

Language clients are **not checked in** — they are regenerated on demand
(see `.gitignore`) through the service CLI:

| Command | Output |
|---|---|
| `cargo run -q -p lumen --bin lumen -- spec gen --lang ts --out projects/lumen/clients/ts` | TypeScript client |
| `cargo run -q -p lumen --bin lumen -- spec gen --lang py --out projects/lumen/clients/python` | Python client |
| `cargo run -q -p lumen --bin lumen -- spec gen --lang rust --out projects/lumen/clients/rust` | Rust client |

The generator is in-tree (`libs/openapi-codegen`), so the only requirement is
a cargo/rustup toolchain. `codegen.toml` pins the default TypeScript, Python,
and Rust contracts; every generated client contains
`.cclab-openapi-codegen.json`. Use `--target <profile>` only for a deliberate
one-off compatibility override.

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
