# courier

## Brief

`courier` is a stateless, GCP-hosted proxy that centralizes GitHub-issue
access for every axiom CLI. It holds the real GitHub credential server-side
and forwards `issue search/view/create/comment` calls to `api.github.com`,
so individual dev machines and CI runners authenticate with a shared
`courier` bearer token instead of each needing their own GitHub credential.
GitHub remains the source of truth for issue data — `courier` stores nothing
of its own.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| GitHub Issues Proxy | - | implementing | pending | none | not ready | forwards search/view/create/comment to GitHub with a server-held credential |

### GitHub Issues Proxy

ID: github-issues-proxy
Type: DeveloperTool
Root WI: -
Status: implementing
Surfaces: HTTP: `GET /v1/issues`, `GET /v1/issues/{repo...}/{number}`, `POST /v1/issues/{repo...}`, `POST /v1/issues/{repo...}/{number}/comments`; CLI: `courier llm|upgrade|issue`.
EC Dimensions: behavior: `cargo test -p courier` - proxy forwarding, auth, and repo allow-list coverage
Required Verification: smoke
Promise:
Every axiom CLI can search/view/create/comment on GitHub issues by
authenticating to `courier` with a shared bearer token, without holding a
personal GitHub credential.
Gate Inventory: `cargo test -p courier`; apps/courier/src/http

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| github-issues-proxy-service | epic | - | implementing | pending | none | apps/courier/src/http |
