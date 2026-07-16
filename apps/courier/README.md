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
| GitHub Issues Proxy | #1332 | full | passing | smoke | ready | forwards search/view/create/comment to GitHub with a server-held credential |
| Chainable Output Conformance | - | full | passing | smoke | ready | outputs conform to standard agent-chainable command envelope protocol |
| CLI Interface | - | full | passing | smoke | ready | exposes a CLI binary interface to run and control the courier proxy |
| CLI Standard Surface | - | full | passing | smoke | ready | exposes standard offline documentation, self-updater, and issue-tracking CLI |
| EC Gates Configured | - | full | passing | smoke | ready | verifies capability contracts via external-contract gate tests |
| HTTP/2 API List | - | full | passing | smoke | ready | exposes a standardized HTTP/2-compatible OpenAPI endpoints list |
| Kubernetes-Native Deployment | - | full | passing | smoke | ready | exposes Kubernetes operator, CRD, and instance manifests |
| Long-Running Stability | - | full | passing | smoke | ready | runs reliably as a stateless daemon proxy service under high request volumes |
| Security Hardening | - | full | passing | smoke | ready | denies unauthorized access by verifying credentials using service-auth role mapping |
| Standard Operational Endpoints | - | full | passing | smoke | ready | exposes standard operational health, Prometheus metrics, and OpenAPI endpoints |

### GitHub Issues Proxy

ID: github-issues-proxy
Type: DeveloperTool
Root WI: #1332
Status: full
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
| github-issues-proxy-service | epic | #1332 | full | passing | smoke | apps/courier/src/http |

### Chainable Output Conformance

ID: chainable-output-conformance
Type: DeveloperTool
Root WI: -
Status: full
Surfaces: CLI: `courier` stdout.
EC Dimensions: behavior: stdout conforms to the standard agent-chainable command envelope protocol
Required Verification: smoke
Promise:
Outputs conform to the standard agent-chainable command envelope protocol so they can be parsed by `aw`.
Gate Inventory: `cargo test -p courier`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| chainable-output-conformance-root | epic | - | full | passing | smoke | apps/courier/src/bin/courier.rs |

### CLI Interface

ID: cli-interface
Type: DeveloperTool
Root WI: -
Status: full
Surfaces: CLI: `courier` binary CLI.
EC Dimensions: behavior: command line parsing and validation
Required Verification: smoke
Promise:
Exposes a CLI binary interface to run and control the courier proxy.
Gate Inventory: `cargo test -p courier`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| cli-interface-root | epic | - | full | passing | smoke | apps/courier/src/bin/courier.rs |

### CLI Standard Surface

ID: cli-standard-surface
Type: DeveloperTool
Root WI: -
Status: full
Surfaces: CLI: `courier llm|upgrade|issue`.
EC Dimensions: behavior: offline docs, self-updates, and issue reporting
Required Verification: smoke
Promise:
Exposes the standard offline documentation (`llm`), self-updater (`upgrade`), and issue-tracking (`issue`) CLI command surface.
Gate Inventory: `cargo test -p courier`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| cli-standard-surface-root | epic | - | full | passing | smoke | apps/courier/src/llm.rs |

### EC Gates Configured

ID: ec-gates-configured
Type: Devops
Root WI: -
Status: full
Surfaces: CLI: `courier` execution verification.
EC Dimensions: behavior: execution verification contract
Required Verification: smoke
Promise:
Verifies capability contracts via external-contract (EC) gate tests to guarantee correctness.
Gate Inventory: `cargo test -p courier`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| ec-gates-configured-root | epic | - | full | passing | smoke | apps/courier/Cargo.toml |

### HTTP/2 API List

ID: http2-api-list
Type: Service
Root WI: -
Status: full
Surfaces: HTTP: `GET /v1/openapi.json` (OpenAPI listing).
EC Dimensions: behavior: HTTP/2 cleartext and protocol conformance
Required Verification: smoke
Promise:
Exposes a standardized HTTP/2-compatible OpenAPI endpoints list.
Gate Inventory: `cargo test -p courier`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| http2-api-list-root | epic | - | full | passing | smoke | apps/courier/src/http/openapi.rs |

### Kubernetes-Native Deployment

ID: kubernetes-native-deployment
Type: Devops
Root WI: -
Status: full
Surfaces: CLI: `courier k8s operator|instance|crd render`.
EC Dimensions: behavior: Kubernetes artifact rendering
Required Verification: smoke
Promise:
Exposes Kubernetes operator, CRD, and instance manifests for deployment on GCP/K8s.
Gate Inventory: `cargo test -p courier`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| kubernetes-native-deployment-root | epic | - | full | passing | smoke | apps/courier/Cargo.toml |

### Long-Running Stability

ID: long-running-stability
Type: Service
Root WI: -
Status: full
Surfaces: HTTP: `GET /health` (liveness/readiness probe).
EC Dimensions: behavior: memory stability and graceful shutdown
Required Verification: smoke
Promise:
Runs reliably as a stateless daemon proxy service under high request volumes.
Gate Inventory: `cargo test -p courier`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| long-running-stability-root | epic | - | full | passing | smoke | apps/courier/src/http/mod.rs |

### Security Hardening

ID: security-hardening
Type: SecurityTool
Root WI: -
Status: full
Surfaces: HTTP Auth header bearer validation.
EC Dimensions: behavior: credential validation and role mapping
Required Verification: smoke
Promise:
Denies unauthorized access by verifying credentials using `libs/service-auth`'s role mapping.
Gate Inventory: `cargo test -p courier`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| security-hardening-root | epic | - | full | passing | smoke | apps/courier/src/http/auth.rs |

### Standard Operational Endpoints

ID: standard-operational-endpoints
Type: Service
Root WI: -
Status: full
Surfaces: HTTP: `GET /health`, `GET /metrics`, `GET /v1/openapi.json`.
EC Dimensions: behavior: metric instrumentation and diagnostic endpoints
Required Verification: smoke
Promise:
Exposes standard operational health, Prometheus metrics, and OpenAPI endpoints.
Gate Inventory: `cargo test -p courier`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| standard-operational-endpoints-root | epic | - | full | passing | smoke | apps/courier/src/http/mod.rs |

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
