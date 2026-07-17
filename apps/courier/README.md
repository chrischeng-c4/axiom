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
| GitHub Issues Proxy | #1332 | implemented | passing | smoke | ready | forwards search/view/create/comment to GitHub with a server-held credential |
| Security Hardening | - | implemented | passing | smoke | ready | denies unauthorized access by verifying credentials using service-auth role mapping |
| Long-Running Stability | - | implemented | passing | smoke | ready | runs reliably as a stateless daemon proxy service under high request volumes |

### GitHub Issues Proxy

ID: github-issues-proxy
Type: Service
Root WI: #1332
Status: full
Surfaces: HTTP: `GET /issues`, `POST /issues`
EC Dimensions: behavior: `python -m pytest` - proxy forwarding, and repo allow-list coverage
Required Verification: smoke
Promise:
Every axiom CLI can search/view/create/comment on GitHub issues by
authenticating to `courier` with a shared bearer token, without holding a
personal GitHub credential.
Gate Inventory: `python -m pytest`; apps/courier/app

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| github-issues-proxy-service | epic | #1332 | implemented | passing | smoke | apps/courier/app |

### Security Hardening

ID: security-hardening
Type: SecurityTool
Root WI: -
Status: full
Surfaces: HTTP Auth header bearer validation.
EC Dimensions: behavior: credential validation against accepted tokens
Required Verification: smoke
Promise:
Denies unauthorized access by verifying credentials using bearer tokens in the Authorization header.
Gate Inventory: `python -m pytest`; apps/courier/app

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| security-hardening-root | epic | - | implemented | passing | smoke | apps/courier/app/main.py |

### Long-Running Stability

ID: long-running-stability
Type: Service
Root WI: -
Status: full
Surfaces: HTTP: `GET /healthz` and `GET /readyz`.
EC Dimensions: behavior: memory stability and graceful shutdown
Required Verification: smoke
Promise:
Runs reliably as a stateless FastAPI service.
Gate Inventory: `python -m pytest`; apps/courier/app

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| long-running-stability-root | epic | - | implemented | passing | smoke | apps/courier/app/main.py |

<!-- aw:meta:project-readme:start -->
## Brief

`courier` is a stateless, GCP-hosted proxy that centralizes GitHub-issue
access for every axiom CLI. It holds the real GitHub credential server-side
and forwards `issue search/view/create/comment` calls to `api.github.com`,
so individual dev machines and CI runners authenticate with a shared
`courier` bearer token instead of each needing their own GitHub credential.
GitHub remains the source of truth for issue data — `courier` stores nothing
of its own.

## Contributing

Project-local authoring and verification rules live in [CONTRIBUTING.md](CONTRIBUTING.md).

## Capability Contract

Product promises and work roots live in [CAPABILITIES.md](CAPABILITIES.md).
<!-- aw:meta:project-readme:end -->
