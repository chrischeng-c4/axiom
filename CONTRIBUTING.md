# Contributing

> An agent or human should learn *what exists* and *where to act* from `ls`,
> paths, and filenames alone — without opening files. **Every file you don't
> open is a saved tool call, less context burned, and a more precise action.**

This repository is a multi-language ecosystem (Rust runtime + libraries, TS/UI,
Python conformance tests, specs, generated code, configs, docs, handoffs). Two
conventions run across all of it, both in service of the same goal — a tree an
agent can navigate cheaply and act on correctly:

1. **Authoring** — how to shape *any* artifact tree (files, paths, names) so it
   is legible from its structure alone.
2. **Ecosystem conventions** — the shapes every long-running **service** and
   every **CLI** repeats, so there is one stack to learn instead of one per
   project.

---

## Authoring principle: right-sized files, semantic paths, explicit names

This principle is **medium-agnostic** — it is about *navigability*, not any
language. Make the repository legible from its structure, so an agent can decide
where to act, and tooling can operate on the tree, without opening many files.

### The three rules

| Rule | What it asserts | Smell → fix |
|------|-----------------|-------------|
| **Right-sized files** | one coherent concern per file — one reason to exist, one reason to open. "Right-sized" ≠ "small": grain follows *access pattern* + *cohesion* (see *Balanced splitting*). | a file you open for several unrelated reasons → split |
| **Semantic paths** | the directory IS the taxonomy; the path conveys a file's role before you open it. Path ⇔ identity are mutually predictable. | you can't predict the path from what the file *is* → reclassify |
| **Explicit names** | the leaf name briefs the content; `ls <dir>/` reads as a table of contents. | a name that needs the body to grasp (`misc_cases`) → rename to the observable (`isleap_rule`) |

### Balanced splitting

Splitting must *earn its keep* — it pays off only when it improves navigation,
reviewability, reuse, or selective execution. Default to keeping.

```text
decide(file) →
  SPLIT  if ANY of:
    - a reader must search INSIDE it to reach one independent concern
    - the pieces are owned / reviewed separately
    - the pieces are executed / skipped / generated / compared independently
    - the resulting leaf names would form a useful table of contents
  KEEP   if ANY of:                                # cohesion outweighs file count
    - the pieces only make sense read together     # split → a cross-file puzzle
    - splitting would create trivial wrapper files
    - a shared setup dominates the content          # the setup, not the cases, is the file
    - the directory would only get noisier, not more discoverable
  DEFAULT → KEEP
```

Tie-break: a file needing internal headers between *unrelated* concerns wants
splitting; parts sharing one concept or one setup stay together (a single rule
over a few representative inputs → one file, as a table).

### Granularity scales with tooling

> Judgment call, called deliberately: the finer you split, the more files must
> stay mutually consistent — so **push granularity as fine as your tooling can
> keep consistent, no finer.**

```text
generated + linted  ⇒  go maximally atomic    # consistency is mechanical — you never hand-maintain the files
hand-authored       ⇒  stay coherent          # the consistency cost bites at scale — lean to cohesion
```

So a fully-tooled fixture tree goes maximally atomic (worked example below),
while hand-written source should not.

### Path grammar (a pattern, not a mandate)

```
<area>/<subject>/<concern>/<artifact>
```

- **area** — broad repo area: `tests`, `specs`, `configs`, `generated`, `docs`,
  `handoffs`, …
- **subject** — the module / feature / protocol / package / service / topic.
- **concern** — *the question this file answers, or its role*: behavior, errors,
  security, performance, integration, schema, api, migration, … (an open idea,
  not a fixed list — each tree names its own concerns).
- **artifact** — the specific case / scenario / generated unit / config concern /
  document.

Not every tree needs four levels — use the depth the tree earns. The same
grammar reads across media:

```
configs/auth/oauth_token_lifetime.yaml
specs/http/errors/malformed_header_rejected.md
generated/parser/ast/node_kinds.ts
handoffs/release/2026-05-rc1-risk-summary.md
tests/std-libs/calendar/behavior/isleap_rule.py     # worked below
```

### Organize by domain, not by tooling

Path grammar fixes the *axis* of a tree; this fixes its *first split*. Organize a
tree by the **capability domain** it covers, not by the runner or tool that
happens to execute it. The top level should tell an agent *what boundary each
subtree pins* before it opens anything — that is what turns an `ls` into a map
instead of a guess.

A test tree, for instance, splits by what it proves — never by test framework:

```text
tests/
├── <external-contract>/   parity / replacement contract against an oracle
├── <native-libs>/         your own library contracts (no external oracle)
├── <cli>/                 CLI / tool behavior, pinned on the built binary
└── governance/            meta-gates over manifests, profiles, CI policy, inventory shape
```

(`projects/mamba/tests/` is the reference adopter: `cpython/` pins the
CPython-replacement contract, `mambalibs/` the native-library contracts,
`pkgmgr/` the CLI, `governance/` the meta-gates. The same first-split-by-domain
shape applies to specs, configs, and generated trees.)

Three rules keep such a tree legible:

- **Entrypoints vs taxonomy.** A domain root holds only entrypoints and taxonomy
  directories; concrete cases live *below* the taxonomy
  (`<domain>/<subject>/<concern>/<artifact>`). A parse-only case belongs under
  that domain's fixtures, never dumped in `governance/`.
- **Wire deep artifacts explicitly.** Many build systems auto-discover
  entrypoints only at a fixed location (Cargo discovers top-level `tests/*.rs`),
  so artifacts nested under a domain must be reached through an explicit
  entrypoint or umbrella runner — not dropped as ad-hoc root files. Domain-local
  helper scripts stay under their domain (`<domain>/tools/regen_golden.py`, not
  `tests/regen_golden.py`).
- **Pair a manifest with its checker.** Where a gate is a manifest plus a
  checker, keep the two discoverably named and co-located so finding one implies
  the other.

**Migrating an existing tree toward this shape?** Lock the legacy form with a
gate that counts old-style monoliths as a *ceiling* — it can only stay flat or
shrink — so the debt is visible and one-directional while new artifacts adopt
the finer shape.

### Where it applies (scope)

Strongest for **naturally decomposable** trees — independent test fixtures,
config entries, generated units, doc/handoff files — where each artifact is
genuinely standalone and (ideally) tool-maintained. Applied with **judgment** to
cohesive hand-written code: there, one-concept-one-file can rightly *outweigh*
file count, and a language's idioms win (Rust `#[test]` fns stay in a
`mod tests`; a cohesive module groups related items). This is a guideline for
legible structure, not a mandate to shred cohesive code into wrapper files.

---

## Example: decomposing a monolith into a navigable tree

> One worked instantiation of the principle above — **not** the definition.

The clearest adopter is a fully-tooled conformance fixture tree, where a
generator emits the structure and a linter enforces it — so it goes **maximally
atomic** (one self-contained case per file) and the path is the grammar made
concrete: `<bucket>/<subject>/<dimension>/<case>`, where `dimension` is the
*concern* for tests (surface · behavior · errors · bench · real_world ·
security).

### Before → after

**Before** — one file, eight unrelated behaviors mixed together:

```
std-libs/calendar/behavior.py        # 8 cases, one big file
```

**After** — the concern is a directory; each case is a named leaf:

```
std-libs/calendar/behavior/
  isleap_rule.py                 # leap-year rule (a few representative years, one table)
  leapdays_counts.py
  monthrange_february.py
  setfirstweekday_roundtrip.py
  ...
```

`ls behavior/` is now the spec — a reader jumps straight to the one case they
need, and no coverage is lost. Note `isleap_rule.py` keeps its several input
years *together* as one table (cohesion), rather than one-file-per-input (which
would be over-splitting).

### Keep tooling and per-tree conventions with the tree

The mechanics that make a tree fully tooled — its layout spec, the
`generate → fill → lint` loop, the file template, and tree-local authoring
conventions (hermetic per-file headers, oracle verification, performance
baselines) — belong **with the tree**, not in this general guide. For the mamba
CPython suite that source of truth is
`projects/mamba/tests/cpython/conventions/FIXTURE-LAYOUT.md`: the six-dimension
table, the `fixture_gen` → fill → `fixture_lint` loop, PEP 723 `[tool.mamba]`
headers, the CPython oracle, and the perf-baseline flow.

---

## Service archetype: HA, HTTP/2 + OpenAPI, k8s-native

> The ecosystem's long-running network services share one shape. A new service
> of this kind — a broker, a store, an orchestrator, an index — **copies the
> archetype** rather than reinventing its transport, consensus, deployment, or
> gates. The wins are concrete: one transport stack to learn, one HA engine to
> harden, one set of gate files an agent can find by `ls` before opening
> anything.

Reference instantiations: **`keep`** (KV / claim-check store), **`relay`**
(broker), **`lumen`** (search / dedup index), and **`loom`** (workflow
scheduler). Planned service placeholders follow the same archetype:
**`tape`** (topic replay journal), **`defer`** (delayed task dispatch),
**`cube`** (OLAP service), and **`beam`** (GPU vector database).

Use these portfolio boundaries when creating TDs or assigning agents:

| Service | Owns | Does not own |
|---------|------|--------------|
| `loom` | workflow state, DAG scheduling, runner selection, timers, fair dispatch | broker delivery, payload bytes, replay archive |
| `relay` | online broker delivery, ordered log, broadcast fan-out, work-queue leasing | workflow decisions, long-term replay/archive, task HTTP dispatch |
| `keep` | KV/result storage, claim-check payloads, collections, durable values | broker delivery, workflow orchestration, analytical scans |
| `tape` | topic history, offset/time replay, consumer checkpoints, retention/backfill | online broker delivery, workflow decisions |
| `defer` | delayed HTTP task lifecycle, retry/DLQ, rate limits, dedupe keys | pub/sub fan-out, topic replay archive |
| `cube` | columnar facts, OLAP scan/filter/group-by/aggregate, rollups, partitions | search ranking, vector ANN, KV payload storage |
| `beam` | GPU vector indexes, vector ingest/rebuild, nearest-neighbor query | lexical/perceptual/duplicate search, OLAP aggregation |
| `lumen` | exact/lexical/semantic/perceptual/duplicate search in one service | OLAP aggregation, vector-only GPU DB ownership |

Capability-profile traits (`[capability.profile].traits` in a project's
`aw.toml`) derive a baseline of required capabilities from this archetype.
This table is generated from the `aw` CLI's trait registry — do not hand-edit
between the markers. Update `cli::doc_mirror::TRAITS` and run the relevant
meta-doc producer/check after a trait changes.

<!-- aw:trait-table:start -->
| Trait | Derives | Enforces | About |
|---|---|---|---|
| `http2_api` | `http2-api-list` | [Transport — h2c + OpenAPI on one port](#transport-h2c-openapi-on-one-port) | Project owes a public HTTP/2 (h2c) + OpenAPI transport baseline, not full OpenAPI completeness. |
| `kubernetes_native` | `kubernetes-native-deployment` | [Deploy artifacts — image, cluster API, operator, instance](#deploy-artifacts-image-cluster-api-operator-instance) | Project owes a Kubernetes-native deployment baseline: image, cluster API, operator, instance. |
| `primary_replicas` | `primary-replicas` | [HA — `raft-core`, sharded and strongly consistent](#ha-raft-core-sharded-and-strongly-consistent) | Project owes a primary/replica HA topology baseline; select only for projects that actually support that topology. |
| `standard_endpoints` | `standard-operational-endpoints` | [Standard endpoints — one operational surface, one contract three ways](#standard-endpoints-one-operational-surface-one-contract-three-ways) | Project owes the standard /healthz, /readyz, /metrics, /openapi.json, /docs operational surface on one port. |
| `ec_gated` | `ec-gates-configured` | [EC gates — `vat`-driven, evidence under `external-contracts/`](#ec-gates-vat-driven-evidence-under-external-contracts) | Project owes vat-driven EC gate files (vat.toml/meter*.toml/guard*.toml) with evidence under external-contracts/. |
| `cli_std` | `cli-standard-surface` | [CLI convention: every CLI ships `llm`, `upgrade`, `issue`](#cli-convention-every-cli-ships-llm-upgrade-issue) | Project owes the mandatory llm/upgrade/issue CLI surface every tool in the ecosystem ships. |
| `chainable_output` | `chainable-output-conformance` | [CLI convention: stdout tells the agent the next step](#cli-convention-stdout-tells-the-agent-the-next-step) | Project owes chainable stdout: every structured/terminal output carries a runnable next command or an explicit terminal marker. |
| `cli_facing` | `cli-interface` | — | Project is primarily driven through a CLI surface; no settled CONTRIBUTING.md doc home yet. |
| `competitive_replacement` | `competitor-feature-parity`, `competitor-performance` | — | Project aims to replace or match an existing competitor tool; no settled CONTRIBUTING.md doc home yet. |
| `long_running` | `long-running-stability` | — | Project runs as a long-lived process; no settled CONTRIBUTING.md doc home yet. |
| `network_exposed` | `security-hardening` | — | Project exposes a network-reachable surface; no settled CONTRIBUTING.md doc home yet. |
| `agent_facing` |  | — | Project is primarily driven by agents rather than humans; prompt-only, no enforced baseline capability yet. |
| `stateful_storage` |  | — | Project owns durable stateful storage; prompt-only, no enforced baseline capability yet. |
| `service` | expands: `http2_api`, `kubernetes_native`, `standard_endpoints`, `ec_gated`, `cli_std`, `chainable_output` | — | Umbrella for a full service-archetype adopter; expands to the transport, deploy, operational, EC-gate, and CLI baseline traits, deduped against any of its members also declared directly. |
<!-- aw:trait-table:end -->

### The shared service kit — compose these libs, do not hand-roll

*(policy-only — judgment, not trait-enforced)*

A service is mostly *wiring* shared libs together. **Adopting them is
mandatory** — a service that re-implements any of these is a defect, not a
variation. `lumen`, `keep`, `relay`, and `loom` may be at different adoption
stages, but new work moves them toward this common kit. If the common kit is
missing a hook, extend the shared lib first; do not fork the pattern into one
project.

| Lib | Role |
|-----|------|
| **`libs/raft-core`** | the step-driven raft **consensus core** (serde-only; replaced openraft). |
| **`libs/raft-host`** | the raft **host**: h2c peer transport, the single apply loop, **snapshot + log compaction** (the "backup layer"), read-your-write `propose`, and **k8s topology + auto-mode** (`cluster::ClusterTopology::from_env` + `replica_mode`, plus the reusable `ClusterDims`/`peer_ordinal`/`parse_peer_overrides` primitives — never re-derive the ordinal math locally). A service supplies a `RaftStateMachine` (`apply`/`snapshot`/`restore`/`applied_index`) and gets HA + backup for free. Also ships the read-side companions: the `X-Read-Consistency` header contract (`ReadConsistency`), the `RaftRole`/cluster-view introspection model, and `OutcomeWindow` (the bounded index→outcome window behind rich read-your-write results). |
| **`libs/operator`** | the **k8s operator scaffold + render toolkit**: `ManagedService`, `ClusterSpec`, `ResourceSpec`, owner refs, labels/selectors, ServiceAccount, client/headless Services, PDB, CronJob, and `sharded_statefulset` with the exact downward-API env that `raft-host` reads — generalized by `service_statefulset`/`ServiceStatefulSet`, the configurable service-workload StatefulSet primitive (service-supplied probes, security contexts, extra volumes/mounts, update strategy — generic JSON pass-throughs, so e.g. a CSI volume needs no lib change) that per-service operators compose while preserving that env contract — plus `resize` (k8s storage-quantity parsing + live-PVC expansion around immutable `volumeClaimTemplates`). |
| **`libs/h2c`** | the **transport**: `h2c::serve` (server, feature `server`) + `h2c_client`/`H2cPool` (client). |
| **`libs/service-http`** | the **HTTP service shell**: standard probe/admin routes, tracing init, graceful drain, metrics/readiness hooks, h2c serving composition, and the shared **HTTP error envelope** (`ErrorEnvelope` + the `ApiErr` status/kind builder) so error JSON is uniform across services. |
| **`libs/service-auth`** | the **request-auth shell**: shared `Authorization: Bearer` extraction, reject/inject middleware, the `Verifier` trait every service implements, and **`role_map`** — the standard token-registry verifier (`Role` hierarchy, `TokenClaims` with wildcard grants, registry-file loader, `StaticRoleMapVerifier`) implementing the archetype's `<SVC>_TOKEN_REGISTRY_FILE` contract. Token crypto belongs in **`libs/claimtoken`** when signed tokens are needed; resource-policy *decisions* stay in the service handlers (`role_map` supplies the mechanism). |
| **`libs/service-backup`** | the **backup contract**: destination/policy schema, `BackupSink`, local + S3-compatible object-store sinks (feature `s3`; GCS destinations parse/round-trip but runners fail loudly until a real GCS adapter lands), and a runner primitive. Services produce consistent snapshot bytes; runners upload them; operators schedule/manage the runner. |
| **`libs/service-tls`** | **peer mTLS material loading**: `PeerTlsConfig::from_env(<PREFIX>)`, PEM cert/key/CA loaders, rustls server/client config builders, and the Once-guarded default-crypto-provider install. (h2c stays cleartext by design; this covers the mutually-authenticated peer/replication port.) |
| **`libs/service-metrics`** | the **metrics registry**: dep-free counter/gauge primitives + the Prometheus text-format encoder — the standard implementation behind `service-http`'s `MetricsProvider` seam and the `/metrics` endpoint. |
| **`libs/cli-std`** | the **standard CLI** commands (`llm` / `upgrade` / `issue`). |
| **`libs/build-stamp`** | the **build stamp** (a `[build-dependencies]` crate): `stamp("<PREFIX>")` emits the `<PREFIX>_GIT_SHA` / `<PREFIX>_BUILT_AT` / `<PREFIX>_TARGET` rustc-env lines that feed `cli-std`'s `ToolInfo` — one implementation instead of a per-service `build.rs` copy. |

**k8s-native auto-mode + discovery.** A service defaults to single-node and turns
on raft **only when the StatefulSet scales out** — `raft_host::cluster::
replica_mode()` is `true` when `REPLICAS_PER_SHARD > 1` (a downward-API value). So
`<svc> serve` needs **no flags or cluster env** for local/single-node dev; k8s
scaling flips it to replica mode automatically, with node id / membership / peers
derived from the downward API by `ClusterTopology::from_env` (a local
`<SVC>_PEERS=host:port,…` override runs a multi-node group on one machine). Do not
hand-roll the pod-ordinal or peer-DNS math.

**Operator/render convergence.** The Kubernetes topology that drives that
auto-mode is also shared. A service CR should flatten or mirror
`operator::ClusterSpec`/`ResourceSpec` unless it has a concrete product reason
not to, implement `operator::ManagedService`, and render shared shapes with
`libs/operator::render` helpers. In particular, StatefulSet identity,
`SHARD_COUNT`, `REPLICAS_PER_SHARD`, `VOTER_COUNT`, headless-service env,
labels/selectors, owner refs, PDB/client/headless Service shapes, and
maintenance CronJobs are library contracts. Do not duplicate that YAML/JSON
construction in `lumen`, `keep`, `relay`, or `loom`; extend `libs/operator`
when the helper is incomplete.

A service is not "done" until it satisfies every row:

| Dimension | Requirement | Reference / gotcha |
|-----------|-------------|--------------------|
| **Shape** | Workspace member that is **both `lib` and `bin`** — embeddable as a crate, runnable as a server. Metadata via `version/edition/authors/license = .workspace`. | every service `Cargo.toml` |
| **Transport** | HTTP/2 cleartext (**h2c**) **+** HTTP/1.1 on **one port**, with an OpenAPI surface (`utoipa`). | Compose **`libs/service-http`** and **`libs/h2c`** — built on `hyper-util` `auto::Builder`, **not `axum::serve`** (HTTP/1-only). The same crate's client side (`h2c_client`/`H2cPool`) is the in-tree client. |
| **Standard endpoints** | The same operational surface on the one port: **`/healthz`** (liveness), **`/readyz`** (readiness), **`/metrics`** (Prometheus), **`/openapi.json`** (machine OpenAPI), **`/docs`** (Swagger UI). Probes + scrape **depend** on these, so they stay auth-exempt and always-on. | Prefer **`libs/service-http`** standard probe/admin route helpers. `lumen` is the reference for the full surface. The contract is reachable three ways — **`<cli> spec`** (offline) ≡ **`/openapi.json`** (served) ≡ **`/docs`** (browsable) — one OpenAPI, three access paths. |
| **Auth** | Every service uses the same bearer-token shape: server env `<SVC>_AUTH=off|required` plus `<SVC>_TOKEN_REGISTRY_FILE=/var/run/secrets/<svc>/token-registry.json`; clients use `<SVC>_URL` + `<SVC>_TOKEN` and send `Authorization: Bearer <token>`. | Compose **`libs/service-auth`** for middleware and **`libs/claimtoken`** for signed-token verification when needed. In k8s/cloud, the registry file is mounted from a Kubernetes Secret, CSI Secret Store, or cloud Secret Manager sync. Do not add one-off auth headers, per-service token env names, or inline server token lists as the production path. |
| **OpenAPI client codegen** | Generate typed clients from the service's **own** OpenAPI via **`libs/openapi-codegen`** (`cclab-openapi-codegen`) — **never** hand-rolled or an external tool. Expose it on the CLI: `<cli> spec gen --lang ts\|py\|rust --out <dir>`. Adopters get a typed client with **no external codegen step**. | `lumen spec gen` is the reference; the polyglot core (ts/py/rust) was extracted so any CLI composes it. |
| **HA / consensus** | **Mandatory for any stateful service:** sharded, strongly-consistent state replicated with **`libs/raft-core`** driven by **`libs/raft-host`** — the replication path **wired** (a `RaftStateMachine` impl), not a DTO-only / "later slice" stub. Follower tails the leader over h2c; snapshot/compaction comes from the host. | Use `raft-core`+`raft-host`, **not `openraft`** and **not** a hand-rolled driver. The raft path may be a Cargo feature (`keep`); `lumen` is the reference adopter (`EngineSm`). |
| **Backup / restore** | Stateful services expose consistent snapshot/restore from their state machine and use **`libs/service-backup`** for destination/policy/sink/runner shape. The operator may render a CronJob/Job and secrets/IAM wiring, but it never serializes service data itself. | `raft-host` owns snapshot install + log compaction. The service admin/CLI produces snapshot bytes; the backup runner uploads to local/S3/GCS; the operator schedules and reports status. |
| **Core neutrality** | Keep domain/payload knowledge **out of the transport core** where feasible, so the core is reusable. | `relay` carries an opaque JSON body and "knows nothing about workflows" (#120). |
| **Deploy** | `Dockerfile` (+ `.release` / `.bench` variants); `<cli> dockerfile render`; **k8s-native** kustomize tree (`k8s/base` + `k8s/overlays`); `<cli> k8s crd/operator/instance`; StatefulSet identity/peers from the **downward API**; dedicated/standalone data-plane mode as the production baseline; an `HA.md`. | Use **`libs/operator`** for CR/operator/render shape. `keep/k8s`, `lumen k8s` (+ `operator` feature), `relay/k8s`, and `loom/deploy` are adoption surfaces; when they differ, converge them toward the shared kit instead of copying local YAML. Shared multi-tenant backends are optional platform work, not the default service archetype. |
| **SDD-managed** | `aw.toml` + `tech-design/` + `SPEC-MANAGED` / `HANDWRITE` markers in source. Drive changes through the `aw` lifecycle. | see the SDD rules in `CLAUDE.md`. |
| **EC gates** | Evidence-contract gates wired below. | see *EC gates* next. |
| **CLI** | The bin ships `llm` / `upgrade` / `issue`. | see the *CLI convention* below. |

### Transport — h2c + OpenAPI on one port

`axum::serve` speaks HTTP/1 only. To serve h2c (HTTP/2 cleartext, no TLS — the
in-cluster default) alongside HTTP/1.1 on a single port, build the connection
with `hyper-util`'s `auto::Builder`. In-tree clients are `reqwest` over h2c
(rustls, no openssl). Describe the surface with `utoipa` so the OpenAPI doc is
generated from the handlers, never hand-maintained.

### OpenAPI client codegen — typed clients from the spec

*(policy-only — judgment, not trait-enforced; codegen-from-spec discipline
for a service's clients, not itself a separate baseline capability — enforced
transitively via `http2_api`'s `http2-api-list` baseline, which already
requires the service's own OpenAPI surface to exist)*

Because the OpenAPI doc is the source of truth, the typed clients adopters use
are **generated from it**, never hand-written and never produced by an external
tool. The shared `libs/openapi-codegen` (`cclab-openapi-codegen`) is the polyglot
core — a language-neutral IR feeding per-language emitters (TypeScript: types +
fetch/axios client + TanStack Query hooks; Python: pydantic + generated
sync/async HTTP/2 runtime that speaks h2c for `http://` and ALPN h2 for
`https://`; Rust: serde + reqwest). A service **composes** it
behind a CLI verb:

```
<cli> spec gen --lang ts|py|rust --out <dir>
```

so an adopter goes from "the service is up" to "a typed client in my language"
with no external codegen step. Reference: `lumen spec gen` (feeds the binary's
own `openapi_json()` into `cclab_openapi_codegen::generate`). Do **not** add a
second codegen path — extend the shared crate (a new emitter / capability) so
every service benefits.

### Deploy artifacts — image, cluster API, operator, instance

Image construction is not a Kubernetes subcommand. Every k8s-native service CLI
ships:

```
<cli> dockerfile render --variant source|release [--version <tag>] [--out <path-or-dir>]
```

`source` renders the workspace-build Dockerfile used by dev/CI; `release`
renders the small production image that fetches a published `<project>@<version>`
binary. The same image artifact feeds compose, kind, and real registries.

Kubernetes output is split by lifecycle layer:

```
<cli> k8s crd render [--out <path>]
<cli> k8s operator render [--namespace <ns>] [--out <path-or-dir>]
<cli> k8s operator run
<cli> k8s instance render --profile dev|staging|prod|template [--out <path-or-dir>]
```

`crd` owns the cluster-scoped API. `operator` owns the control plane, normally in
an independent namespace such as `<svc>-system`; `operator run` is the controller
process/container entrypoint. `instance` renders the app-namespace custom
resource that an application team applies next to the app it integrates with.

### Deploy tenancy — dedicated first, shared only when justified

*(policy-only — judgment, not trait-enforced)*

The service archetype is **dedicated-first**. A stateful service must be able to
run as its own data plane — one service instance, one app namespace or
service-owned namespace, one StatefulSet/Deployment, one storage/backup surface,
and one operational SLO envelope. This dedicated/standalone mode is mandatory
because it is the simplest reliable production shape: ownership, upgrades,
backups, failure blast radius, and delete/finalizer behavior are all local to the
service instance.

Shared multi-tenant backends are a separate platform capability, not the default.
They are appropriate only when the product explicitly needs many small tenants to
share a physical data plane and the platform is ready to own the extra control
loops. A shared mode requires placement, metering, quota/rate-limit enforcement,
tenant identity, backend capacity accounting, promotion/demotion policy,
migration, endpoint/secret rotation, backup partitioning, and finalizer semantics.
Those moving parts materially increase operational complexity, and most service
deployments do not need them.

When a shared mode exists, keep the API resource paths service-domain scoped, not
Kubernetes-namespace scoped. Namespace is deployment and service-discovery
context; it belongs in DNS, RBAC, CR metadata, and status endpoints, not in HTTP
paths. For example a tenant may reach
`http://lumen.lumen-shared.svc/collections/users/index`, while the HTTP route
remains `POST /collections/{collection_id}/index`.

Use this default decision rule:

| Mode | Default? | Use when |
|------|----------|----------|
| **Dedicated / standalone** | Yes | Most production services, high isolation, simple ownership, clear backup/restore and SLO boundaries. |
| **Shared backend** | No | A platform team explicitly owns placement, metering, quotas, migration, endpoint switching, and tenant lifecycle. |
| **Promote to dedicated** | Optional | A shared tenant exceeds sustained usage, SLO, storage, or isolation thresholds and migration is controlled by policy. |

### Service dogfood rules — keep the whole surface honest

*(policy-only — judgment, not trait-enforced)*

Recent service hardening work exposed a repeated failure mode: one slice moves to
the new archetype while build scripts, CRDs, k8s overlays, tests, or EC gates
still point at the old backend. Treat the following as contract, not cleanup
advice:

- **One active data plane.** A service may keep legacy compatibility code only
  when it is explicitly labelled legacy and still tested. The production build,
  Dockerfiles, release workflow, `build.sh`, k8s manifests, operator render
  path, examples, README/HA docs, perf modes, and EC gates must all name the
  same active data plane. Retired backends must not remain in active gates.
- **Direct install is not the HA story.** Kustomize `base` / overlays should be a
  small direct install, usually single-node/embedded for kind and smoke tests.
  Production HA goes through the operator CR path, which renders the StatefulSet
  topology and downward-API env that `raft-host` consumes.
- **Operator owns lifecycle, not bytes.** The operator creates RBAC,
  ServiceAccounts, Services, StatefulSets/Deployments, PDBs, CronJobs, Secrets,
  status, and finalizers. It does not serialize service data. Snapshot bytes are
  produced by the service state machine/admin surface; `raft-host` installs
  snapshots and compacts logs; `libs/service-backup` runners upload/prune them.
- **Namespace is deployment context.** The service instance CR lives with the app
  namespace unless a platform team intentionally separates app and backend
  lifecycle. Operator/control-plane resources normally live in `<svc>-system`.
  HTTP paths stay service-domain scoped; do not add Kubernetes namespace names to
  public routes.
- **CRDs must be Kubernetes OpenAPI compatible.** Generated CRDs are not done
  until `kustomize build` and CRD render pass. Normalize schema details that
  Kubernetes rejects, such as `format: uint32` / `format: uint64`; express
  unsigned integer intent as `type: integer` plus `minimum: 0`.
- **Rustls provider is binary startup work.** Any binary that links rustls-backed
  clients, including operator mode or raft/online CLI paths, installs the
  process-level crypto provider before parsing commands or starting async work.
- **Kind gates exercise the current service, not retired dependencies.** A
  service dogfood script must not build or deploy a retired external component
  just because old manifests still mention it. If the current kind gate is
  intentionally single-node, say so and keep replay/HA claims in separate gates.
- **Peer-service benchmarks are calibration, not every-run work.** Once
  Postgres/OpenSearch or similar competitor baselines are captured, regular
  production gates should run the service-only regression path against retained
  floors. Rerun peers only for explicit recalibration or new comparison rows.

### Standard endpoints — one operational surface, one contract three ways

Every service exposes the same five endpoints on its one port, so an operator,
a probe, a scraper, or an agent finds the same surface without per-service
lookup:

```
/healthz       liveness  — k8s livenessProbe
/readyz        readiness — k8s readinessProbe (gates traffic)
/metrics       Prometheus exposition — the ServiceMonitor scrape target
/openapi.json  the machine-readable OpenAPI (utoipa-generated)
/docs          Swagger UI — the human-browsable render of that OpenAPI
```

`/healthz` and `/readyz` are **not optional**: the StatefulSet/Deployment the
operator renders points its probes at them, and `/metrics` is the scrape target —
so all three stay **auth-exempt and always-on** (a 401 on `/healthz` fails the
probe). They are the intersection of the *Transport* and *Deploy* rows.

The integration contract is reachable **three equivalent ways** — same OpenAPI,
different access path:

| Path | Context |
|------|---------|
| **`<cli> spec [--format openapi\|openapi-yaml\|json-schema] [--shapes] [--fields]`** | **offline** (no server) — for agents, CI, codegen input. The offline twin of `/openapi.json`; `spec gen` is its codegen sub-verb. |
| **`/openapi.json`** | served, machine-readable |
| **`/docs`** | served, human-browsable (Swagger UI) |

So `spec` belongs to **this** archetype (a service has an OpenAPI to emit), **not**
`cli-std` — which carries only the universal `llm`/`upgrade`/`issue` that *every*
CLI ships, services and non-services alike. `lumen` is the reference for all of it.

### Service auth — one Bearer-token contract

*(policy-only — judgment, not trait-enforced; candidate for a future
`service_auth` trait once `libs/service-auth` adoption is mechanically
checkable)*

Service auth is shared infrastructure, not a per-project design space. Every
long-running service uses `libs/service-auth` for request authentication:
extract `Authorization: Bearer <token>`, verify it through a service-supplied
`Verifier`, reject with the shared JSON error shape, and inject the authenticated
principal into handlers. Services use the shared registry verifier
(`service_auth::role_map::StaticRoleMapVerifier` — role hierarchy, wildcard
grants, registry-file loader) or signed tokens through `libs/claimtoken`, but
the HTTP contract and middleware shape stay the same.

Production server config follows one env pattern:

```
<SVC>_AUTH=required
<SVC>_TOKEN_REGISTRY_FILE=/var/run/secrets/<svc>/token-registry.json
```

Local/dev may use `<SVC>_AUTH=off`. Production must not depend on inline token
lists as the primary path. In Kubernetes or cloud-native deployments, the
operator/render layer mounts the registry file from a Kubernetes Secret, CSI
Secret Store, or cloud Secret Manager sync, and owns the env wiring. Clients do
not need connection strings for auth; they use `<SVC>_URL` for routing and
`<SVC>_TOKEN` for credentials, then send `Authorization: Bearer <token>`.

Keep the boundary explicit: `libs/service-auth` authenticates callers;
service handlers authorize per resource, tenant, collection, queue, workflow, or
admin action. Standard probe/spec/scrape endpoints stay auth-exempt according to
the standard-endpoint contract above.

### HA — `raft-core`, sharded and strongly consistent

**HA is mandatory for any stateful service — not a "wire it later" slice.** A
DTO / cluster-state surface (`/debug/cluster`) without the `raft-core` replication
path actually wired does **not** satisfy the HA row; the service is not
production-ready until writes are ordered and replicated through `raft-core`.

State is **sharded** and **strongly consistent**, replicated by the shared
`libs/raft-core` engine (serde-only; it replaced `openraft` across the
ecosystem). The leader owns writes; followers tail it over h2c. Node identity
and the peer set come from the Kubernetes **downward API** on a StatefulSet —
nothing is hand-configured per replica. Gate consensus behind a Cargo feature
only when a single-node mode is a legitimate deployment (e.g. `keep`).

### EC gates — `vat`-driven, evidence under `external-contracts/`

Every service carries a fixed set of **evidence-contract (EC) gate files**, each
`SPEC-MANAGED` and pointed at a contract under `external-contracts/`. They are
named so `ls` tells you what is enforced before you open anything:

- **`vat.toml`** — the EC test runner; backs integration tests with **real
  services / emulators** (never hand-rolled mocks) and lists the setup steps.
- **`meter*.toml`** — performance/efficiency/stability EC gates
  (`meter.toml` + `meter-<scope>-<dimension>.toml`, e.g.
  `meter-keep-performance.toml`, `meter-search-{efficiency,stability}.toml`),
  run via `vat run meter-*`; evidence under
  `external-contracts/competitor-performance/`.
- **`guard*.toml`** — the security EC gate (`guard-<scope>-security.toml`),
  run via `vat run guard-security`; evidence under
  `external-contracts/security-hardening/`.

A breach is a non-zero-exit finding that blocks the terminal
`aw td code-check <slug>` gate. Keep these files `SPEC-MANAGED` — regenerate
them from their contract; do not hand-edit the `AW-EC-TOOL` block.

## CLI convention: every CLI ships `llm`, `upgrade`, `issue`

> Every binary a human or agent runs must answer three questions without prior
> knowledge: *how do I drive this?* (`llm`), *am I current?* (`upgrade`), and
> *what's broken — what's already reported, and how do I file it?* (`issue`).
> These three are
> **mandatory** on every CLI surface in the ecosystem (`mamba`, `jet`, `lumen`,
> `vat`, `aw`/`cclab`, and any new tool) — the agent-facing contract that lets a
> tool an agent has never seen self-onboard, self-update, and file a structured
> defect using the binary alone.

A new CLI is not "done" until all three appear in `--help`.

**Positionals are for subcommands and a verb's one primary object; structured
parameters are flags.** A positional names a *subcommand* (`jet build`, `jet
issue search`) or the verb's single natural object — an id, a query, or free-form
prose (`issue view <n>`, `issue search [query]`, `issue create [msg…]`). Anything
the command *selects or configures* — a topic, title, version, tag, state — is a
named flag (`--topic`, `--title`, `--version`, `--state`), so the grammar stays
unambiguous as the surface grows.

| Subcommand | Signature | Contract |
|------------|-----------|----------|
| `llm` | `<cli> llm [--topic <topic>] [--format md\|json]` | Offline (no server/network) docs that teach an agent to drive the tool. Topic via `--topic` (not positional); default `outline` (a topic map); per-tool topics follow its domain. Markdown default, `--format json` for machine-readable. |
| `upgrade` | `<cli> upgrade [--version <tag>] [--check]` | Self-update to the latest `<project>@*` GitHub release. `--check` = report whether newer exists, no install; `--version` = pin a tag. |
| `issue` | `<cli> issue search [query]` · `view <n>` · `create [--title <t>] [msg…]` | Read **and** write the tool's issues on the tracker. `search` finds this tool's issues (filtered to `project:<name>`; omit the query to list recent), `view <n>` prints one, `create` files a structured issue (auto-attaching `--version` + OS/arch + context, tagged with the `project:<name>` label). |

`llm` is a **cross-scope agent index**, not a feature-local doc page. It must
teach the smallest command/topic an agent should read across the tool's real
operating scopes: CLI grammar, workflow, API/spec, auth, deployment,
operator/k8s, storage/backup, data movement, integration adapters, diagnostics,
and domain recipes. When a change alters any public command, generated
artifact, API contract, operator/CRD field, deployment topology, auth model,
backup/import/export path, or issue/upgrade flow, the change must either update
the relevant `llm --topic ...` body or explicitly verify that the existing topic
already teaches the new behavior. Treat a stale `llm` topic as a public CLI
regression.

`llm` topics should be assembled from one in-code topic registry, not from
duplicated hand-written command lists. The default `outline` is generated from
that registry and is the cross-scope table of contents; detail topics own their
domain body. Tests should lock both levels: the outline advertises every topic
with the convention-canonical `--topic <id>` flag, and every advertised command
parses through the real binary. For services, add focused topic assertions for
the scopes the service exposes (for example deployment topology, storage/backup,
auth, integration, and query recipes) so cross-scope drift is caught by the same
patch that introduces it.

Internal libraries may contribute agent-facing `llm` fragments, but they do not
become standalone user-facing CLIs just to publish docs. A reusable lib with an
operational contract agents need to understand (for example `libs/operator`,
`libs/raft-host`, `libs/service-backup`, `libs/service-auth`, `libs/h2c`, or
`libs/openapi-codegen`) should expose a small `cli_std::llm::Topic` provider or
constructor from its Rust API. The consuming project decides whether that topic
belongs in its own `llm` registry, may wrap or prefix the id to fit the
project's vocabulary, and may omit irrelevant library topics. This keeps the
source of shared behavior close to the library while keeping the final
agent-facing outline scoped to the actual tool the agent is driving.

Library-contributed topics must describe the library contract, not the consuming
project's product policy. Project topics own final choices such as "Lumen uses
three raft voters" or "this CRD exposes `spec.serving.backup`"; library topics
own reusable mechanics such as raft ordinal math, backup destination semantics,
auth registry shape, or operator render layers. When a library contract changes,
update its topic provider and at least one consuming project's `llm` regression
test if that project exposes the contract.

Implementation notes not obvious from the signature:

The logic for all three lives in the shared **`libs/cli-std`** crate (`cli_std`),
which is **clap-agnostic**: each CLI keeps its own clap registration — so it owns
the convention's flag shape (`--topic`, not a positional) — and delegates the
behavior to the crate, parameterized by a `cli_std::ToolInfo` it fills from its
own `build.rs` stamps (project, repo, target triple, version, git sha — emit
them with `libs/build-stamp`'s `stamp("<PREFIX>")`, not a hand-rolled
`build.rs`). A tool
provides only its clap surface, that `ToolInfo`, and (for `llm`) its topic list;
the crate does the rest. The network paths (`upgrade` install, `issue`
search/view/create) sit behind cli-std's `online` feature — enable it in release
builds. Reference adopters: `projects/jet` and `projects/lumen`.

- **`llm`** — `cli_std::llm::render(project, version, topics, topic, format)`. The
  tool supplies `&[cli_std::llm::Topic]` (`id`/`summary`/`body` — the one in-code
  source of truth) and cli-std renders the `outline` topic map + the
  standard-command footer. Pure offline; always builds.
- **`upgrade`** — `cli_std::upgrade::run(&tool, opts)`: the in-binary form of
  `projects/<project>/install.sh` — detect target (`<arch>-<os>`) → download the
  matching `*.tar.gz` → verify `.sha256` → **atomically** replace the binary.
  Fail loudly on checksum mismatch; never leave a half-written binary.
- **`issue`** — `cli_std::issue::{search, view, create}`. `search`/`view` are
  read-only GitHub API GETs (tokenless on public repos); `create` submits via the
  API when `GITHUB_TOKEN` is set, else prints a pre-filled `issues/new` URL. Pass
  the tracker's `project:<name>` label in `CreateOptions.label` so it is applied
  on submit **and** carried into the URL fallback's `&labels=`; `search` filters
  to that same label. The group is named `issue` (**not** `report`), leaving
  domain `report` verbs (`jet report` = HTML **test** reports) untouched.

## CLI convention: stdout tells the agent the next step

> Every CLI's machine-readable output must tell the agent what happens next —
> a runnable command, or an explicit "done" — so the agent never has to guess
> **what now?** after a command finishes.

A CLI's structured/terminal output (its JSON payload or fixed final stdout
line — not incidental logs) **MUST** carry either (a) `next` — a command
string the agent can execute verbatim, or (b) an explicit terminal marker
meaning "done — report completion to the user." There is no third state: an
output that is neither runnable nor terminal is a defect. Errors **MUST**
carry a remediation next step too; the runnable-or-terminal rule applies on
the failure path exactly as it does on success.

Emitted commands **MUST** be executable, not aspirational: multi-level verbs
must exist on the real clap surface and chain-required positionals/flags must
be present. A `next` that doesn't resolve against the binary's own `--help`
is worse than no `next` at all.

The reference implementation is `aw`'s `aw.cli.v1` envelope
(`apps/agentic-workflow/src/runtime/envelope.rs`): `invoke.command` names
the command an agent runs next, `next.command` carries the workflow loop's
next step, and `completion.workflow_complete=true` is the only terminal
marker — `action=done` on one child root is not workflow completion by
itself. Executability is enforced, not just documented:
`apps/agentic-workflow/src/cli/chain.rs`
(`validate_aw_command_string` + the `EMIT_REGISTRY` catalogue of every
command-emitting site) re-parses each emitted string through the real
`Commands` clap tree plus a chain-policy table of positionals that are
clap-optional but semantically required. This is chain-conformance tier 1a
(aw only, today); per-CLI rollout WIs cite this section and its enforcement
pattern as their acceptance contract.

Simple CLIs without a full envelope **MAY** use a lighter conforming form —
a single `next` field in JSON output, or a fixed trailing stdout line. The
wire format is free; the runnable-or-terminal semantics are not.

`aw health` and this convention measure different things. `aw health` asks
whether a project is *complete* (是否做完 — capability/managed/semantic/
traceability coverage); this convention asks whether one command's output is
*executable by the next agent* (轉不轉得動 — handoff, not completeness). A
CLI can be 100% complete by `aw health` and still emit unchainable output,
and vice versa.

## Meta-doc content contract

> Every doc fact carries exactly one of: a generator (a projection with a
> named source), a validator (a contract or scanner that fails when the fact
> goes stale), or an explicit policy-only marker. A fact restated only in
> prose — none of the three — rots the first time reality moves, because
> nothing catches the drift.

Meta docs split by *layer*, not by topic. A fact belongs to exactly one layer.

**Repo/global layer** is for facts that apply across the repository or teach an
agent how to operate in this checkout. The only live root meta docs are:

| Doc | Ownership |
|-----|-----------|
| `README.md` | repo identity, generated Projects table, install/discovery entrypoint |
| `CONTRIBUTING.md` | repo-wide authoring contracts, CLI conventions, service archetypes, meta-doc taxonomy |
| `CLAUDE.md` | repo-level agent operating manual for Claude Code |
| `AGENTS.md` | repo-level agent operating manual for Codex; generated/mirrored from `CLAUDE.md` plus Codex-only inserts |
| `LICENSE` | legal license text; the only root uppercase meta file without a `.md` extension |

**Project/app layer** is for one deliverable's product contract and scoped
local conventions. `apps/<name>` is for app-facing binaries/services; legacy
and library-like deliverables may still live under `projects/<name>`. Allowed
project/app-layer meta docs are:

| Doc | Ownership |
|-----|-----------|
| `apps/<name>/README.md` or `projects/<name>/README.md` | fixed orientation shell: `## Brief`, `## Contributing`, and `## Capability Contract`; the latter two include a short brief extracted from the linked project/app-layer docs |
| `apps/<name>/CONTRIBUTING.md` or `projects/<name>/CONTRIBUTING.md` | local authoring, verification, migration, and contribution rules that are too specific for this repo-wide file; must include `## Brief` |
| `apps/<name>/CAPABILITIES.md` or `projects/<name>/CAPABILITIES.md` | capability contract using the canonical capability schema; must include `## Brief`; README links here rather than restating the full contract |
| scoped convention docs | only when placed next to the tree they govern, e.g. test fixture layout rules under the fixture tree |
| generated evidence/docs | only when backed by an explicit producer, validator, or policy-only marker |

Project-layer docs are written for agents first: they should answer "what is
this project promising?", "where is the source of truth?", "what am I allowed
to edit?", and "how do I prove the change?" without making the agent read a
full design book.

`apps/<name>/CAPABILITIES.md` / `projects/<name>/CAPABILITIES.md` is the
product contract. It is hierarchical:
a top-level capability may be a product area, a narrower capability may be a
feature or surface, and the smallest useful capability may be one API endpoint,
CLI command, event, background job, or documented behavior if it can be
implemented and verified independently. Do not flatten agent-addressable
endpoint/command promises into prose under a larger heading when a future WI,
TD, EC, or test gate will need to refer to them directly.

The required shape is:

- `# <Project> Capabilities`
- `## Brief` — one to three sentences, copied into the project README's
  `## Capability Contract` section.
- `## Capabilities`
- `### Capability Index` — compact scan table for every release-significant or
  agent-addressable capability.
- `### <Capability>` roots — each root uses field-style lines for `ID`,
  `Type`, `Surfaces`, `EC Dimensions`, `Root WI`, `Status`,
  `Required Verification`, `Promise`, and `Gate Inventory`, followed by a
  `Work Root` table for child WIs, TDs, EC gates, tests, and evidence.

Large capabilities should own child work roots or nested capability headings;
small leaf capabilities can have one work-root row. The sizing rule is
independent verification: if an agent can build, test, or close it separately,
it is allowed to be a capability. Private implementation details stay out
unless they are user-visible, release-blocking, or needed as named evidence.

`apps/<name>/CONTRIBUTING.md` / `projects/<name>/CONTRIBUTING.md` is the local
operating guide. It does not restate root authoring rules and does not carry
product promises that belong in CAPABILITIES. It explains how an agent safely
changes this deliverable. The required shape is:

- `# <Project> Contributing`
- `## Brief` — one to three sentences, copied into the project README's
  `## Contributing` section.
- `## Authoritative Inputs` — ordered source-of-truth list for product,
  behavior, generated code, external contracts, and local conventions.
- `## Local Workflow` — the commands, generators, lifecycle routes, and edit
  boundaries an agent should use for this project.
- `## Verification` — exact local checks, test gates, service dependencies,
  skip rules, and evidence commands needed before claiming completion.

Optional sections are allowed when they save an agent from guessing:
`## Architecture Boundaries`, `## Data and Services`, `## Migration and
Compatibility`, `## Release`, and `## Meta Docs`. Keep them scoped to this
project. If a rule applies to every project, move it back to this root
CONTRIBUTING instead.

`CLAUDE.md` and `AGENTS.md` are **not** project-layer docs. Project-specific
agent behavior must be expressed through the project
README/CONTRIBUTING/CAPABILITIES contract, scoped convention docs,
skills/templates, or command output. Template sources may contain a
`CLAUDE.md` filename when they are producer inputs, but they are not live
project-layer meta docs.

Three rules govern every cell above:

- **Single-level rule** — content lives at exactly one level; every other
  level links to it, never restates it.
- **Fact-ownership rule** — a fact with no generator, no validator, and no
  policy-only marker is a defect, not a stylistic choice; add whichever of
  the three fits before landing the fact.
- **Level heuristic** — "how do all projects do X" belongs in CONTRIBUTING;
  "what does this project promise" belongs in its own README/CAPABILITIES;
  "how does an agent operate here" belongs in CLAUDE/AGENTS; "what exists"
  belongs in the repo-root README.

Enforcement, so this is a contract and not a reminder:

- Root-doc producer/check tests — repo-root projections (the `aw:start` block,
  generated command tables, and generated trait table) stay fresh against
  their generators. `aw health` should route agents to those producer commands
  as they are wired.
- The repo-root doc allowlist test — the root carries only the doc files this
  contract names; a stray file is a defect, not a new home.
- The project/app-layer agent-doc test — `apps/**/{CLAUDE,AGENTS}.md` and
  `projects/**/{CLAUDE,AGENTS}.md` are forbidden except for declared
  template-source files.
- `root_doc_mirror_test` — CLAUDE and AGENTS stay one document in two
  agent-runtime flavors via an explicit whitelist, not freehand divergence.
- `aw capability check` plus rule R7g — a project's README-owned capability
  contract stays internally consistent, including that TD capability refs
  actually resolve against it.
- The active-docs scanners — agent-facing docs and skills are re-scanned for
  retired command literals on every change, including the negative case: a
  doc that only explains a command was retired must not itself carry that
  command's literal invocation text, or the scanner cannot tell "documents
  the removal" from "still tells an agent to run it."

Payload corollary: when a lifecycle section has a schema (e.g. TD `unit-test`),
its agent payload is JSON data and the artifact's YAML frontmatter + mermaid
diagram are CLI-rendered projections; prose-only payloads remain Markdown.

## Project build and release contract

Every project build skill and project `build.sh` must use the same two-mode
contract. The agent-facing entry points are `<project>:build:debug` and
`<project>:build:release`; the generic dispatcher is `aw:build:{debug,release}`
using the project entry in `aw.toml`.

### Debug builds

Debug builds are local install checkpoints:

1. If the tree is dirty, commit first with a debug checkpoint commit.
2. Read the project's configured base version and check whether
   `<project>@<version>` already exists locally or on `origin`.
3. If the tag exists, advance to the next version using the repository base-64
   carry rule: patch increments first; when patch would exceed 63, reset patch
   to 0 and increment minor; when minor would exceed 63, reset minor to 0 and
   increment major.
4. Build with the Cargo debug profile and a SemVer build-metadata suffix:
   `<base-version>+<git-sha>`.
5. Install the debug binary locally, then restore manifest and lockfile edits so
   the debug-only version suffix is not left in the worktree.

### Release builds

Release builds are not complete until the GitHub Release is visible. The
required chain is:

1. **release-prep**: run the project `build.sh release` through the relevant
   skill wrapper. The script checks local and remote tag collisions for
   `<project>@<version>`, advances the version with the same base-64 carry rule
   when needed, runs the Cargo release build, installs the local release binary,
   commits version files as `release(<project>): <project>@X.Y.Z`, and prints
   `RELEASE_TAG=<project>@X.Y.Z`. It must not create or push the tag.
2. **land**: run `git:land` as-is so the release commit lands on `main`. Do not
   tag before this step; rebases and squash merges can orphan a pre-land tag.
3. **tag + push**: tag the landed `HEAD` and push the tag:

   ```bash
   git tag -a <project>@X.Y.Z -m "Release <project>@X.Y.Z"
   git push origin <project>@X.Y.Z
   ```

4. **monitor**: run the release monitor and do not report success until it
   completes:

   ```bash
   scripts/project-build-monitor-release.sh <project> <project>@X.Y.Z
   ```

   The monitor watches `.github/workflows/<project>-release.yml` for that tag
   when the workflow exists, fails if the Actions run fails, then verifies
   `gh release view <project>@X.Y.Z` before printing the GitHub Release URL. If
   a project has no release workflow yet, the monitor still polls the GitHub
   Release directly and fails if it never appears.

The release identity is always:

```
<project>@X.Y.Z        # e.g. lumen@0.4.4, vat@0.3.62
```

Projects should release **independently**. A releasing project should prefer its
own `version` in its project `Cargo.toml`, so bumping one project does not bump
the others. A few crates still inherit `[workspace.package].version`; until they
are migrated, their `build.sh` must explicitly name the version source and every
manifest it edits. Do not silently bump the workspace version for an unrelated
project release.
