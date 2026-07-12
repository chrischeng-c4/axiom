# Pgpool

## Brief

`pgpool` is the working app id for Axiom's Kubernetes-native PostgreSQL
connection pooler. The final product name is not settled yet; until then the
repository path, crate name, binary name, and tracker label use `pgpool` as a
stable implementation placeholder.

The app owns the database-pooling data plane: frontend PostgreSQL TCP
admission, backend connection budgeting, drain-aware shutdown, pool health, and
standard operational HTTP endpoints. Cloud SQL, AlloyDB, and other platform
connectors stay explicit adapters above this core rather than being baked into
the pooler runtime.

Current implementation slice: `apps/pgpool` is a Rust workspace crate and
binary with an offline runtime plan, OpenAPI-shaped admin route inventory,
agent docs, and shared server substrate dependencies wired through
`server-core`, `tcp-server`, and `http-server`. PostgreSQL wire handling,
auth/backend adapters, k8s operator artifacts, and real database integration
gates are separate work roots.

## Boundaries

- `pgpool` owns Postgres-compatible pooling and proxy admission.
- `server-core`, `tcp-server`, and `http-server` own generic server runtime
  mechanics; `pgpool` composes them instead of duplicating accept loops,
  connection budgets, h2c serving, drain, or tracing.
- Platform adapters such as Cloud SQL Proxy or AlloyDB endpoint discovery stay
  optional integration layers, not required runtime dependencies.
- Application services should connect to `pgpool` over the PostgreSQL wire
  protocol and inspect operations through the admin HTTP surface.

## Capabilities

The baseline capabilities selected by aw.toml's `service` umbrella profile
(plus `cli_facing`, `competitive_replacement`, `kubernetes_native`,
`long_running`, and `network_exposed`) are mandatory for this pooler class.
They do not replace pgpool's product capabilities; the PostgreSQL pooler core
and the platform adapter boundary remain first-class domain roots.

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Working-Name App Scaffold | - | implemented | passing | smoke | partial | crate/bin/README/AW metadata and route inventory are present under `apps/pgpool` |
| Shared Server Substrate Adoption | - | implemented | passing | smoke | partial | runtime plan composes `server-core`, `tcp-server`, and `http-server` types |
| PostgreSQL Pooler Core | 1282 | planned | planned | none | not_ready | domain: frontend pg wire parser, backend pool, transaction/session modes |
| Platform Adapter Boundary | 1283 | planned | planned | none | not_ready | domain: Cloud SQL/AlloyDB discovery/auth adapters remain outside core runtime |
| CLI Interface | 1282 | implemented | passing | smoke | partial | mandatory baseline: single `pgpool` bin with runtime-plan/spec verbs; serve entrypoint remains open |
| CLI Standard Surface | - | implemented | passing | smoke | partial | mandatory baseline: shared `cli-std` llm/upgrade/issue surface with build-stamp provenance |
| Chainable Output Conformance | - | implemented | passing | smoke | partial | mandatory baseline: `runtime-plan` emits `next:`; raw spec streams stay unwrapped |
| Competitor Feature Parity | 1285 | planned | planned | none | not_ready | mandatory baseline: PgBouncer/Odyssey/pgcat transaction-pooling replacement breadth |
| Competitor Performance | 1285 | planned | planned | none | not_ready | mandatory baseline: vat-isolated meter throughput/latency ratchet vs PgBouncer-class poolers |
| EC Gates Configured | 1285 | planned | planned | none | not_ready | mandatory baseline: aw.toml EC inventory, vat meter/guard runners, external-contracts evidence |
| HTTP/2 API List | 1282 | implemented | passing | smoke | partial | mandatory baseline: offline `pgpool spec` admin route inventory; served contract remains open |
| Kubernetes-Native Deployment | 1284 | planned | planned | none | not_ready | mandatory baseline: PgpoolSpec CRD/operator/instance render and pod drain behavior |
| Long-Running Stability | 1282 | planned | planned | none | not_ready | mandatory baseline: backend reuse without leaks, graceful drain, restart safety |
| Security Hardening | 1286 | planned | planned | none | not_ready | mandatory baseline: frontend auth passthrough, TLS posture, admin-plane exposure gates |
| Standard Operational Endpoints | 1282 | planned | planned | smoke | not_ready | mandatory baseline: one-port `/healthz`, `/readyz`, `/metrics`, `/openapi.json`, `/docs`; offline twin exists |

### Working-Name App Scaffold

ID: working-name-app-scaffold
Type: RuntimeTool
Surfaces: CLI: `pgpool runtime-plan` - offline shared-runtime plan for the data and admin planes.; Config: `apps/pgpool/aw.toml` - project registration, capability profile traits, and workspace test gate.
EC Dimensions: behavior: `cargo test -p pgpool --test cli_contract` - compiled-binary contract for the scaffold surface
Root WI: -
Status: auditing
Required Verification: smoke
Promise:
Hold `pgpool` as a stable working app id — crate, binary, README capability
map, and AW metadata live under `apps/pgpool` — so pooler work roots can land
before the final product name is settled, without renaming churn.
Gate Inventory:
- apps/pgpool/tests/cli_contract.rs; apps/pgpool/aw.toml; apps/pgpool/src/bin/pgpool.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| app-scaffold-and-crate-layout | change | - | implemented | passing | smoke | apps/pgpool/tests/cli_contract.rs |

### Shared Server Substrate Adoption

ID: shared-server-substrate-adoption
Type: RuntimeTool
Surfaces: Rust API: `RuntimePlan` - composes `server-core` bind/budget/drain, `tcp-server` socket options, and `http-server` h2c options.; CLI: `pgpool runtime-plan` - JSON plan naming the shared libs.
EC Dimensions: behavior: `cargo test -p pgpool` - runtime plan composes shared substrate types instead of local reinventions
Root WI: -
Status: auditing
Required Verification: smoke
Promise:
`pgpool` starts from the shared service substrate instead of inventing a local
accept loop or HTTP admin server. The TCP data-plane listener uses
`tcp-server` concepts, the admin listener uses `http-server`/h2c concepts, and
connection limits/readiness/drain are represented by `server-core`.
Gate Inventory:
- apps/pgpool/src/lib.rs; apps/pgpool/tests/cli_contract.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| app-scaffold-and-runtime-plan | change | - | implemented | passing | smoke | apps/pgpool/src/lib.rs |

### PostgreSQL Pooler Core

ID: postgres-pooler-core
Type: RuntimeTool
Surfaces: TCP: `0.0.0.0:6432` - PostgreSQL wire protocol frontend admission for application clients.; Rust API: `RuntimePlan` - pool mode, frontend budget, and backend budget configuration.
EC Dimensions: behavior: pending pg wire parser and pool lifecycle conformance gates - startup/auth passthrough, transaction/session pooling, drain
Root WI: 1282
Status: confirmed
Required Verification: conformance, performance, negative
Promise:
Provide a high-throughput PostgreSQL pooler with bounded frontend admission,
backend connection reuse, transaction/session pool modes, graceful drain, and
clear observability before platform-specific adapters are added.
Gate Inventory:
- apps/pgpool/tests/wire_codec.rs (`cargo test -p pgpool --test wire_codec`)
- apps/pgpool/tests/proxy.rs; apps/pgpool/tests/session_proxy.rs (`cargo test -p pgpool --test proxy --test session_proxy`)
- apps/pgpool/tests/pool.rs; apps/pgpool/tests/pool_modes.rs (`cargo test -p pgpool --test pool --test pool_modes`)

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| pg-wire-frontend-protocol | epic | 1287 | implemented | passing | conformance | apps/pgpool/tests/wire_codec.rs; apps/pgpool/tech-design/logic/pg-wire-message-codec-frontend-backend-frames.md |
| backend-pool-and-reuse | epic | 1289 | implemented | passing | conformance | apps/pgpool/tests/pool.rs; apps/pgpool/tests/pool_modes.rs; apps/pgpool/tech-design/logic/backend-pool-connection-reuse-and-transaction-session-pool-modes.md |
| transaction-session-pool-modes | epic | 1289 | implemented | passing | conformance | apps/pgpool/tests/pool.rs; apps/pgpool/tests/pool_modes.rs; apps/pgpool/tech-design/logic/backend-pool-connection-reuse-and-transaction-session-pool-modes.md |
| serve-entrypoint-and-drain | epic | 1288 | implemented | passing | conformance | apps/pgpool/tests/proxy.rs; apps/pgpool/tests/session_proxy.rs; apps/pgpool/tech-design/logic/session-mode-proxy-with-auth-passthrough-and-serve-entrypoint.md |

### Platform Adapter Boundary

ID: platform-adapter-boundary
Type: RuntimeTool
Surfaces: Rust API: backend endpoint/auth adapter seam - Cloud SQL, AlloyDB, and plain-Postgres backends supply endpoints and auth material above the core runtime.
EC Dimensions: behavior: pending adapter seam conformance gates - core runtime stays adapter-free; adapters compose from outside
Root WI: 1283
Status: confirmed
Required Verification: conformance
Promise:
Keep Cloud SQL Proxy, AlloyDB endpoint discovery, and other platform
connectors as explicit adapters above the pooler core: the core runtime never
embeds platform SDKs, and adapters only supply backend endpoints and auth
material through a stable seam.
Gate Inventory:
- pending: adapter seam contract tests
- pending: plain-Postgres backend integration gate

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| backend-adapter-seam | epic | 1283 | planned | planned | none | pending: adapter seam contract tests |

### CLI Interface

ID: cli-interface
Type: RuntimeTool
Surfaces: CLI: `pgpool` - single bin; `runtime-plan` and `spec` verbs today, serve-by-default data/admin plane entrypoint planned.
EC Dimensions: behavior: `cargo test -p pgpool --test cli_contract` - compiled-binary help/verb contract
Root WI: 1282
Status: auditing
Required Verification: smoke
Promise:
Expose pgpool as one runnable binary with a stable process entrypoint — serve
by default once the pooler core lands — plus offline runtime-plan and spec
verbs for agents and operators.
Gate Inventory:
- apps/pgpool/tests/cli_contract.rs; apps/pgpool/src/bin/pgpool.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| offline-plan-and-spec-verbs | change | - | implemented | passing | smoke | apps/pgpool/tests/cli_contract.rs |
| serve-by-default-entrypoint | epic | 1288 | implemented | passing | smoke | apps/pgpool/tests/cli_contract.rs (`help_and_llm_workflow_topic_mention_serve`); apps/pgpool/src/bin/pgpool.rs |

### CLI Standard Surface

ID: cli-standard-surface
Type: RuntimeTool
Surfaces: CLI: `pgpool llm` - offline agent self-doc topics (outline, workflow, api, boundaries).; CLI: `pgpool upgrade` - shared self-update and `--check` surface through `cli-std`.; CLI: `pgpool issue search|view|create|comment` - shared tracker surface scoped to `project:pgpool`.
EC Dimensions: behavior: `cargo test -p pgpool --test cli_contract` - llm/upgrade/issue appear in the compiled binary help contract
Root WI: -
Status: auditing
Required Verification: smoke
Promise:
Ship the mandatory shared `cli-std` surface (llm/upgrade/issue) every
ecosystem CLI owes, backed by build-stamp provenance, without blurring it into
pgpool's domain verbs.
Gate Inventory:
- apps/pgpool/src/bin/pgpool.rs; apps/pgpool/tests/cli_contract.rs; libs/cli-std/src

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| shared-llm-upgrade-issue-surface | change | - | implemented | passing | smoke | apps/pgpool/tests/cli_contract.rs |

### Chainable Output Conformance

ID: chainable-output-conformance
Type: RuntimeTool
Surfaces: CLI: `pgpool spec --format openapi|openapi-yaml|json-schema|routes` - raw artifact streams that intentionally stay unwrapped bytes.; CLI: `pgpool runtime-plan` - operational output carrying a runnable `next:` step.
EC Dimensions: behavior: `cargo test -p pgpool --test cli_contract` - runtime-plan emits `next: pgpool spec --format routes`; spec stdout stays raw parseable bytes
Root WI: -
Status: auditing
Required Verification: smoke
Promise:
Keep pgpool's CLI outputs chainable per the CLI convention: raw artifact
streams (spec renders) stay unwrapped bytes, while operational verbs carry
explicit `next:`/terminal markers.
Gate Inventory:
- apps/pgpool/tests/cli_contract.rs; apps/pgpool/src/bin/pgpool.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| next-marker-on-runtime-plan | change | - | implemented | passing | smoke | apps/pgpool/tests/cli_contract.rs |
| raw-spec-streams-stay-unwrapped | change | - | implemented | passing | smoke | apps/pgpool/tests/cli_contract.rs |

### Competitor Feature Parity

ID: competitor-feature-parity
Type: RuntimeTool
Surfaces: TCP: PostgreSQL wire frontend - transaction/session pooling workflows PgBouncer-class poolers cover.; HTTP: admin pool/stats/drain routes - operational parity with pooler admin consoles.
EC Dimensions: behavior: pending pooler parity conformance gates - transaction pooling, session pooling, drain, and stats parity vs PgBouncer/Odyssey/pgcat
Root WI: 1285
Status: candidate
Required Verification: conformance, dogfood
Promise:
Cover the baseline connection-pooler functions pgpool needs to replace
PgBouncer, Odyssey, and pgcat for Axiom workloads: transaction and session
pooling, bounded admission, drain, and pool observability.
Gate Inventory:
- pending: parity conformance matrix vs PgBouncer/Odyssey/pgcat

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| pooler-parity-matrix | epic | 1285 | planned | planned | none | pending: parity conformance gates |

### Competitor Performance

ID: competitor-performance
Type: RuntimeTool
Surfaces: Meter/Vat: pending `apps/pgpool/vat.toml#meter-perf` - isolated meter execution for the throughput/latency ratchet.; Harness: pending external pooler comparison vs PgBouncer-class targets.
EC Dimensions: efficiency: pending vat meter-perf runner - meter-owned throughput/latency model and ratchet conformance
Root WI: 1285
Status: candidate
Required Verification: dogfood
Promise:
Tie pgpool's performance claims to repeatable pooled-connection throughput and
latency tests under a vat-isolated meter gate, with the external PgBouncer /
Odyssey / pgcat comparison as advisory dogfood until promoted.
Gate Inventory:
- pending: apps/pgpool/vat.toml meter-perf runner
- pending: pooled-throughput deterministic local gate

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| vat-meter-throughput-gate | epic | 1285 | planned | planned | none | pending: vat meter-perf runner |
| external-pooler-comparison | epic | 1285 | planned | planned | none | pending: bench harness vs PgBouncer-class poolers |

### EC Gates Configured

ID: ec-gates-configured
Type: Devops
Surfaces: Config: `apps/pgpool/aw.toml` - AW EC inventory and generated dispatch commands (pending).; Config: pending `apps/pgpool/vat.toml` - vat-managed meter/guard runners.
EC Dimensions: behavior: pending `aw ec gen --verify` inventory - EC dimensions configured and dispatchable for the pooler capability set
Root WI: 1285
Status: candidate
Required Verification: conformance
Promise:
Keep pgpool's service-trait EC baseline explicit and runnable: aw.toml owns
the EC inventory, vat owns the meter/guard runners, and external-contracts/
carries the evidence contracts each gate closes against.
Gate Inventory:
- pending: aw.toml EC inventory
- pending: vat meter/guard runners and external-contracts evidence

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| aw-ec-inventory-and-runners | epic | 1285 | planned | planned | none | pending: aw ec gen --verify |

### HTTP/2 API List

ID: http2-api-list
Type: RuntimeTool
Surfaces: CLI: `pgpool spec --format routes|openapi|openapi-yaml|json-schema` - offline admin API inventory and OpenAPI twin.; HTTP: served `/openapi.json` and admin routes on the running admin plane, matching the offline twin byte-for-byte.
EC Dimensions: behavior: `cargo test -p pgpool` - offline route inventory names the standard and pool admin endpoints; served-vs-offline conformance proven by `tests/admin_plane.rs`
Root WI: 1282
Status: auditing
Required Verification: smoke
Promise:
Publish pgpool's admin HTTP surface as a compact route inventory — standard
operational endpoints plus `/pools`, `/pools/{pool}/stats`, and `/drain` —
with the offline `pgpool spec` twin matching the served contract once the
admin plane runs.
Gate Inventory:
- apps/pgpool/src/spec.rs; apps/pgpool/tests/cli_contract.rs
- apps/pgpool/tests/admin_plane.rs (`served_contract_matches_offline_spec`, AC3)

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| offline-route-and-openapi-inventory | change | - | implemented | passing | smoke | apps/pgpool/src/spec.rs; apps/pgpool/tests/cli_contract.rs |
| served-contract-matches-offline-spec | epic | 1290 | implemented | passing | conformance | apps/pgpool/tests/admin_plane.rs (`served_contract_matches_offline_spec`); apps/pgpool/tech-design/logic/served-admin-plane-with-drain-aware-readiness.md |

### Kubernetes-Native Deployment

ID: kubernetes-native-deployment
Type: Devops
Surfaces: CLI: pending `pgpool k8s crd|operator|instance render` and `pgpool dockerfile render` - layered deployment artifact verbs per the service CLI convention.; K8s: pending PgpoolSpec CRD, operator, and instance profiles.
EC Dimensions: behavior: pending k8s render conformance gates - CRD/operator/instance artifacts render deterministically from the binary
Root WI: 1284
Status: candidate
Required Verification: conformance, dogfood
Promise:
Ship pgpool as a Kubernetes-native pooler: CRD/operator/instance render verbs,
image fixtures rendered from the binary, and pod lifecycle behavior (readiness
flip plus graceful drain) proven in a kind smoke path.
Gate Inventory:
- pending: k8s render conformance gates
- pending: kind drain/readiness smoke

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| crd-operator-instance-render | epic | 1284 | planned | planned | none | pending: render conformance gates |
| kind-drain-readiness-smoke | epic | 1284 | planned | planned | none | pending: kind smoke script |

### Long-Running Stability

ID: long-running-stability
Type: RuntimeTool
Surfaces: CLI: pending `pgpool` serve process - durable pooler with drain-aware shutdown.; TCP/HTTP: frontend admission and admin plane surviving backend restarts and rolling deploys.
EC Dimensions: stability: pending long-run and drain conformance gates - backend reuse without connection/fd leaks, drain without dropped in-flight transactions, restart safety
Root WI: 1282
Status: candidate
Required Verification: conformance, dogfood
Promise:
Run as a long-lived pooler without leaking backend connections or file
descriptors, dropping in-flight transactions on drain, or corrupting pool
state across backend restarts and rolling deploys.
Gate Inventory:
- apps/pgpool/tests/pool_modes.rs (`churn_100_cycles_holds_backend_count_stable_no_leak`); apps/pgpool/tests/pool.rs (`dropped_lease_without_explicit_release_does_not_leak_capacity_slot`) — bounded-cycle proof, not a true long-run soak
- pending: drain and backend-restart conformance tests

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| pool-leak-and-reuse-longrun | epic | 1289 | implemented | passing | conformance | bounded-cycle proof, not a true long-run soak: apps/pgpool/tests/pool_modes.rs (`churn_100_cycles_holds_backend_count_stable_no_leak`); apps/pgpool/tests/pool.rs (`dropped_lease_without_explicit_release_does_not_leak_capacity_slot`) |
| drain-and-backend-restart-safety | epic | 1289 | planned | planned | none | pending: drain conformance tests |

### Security Hardening

ID: security-hardening
Type: SecurityTool
Surfaces: TCP: PostgreSQL frontend auth passthrough - client credentials verified against the backend, never stored.; HTTP: admin plane exposure posture - probes stay tokenless, mutating admin verbs gated.; Env: pending TLS material configuration for frontend and backend links.
EC Dimensions: security: pending guard scan and negative gates - auth passthrough, TLS posture, malformed-frame rejection, admin exposure
Root WI: 1286
Status: candidate
Required Verification: negative
Promise:
Keep pgpool safe as a network-exposed credential-carrying proxy: auth
passthrough without credential persistence, explicit TLS posture on both
frontend and backend links, malformed wire-frame rejection, and a gated admin
plane before production readiness.
Gate Inventory:
- pending: vat guard-security runner
- pending: auth passthrough and malformed-frame negative tests

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| auth-passthrough-and-tls-posture | epic | 1286 | planned | planned | none | pending: negative gates |
| guard-static-runtime-evidence | epic | 1286 | planned | planned | none | pending: vat guard-security runner |

### Standard Operational Endpoints

ID: standard-operational-endpoints
Type: Service
Surfaces: HTTP: `/healthz`, `/readyz`, `/metrics`, `/openapi.json`, `/docs` - one-port operational surface, served on `RuntimePlan.admin_bind` via `http_server::serve_h2c_with_options`.; CLI: `pgpool spec` - offline OpenAPI evidence for the same contract when no server is running.
EC Dimensions: behavior: `cargo test -p pgpool` - offline inventory carries the five standard endpoints; served conformance proven by `tests/admin_plane.rs`
Root WI: 1282
Status: candidate
Required Verification: conformance
Promise:
Expose the standard one-port operational surface the service trait requires —
probes, metrics scrape, live spec, and Swagger UI stay always-on on the admin
port, with readiness flipping on drain and `pgpool spec` as the offline twin.
Gate Inventory:
- apps/pgpool/src/spec.rs
- apps/pgpool/tests/admin_plane.rs (`all_routes_respond_on_h2c_and_http1` AC1, `drain_flips_readyz_and_process_exits_cleanly` AC2, `metrics_exposes_prometheus_pool_gauges` AC4)

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| offline-standard-endpoint-inventory | change | - | implemented | passing | smoke | apps/pgpool/src/spec.rs |
| served-probes-and-drain-flip | epic | 1290 | implemented | passing | conformance | apps/pgpool/tests/admin_plane.rs (`all_routes_respond_on_h2c_and_http1`, `drain_flips_readyz_and_process_exits_cleanly`, `metrics_exposes_prometheus_pool_gauges`); apps/pgpool/tech-design/logic/served-admin-plane-with-drain-aware-readiness.md |
