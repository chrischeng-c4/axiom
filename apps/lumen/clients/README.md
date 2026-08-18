# lumen — generated clients

This directory holds lumen's committed **OpenAPI contract** and pinned client
target policy.

## Contract

`openapi.json` is the **committed reference spec** and is the single source of
truth that downstream client consumers integrate against. It is produced by:

```bash
cargo run -q -p lumen --bin lumen -- spec > apps/lumen/clients/openapi.json
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
| `cargo run -q -p lumen --bin lumen -- spec gen --lang ts --out apps/lumen/clients/ts` | TypeScript client |
| `cargo run -q -p lumen --bin lumen -- spec gen --lang py --out apps/lumen/clients/python` | Python client |
| `cargo run -q -p lumen --bin lumen -- spec gen --lang rust --out apps/lumen/clients/rust` | Rust client |

The generator is in-tree (`libs/openapi-codegen`), so the only requirement is
a cargo/rustup toolchain. `codegen.toml` pins the default TypeScript, Python,
and Rust contracts; every generated client contains
`.openapi-codegen.json`. Use `--target <profile>` only for a deliberate
one-off compatibility override.

## Connecting to a production fleet

The spec's first `servers` entry is `https://{instance}.{namespace}.svc:7373`,
not a published hostname. A production Lumen is a private ClusterIP that
terminates TLS in the serving pod itself — there is no Ingress, Gateway,
LoadBalancer, NodePort, or mesh in front of it — so a client needs two things
the public internet would have supplied for it:

1. **The trust anchor.** The deployment administrator or external certificate
   platform distributes the public serving CA separately from the
   private-key-bearing serving Secret. Pass its PEM file to `--ca-file`; do
   not read the serving Secret, which carries `tls.key`.
2. **The name to verify.** The leaf asserts the instance's own Service DNS
   names and nothing else, so the base URL must address the same name the
   certificate is checked against.

Every generated client takes both as one `PrivateTrust` value and **replaces**
the public roots with that anchor rather than adding to it — a private trust
domain merely added to the public set still lets any public CA certify this
name. None of the three exposes a way to skip verification.

```ts
import { Client, privateCaFetch, type PrivateTrust } from "./ts";

const trust: PrivateTrust = {
  caBundle: "/var/run/lumen-trust/ca.crt",
  serverName: "lumen.analytics.svc",
};
const client = new Client({
  baseUrl: "https://lumen.analytics.svc:7373",
  trust,
  fetch: await privateCaFetch(trust), // Node; a browser installs the anchor in the platform store
  headers: { Authorization: `Bearer ${ksaToken}` },
});
```

```python
from python import Client, PrivateTrust

client = Client(
    "https://lumen.analytics.svc:7373",
    trust=PrivateTrust(
        ca_bundle="/var/run/lumen-trust/ca.crt",
        server_name="lumen.analytics.svc",
    ),
    auth_token=ksa_token,
)
```

```rust
let client = Client::with_private_ca(
    "https://lumen.analytics.svc:7373",
    TransportPolicy::default(),
    PrivateTrust {
        ca_bundle: "/var/run/lumen-trust/ca.crt".into(),
        server_name: "lumen.analytics.svc".into(),
    },
)?;
```

`auth_token` / the `Authorization` header is a short-lived, audience-bound
Kubernetes ServiceAccount token from the TokenRequest API — see the project
README's *Authentication and authorization* section. It is a bearer credential,
which is exactly why the transport above is not optional. For an ad-hoc shell
against a fleet, `lumen connect --ca-file <anchor>` does the same two steps
plus the TokenRequest.

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
