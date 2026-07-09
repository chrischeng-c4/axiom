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
| GitHub Issues Proxy | #1332 | partial | pending | none | not ready | forwards search/view/create/comment to GitHub with a server-held credential |

### GitHub Issues Proxy

ID: github-issues-proxy
Type: DeveloperTool
Root WI: #1332
Status: partial
Surfaces: HTTP: `GET /v1/issues/{owner}/{name}`, `GET /v1/issues/{owner}/{name}/{number}`, `POST /v1/issues/{owner}/{name}`, `POST /v1/issues/{owner}/{name}/{number}/comments`; CLI: `courier llm|upgrade|issue`.
EC Dimensions: behavior: `cargo test -p courier` - proxy forwarding, auth, and repo allow-list coverage
Required Verification: smoke
Promise:
Every axiom CLI can search/view/create/comment on GitHub issues by
authenticating to `courier` with a shared bearer token, without holding a
personal GitHub credential.
Gate Inventory: `cargo test -p courier`; apps/courier/src/http

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| github-issues-proxy-service | epic | #1332 | partial | pending | none | apps/courier/src/http |
