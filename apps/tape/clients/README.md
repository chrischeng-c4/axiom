<!-- HANDWRITE-BEGIN gap="missing-generator:logic:a48eafff" tracker="pending-tracker" reason="Usage doc for the clients/ scaffold: direct CLI generation of the OpenAPI snapshot and language clients without a Makefile wrapper." -->
# tape — generated clients

This directory holds tape's committed **OpenAPI contract** and pinned client
target policy (WI #1329).

## Contract

`openapi.json` is a **committed snapshot** of `tape spec --format openapi` —
the same document `GET /openapi.json` serves — so client generation works
fully offline, without a running server:

```bash
cargo run -q -p tape --bin tape -- spec --format openapi > apps/tape/clients/openapi.json
```

## Generating language clients

Clients are generated **in-binary** via `tape spec gen` and the shared
`libs/openapi-codegen` crate — no Makefile, external tool, `node`, or `npx`
requirement.

| Command | Output |
|---|---|
| `cargo run -q -p tape --bin tape -- spec gen --lang ts --out apps/tape/clients/ts` | TypeScript client |
| `cargo run -q -p tape --bin tape -- spec gen --lang py --out apps/tape/clients/py` | Python client |
| `cargo run -q -p tape --bin tape -- spec gen --lang rust --out apps/tape/clients/rust` | Rust client |

Requirements: a `cargo`/rustup toolchain only.

`codegen.toml` pins the default target for each language. Every generated
client includes `.cclab-openapi-codegen.json` with the exact target contract;
pass `--target <profile>` to `tape spec gen` only for an explicit override.

## Why not commit the language clients?

They are a deterministic function of `openapi.json`; committing them just
duplicates state and creates review churn every time the contract moves.
Consumers who want a pinned snapshot can vendor the generator output into
their own repo, gated against this `openapi.json`.

## Keeping `openapi.json` current

When tape's public HTTP contract changes (new route, new schema field), rerun
the direct `tape spec --format openapi` command above and commit the updated
`openapi.json` in the same change.
<!-- HANDWRITE-END -->
