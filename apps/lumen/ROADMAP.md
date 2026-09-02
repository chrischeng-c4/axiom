# Lumen roadmap

## Purpose

This document records future product outcomes and explicit non-goals. It does
not describe current support. [STATUS.md](STATUS.md) owns that contract.

The issue tracker owns assignees, work state, schedules, and delivery history.
This file keeps stable outcome IDs so current limits can point to one future
destination without copying tracker state.

## Near-term outcomes

### Durable write contract

- ID: `durable-write-contract`
- Outcome: Every persistent write has one acknowledgement point after durable
  commit and index apply, while in-memory mode is explicitly ephemeral.
- Boundary: The selected backend can use different internal logs and fsync
  mechanisms, but a persistent 2xx has one public meaning. This outcome does
  not make the derived index the caller's source of truth. The Docker image
  selecting a durable backend does not complete the uniform durable-2xx or
  crash-injection outcome.
- Completion evidence: Crash-injection tests cover every persistent backend
  before commit, after commit, before apply, and after apply. Acknowledged
  values survive restart, rejected values do not appear, and in-memory startup
  and status identify the mode as ephemeral.
- Tracking: [Milestone #9](https://github.com/chrischeng-c4/axiom/milestone/9).

### Idempotent write replay

- ID: `idempotent-write-replay`
- Outcome: HTTP writes use a durable, payload-bound `Idempotency-Key` and replay
  the original result for a safe caller retry.
- Boundary: Generated clients create a key by default and allow caller
  override. Lumen retains the key and original response for at least 24 hours.
  The same key with a different payload is a conflict, not a second write.
- Completion evidence: Runtime and TypeScript, Python, and Rust client tests
  prove default and caller-supplied keys, exact response replay, `409` for a
  different payload, restart and multi-process retention, expiry after the
  stated window, redaction, and no second mutation under concurrent retry.
- Tracking: [Milestone #10](https://github.com/chrischeng-c4/axiom/milestone/10).

### Item-atomic batch writes

- ID: `item-atomic-batch-writes`
- Outcome: A batch reports stable per-item success or failure, and each item is
  applied in full or not applied.
- Boundary: Sibling items can have different results. The contract does not
  turn a batch into one all-or-nothing transaction across every item.
- Completion evidence: Tests inject validation, capacity, storage, and shard
  failures between the fields of an item. They prove no partial item becomes
  visible, accepted siblings remain committed, response order is stable, and a
  safe retry uses the idempotency contract.
- Tracking: [Milestone #11](https://github.com/chrischeng-c4/axiom/milestone/11).

### Versioned deletes and tombstones

- ID: `versioned-deletes-and-tombstones`
- Outcome: External-version collections keep a delete tombstone that prevents
  an older delayed write from restoring deleted data.
- Boundary: The collection selects arrival order or external versioning. A
  tombstone follows the same ownership level as the selected document or field
  write model and can be reclaimed only after its safe retention boundary.
- Completion evidence: Ordered and out-of-order tests cover document and field
  deletes, equal and older versions, restart, compaction, replication, rebuild,
  and retention. No accepted old write resurrects a deleted value.
- Tracking: [Milestone #11](https://github.com/chrischeng-c4/axiom/milestone/11).

### Shadow rebuild generations

- ID: `shadow-rebuild-generations`
- Outcome: A collection can rebuild a shadow generation, seal it, activate it
  atomically, and roll back to the retained previous generation.
- Boundary: Active and shadow receive the same live writes only after full
  validation against both. Rebuild input uses ordered chunks and one durable
  Operation. Activation and rollback use ETag compare-and-swap. Previous
  generation retention defaults to 24 hours and accepts one hour through seven
  days.
- Completion evidence: Tests prove dual validation writes neither generation
  when one rejects, ordered-chunk resume after restart, durable operation
  status, explicit seal, stale ETag refusal, activation, rollback, retention
  expiry, and cancellation back to active-only writes without acknowledged
  data loss.
- Tracking: [Milestone #16](https://github.com/chrischeng-c4/axiom/milestone/16).

### Strict search schema types

- ID: `strict-search-schema-types`
- Outcome: Collections use explicit scalar types with orthogonal multi-value
  and facet options instead of the legacy `number` and `set` model.
- Boundary: The types are `text`, `keyword`, `int64`, `float64`,
  `decimal(p,s)`, `timestamp`, `date`, `boolean`, `vector`, and `hash`.
  `multi` and `facetable` are independent options. Decimal precision is 1
  through 38, scale is 0 through precision, and invalid values are rejected
  without rounding.
- Completion evidence: Schema and wire tests cover every type, canonical
  decimal strings without exponents, precision and scale limits, null and
  omitted values, empty multi-value sets, de-duplicated unordered values, and
  rejection of overflow. Migration tests prove an existing field needs a
  shadow rebuild to become facetable while a new field with no history can be
  added online.
- Tracking: [Milestone #15](https://github.com/chrischeng-c4/axiom/milestone/15).

### Unified search contract

- ID: `unified-search-contract`
- Outcome: Search separates scoring from filtering and returns one deterministic
  ordering, pagination, total, and collapse contract.
- Boundary: `ScoringQuery` and `FilterExpr` are optional and intersect when
  both exist. Filters do not affect score and run before kNN and each RRF leg.
  Lexical bool excludes kNN, heterogeneous hybrid uses RRF, `has_child` is a
  bounded semi-join, and cross-collection search keeps independent results.
  The result contract caps page and offset, supports live cursor and optional
  PIT, and preserves source IDs during collapse.
- Completion evidence: Tests cover match-all, filter-only null score, scoring
  with field sort, bool `minimum_should_match` defaults and overrides, kNN and
  RRF candidate filtering, child-score isolation, deterministic external-ID
  tie-breaks, page size 1,000, offset 10,000, live and PIT cursors, `400`
  mismatch, `409` stale generation, `410` expiry, tagged totals, and collapse
  totals, missing values, representative order, and field eligibility.
- Tracking: [Milestone #17](https://github.com/chrischeng-c4/axiom/milestone/17).

### Exact search facets metrics

- ID: `exact-search-facets-metrics`
- Outcome: Search returns exact terms facets, caller-defined range facets, and
  top-level count, valueCount, min, max, sum, and avg metrics.
- Boundary: Facets use the complete `query + filter` match set before paging,
  sort, cursor, and collapse. They do not remove their own filter. The first
  version excludes kNN, RRF, `search:all`, nested metrics, general aggregation,
  histograms, percentiles, cardinality, pipelines, and bucket cursors.
- Completion evidence: Contract tests cover terms size and order, canonical
  prefix, `min_count`, exact distinct and missing counts, truncation, sorted
  non-overlapping half-open ranges, unbounded ends, gaps, stable keys, exact
  unbucketed count, multi-value per-document counting, empty results, exact
  int64 and decimal sums without overflow, 18-place half-even decimal average,
  deterministic compensated float sums marked approximate, decimal-string
  counts, `facets` and `metrics` maps, alias grammar, snake-case fields, and
  explicit `kind` discriminators.
- Tracking: [Milestone #19](https://github.com/chrischeng-c4/axiom/milestone/19).

### Facet resource governance

- ID: `facet-resource-governance`
- Outcome: Facet work has deterministic complexity, memory, timeout, admission,
  cache, failure, and disclosure limits.
- Boundary: One request has at most 16 facet plus metric definitions, 100
  buckets per definition, 65,536 working buckets, 16 MiB state per shard and
  coordinator, and 10% process memory for facet state. Static excess returns
  `400`; temporary capacity returns `429` with `Retry-After`; timeout returns
  `504` without partial data. Authorization runs before one process-wide
  byte-weighted LRU whose budget is 5% of internal memory and at most 256 MiB.
- Completion evidence: Negative tests cover every static and dynamic limit,
  timeout range 1 through 30,000 with 5,000 default, disconnect cancellation,
  whole-search shard failure, state cleanup, admission credits for items plus
  facets plus metrics, Managed KSA and Standalone anonymous buckets, cache key
  dimensions and revision invalidation, entries above 1 MiB, authorization
  before cache, shared authorized results, and whole-runtime disclosure of
  facetable values. A single-core CI fixture with 10,000 documents, five facets,
  and two metrics stays below 200 ms. Release evidence records 100,000-document
  p95 and peak memory and rejects latency regression above 20% or memory above
  10%. It also reports write throughput and segment bytes for review above 30%
  and 50% change.
- Tracking: [Milestone #20](https://github.com/chrischeng-c4/axiom/milestone/20).

### Distributed facet convergence

- ID: `distributed-facet-convergence`
- Outcome: Shards and the coordinator merge exact facet and metric state under
  the selected read consistency without changing single-shard semantics.
- Boundary: The coordinator honors the same memory and bucket limits. A shard
  failure fails the complete search. The first version does not silently
  approximate or return partial buckets.
- Completion evidence: Multi-shard and replicated tests compare terms, ranges,
  multi-value counts, exact integer and decimal metrics, deterministic float
  metrics, missing and unbucketed counts, and truncation with a single-shard
  oracle. Fault tests cover leader movement, retry, timeout, shard failure,
  cancellation, resource cleanup, and no partial response.
- Tracking: [Milestone #21](https://github.com/chrischeng-c4/axiom/milestone/21).

### Generated-client search v2 parity

- ID: `generated-client-search-v2-parity`
- Outcome: Generated TypeScript, Python, and Rust clients expose the complete
  Search v2 request and response contract as typed APIs.
- Boundary: Lumen integration owns the service-specific discriminated query,
  filter, facet, metric, total, cursor, and collapse contract. Generic
  cross-language discriminator and schema mechanisms remain in
  `libs/openapi-codegen`. Package publication remains a non-goal.
- Completion evidence: A required three-language gate compiles and executes
  scoring and filter combinations, every facet and metric definition, tagged
  totals, cursor errors, collapse, decimal strings, and every `kind` result.
  Rust does not fall back to JSON values or strings for the new unions and
  enums. No language can silently skip because a toolchain or dependency is
  absent.
- Tracking: [Milestone #22](https://github.com/chrischeng-c4/axiom/milestone/22).

### Search v2 migration

- ID: `search-v2-migration`
- Outcome: Callers can move a 0.4 schema, request, and response integration to
  Search v2 through an explicit compatibility window and offline tools.
- Boundary: Strict unknown-field rejection lands in 0.4.x before new request
  fields. The migration covers `number`, `set`, tagged totals, missing sort,
  cursor errors, source-ID-preserving collapse, and removal of `/duplicates`.
  `lumen migrate search-request` and `lumen migrate collection-schema` read
  stdin and write JSON plus a report without contacting a runtime or guessing
  an ambiguous mixed `OR` or `NOT`.
- Completion evidence: Versioned tests prove every 0.4 warning and every Search v2
  refusal and response shape. Tool fixtures cover successful conversions,
  numeric and set schema rebuild requirements, ambiguous request refusal, no
  network or runtime write, stable reports, and round-trip validation against
  the Search v2 schema.
- Tracking: [#3813](https://github.com/chrischeng-c4/axiom/issues/3813) (`lumen@0.4.51`).

### Search capability activation

- ID: `search-capability-activation`
- Outcome: Managed Lumen enables a search capability only after every serving
  member supports it and the effective compatibility version is finalized.
- Boundary: `/version` separates binary capabilities, active capabilities, and
  effective compatibility version. The first facet capability is
  `search_facets_v1`. Standalone can use a supported new binary directly.
  Managed mixed versions reject activation and do not use version-aware data
  routing.
- Completion evidence: Operator and runtime tests cover member discovery,
  stale and missing capability reports, finalization, activation, rollback,
  restart, member replacement, and refusal of mixed-version search fields.
  `/version` reports each dimension without claiming a capability active before
  convergence.
- Tracking: [#3814](https://github.com/chrischeng-c4/axiom/issues/3814) (`lumen@0.5.0`).

### Runtime configuration parity

- ID: `runtime-configuration-parity`
- Outcome: Standalone and Managed runtimes use one classified configuration
  contract. Each Fleet instance can receive distinct supported settings.
- Boundary: `instances[].spec` keeps its RFC 7386 advanced-patch behavior. A
  new typed `instances[].runtime` surface becomes the primary runtime interface.
  An entry that sets the same runtime value through both surfaces is rejected.
  `extraEnv` accepts literals and Secret or ConfigMap key references only.
  `extraArgs` accepts flags only. Identity, topology, storage, and security
  values reserved by the operator cannot be overridden.
- Completion evidence: An inventory classifies every `lumen serve` option as
  typed, guarded escape hatch, or reserved. CRD and render tests prove that two
  namespaces receive different settings, conflicts and reserved values fail,
  Secret values do not enter status, and a stable effective-config hash rolls
  pods only when restart-required inputs change. Referenced Secret and
  ConfigMap resource versions affect that hash. TLS file rotation stays hot.
- Tracking: [Milestone #24](https://github.com/chrischeng-c4/axiom/milestone/24).

### Managed runtime KSA access

- ID: `managed-runtime-ksa-access`
- Outcome: Direct Managed and Fleet declarations name the exact Kubernetes
  ServiceAccounts that may use each complete Lumen runtime, and the operator
  converges the matching namespaced RBAC.
- Boundary: `Lumen.spec.access` is the direct entry. Fleet defaults use
  `defaults.access`; `instances[].access` replaces the complete default list
  and never forms a union. An empty list is explicit deny-all. Every subject
  has one namespace and name. `instances[].spec.access`, wildcards, groups,
  and raw usernames are rejected. The operator does not create a namespace,
  ServiceAccount, client Deployment, token Secret, or cluster. Kubernetes RBAC
  remains authoritative, including grants created by a cluster administrator
  outside Fleet ownership. The single `lumenruntimes/use` grant permits query,
  index, collection-management, and admin requests. A runtime is one complete
  trust boundary. This outcome does not provide a fine-grained permission
  model.
- Completion evidence: CRD, merge, render, and controller tests prove Direct
  and Fleet parity, cross-namespace KSA subjects, explicit deny-all,
  replacement instead of union, advanced-patch rejection, duplicate and
  wildcard refusal, exact `lumenruntimes/use` resource names, Role and
  RoleBinding apply, adoption refusal, stale-grant prune, and recovery after a
  per-object failure. Condition tests prove `AccessPolicyReady=False` blocks
  `Ready=True`, while a converged deny-all policy reports
  `AccessPolicyReady=True`.
- Tracking: [#3798](https://github.com/chrischeng-c4/axiom/issues/3798) (`lumen@0.4.35`).

### Projected KSA client auth

- ID: `projected-ksa-client-auth`
- Outcome: A generated Lumen client keeps local Standalone credential-free,
  supports the Kubernetes default ServiceAccount token for in-cluster
  Standalone, and has an explicit Managed KSA connection profile. Application
  code never holds the credential value.
- Boundary: Local and Compose Standalone never read a token file. In-cluster
  Standalone reads the Kubernetes default ServiceAccount token only for an
  exact Service DNS URL. `ManagedKsa` requires HTTPS Service DNS, a CA path,
  and the fixed token path
  `/var/run/secrets/lumen.axiom.dev/token`. The application or platform projects
  the audience `lumen.axiom.dev` token and CA into its workload. Fleet does not
  mutate client workloads. `libs/service-auth` supplies a portable opaque token
  source and redaction. `libs/openapi-codegen` supplies the per-request header
  provider. A missing, unreadable, or empty Managed token fails before
  transport. The client does not parse the token as an authorization decision.
  Server-side TokenReview remains authoritative for signature, expiry,
  audience, and KSA identity. A `401` never downgrades to anonymous.
- Completion evidence: Generated TypeScript, Python, and Rust integration tests
  prove local Standalone use with no credential access, in-cluster Standalone
  use with the default token, Managed use with a valid audience-bound token and
  private CA, a new opaque token on the next request after kubelet rotation,
  hard failure for missing, unreadable, and empty required files, server
  rejection of expired, wrong-audience, and malformed tokens, no anonymous
  fallback after `401`, and no credential in arguments, environment, status,
  Events, logs, or error text.
- Tracking: [#3799](https://github.com/chrischeng-c4/axiom/issues/3799) (`lumen@0.4.36`).

### Managed auth unification

- ID: `managed-auth-unification`
- Outcome: Managed Lumen has one required KSA identity path and one
  whole-runtime permission model, while Standalone keeps its local auth-off
  default.
- Boundary: The 0.4.x migration keeps missing `access`, per-collection grants,
  and `auth: disabled` working with deprecation notices. Version 0.5.0 requires
  explicit Managed `access`, removes per-collection permissions, and rejects
  disabled Managed auth. Only `/healthz`, `/readyz`, `/metrics`, and `/version`
  stay anonymous. `/debug/cluster`, `/openapi.json`, `/docs`, and every runtime
  API require the whole-runtime grant.
- Completion evidence: Versioned compatibility tests prove each 0.4.x warning,
  old external RBAC behavior when `access` is absent, explicit empty deny-all,
  and 0.5.0 refusal of missing access, disabled auth, and per-collection grants.
  Route tests prove the exact anonymous set and protect `/debug/cluster`.
  Migration tests prove old resources receive an actionable message without
  leaking a token or silently widening access.
- Tracking: [Milestone #25](https://github.com/chrischeng-c4/axiom/milestone/25).

### Protocol contract completeness

- ID: `protocol-contract-completeness`
- Outcome: Lumen's machine-readable contract describes every public request
  control, shared failure response, and media type that a client must handle.
- Boundary: `apps/lumen` keeps domain routes, schemas, and error meaning.
  `libs/service-http` supplies reusable middleware response projection. The
  OpenAPI document adds `X-Read-Consistency`, the shared `401`, `413`, `429`,
  and `500` structured error responses, and the complete `text/plain` NDJSON
  request and streaming response declarations. Runtime behavior does not move
  into the document checker or this roadmap.
- Completion evidence: A route-and-middleware parity test fails when a mounted
  operation, public request header, reachable shared status, error envelope, or
  media type is absent from OpenAPI. Snapshot tests prove the committed
  document is byte-identical to live generation, and stream-reindex tests prove
  its declared framing matches the runtime.
- Tracking: [Milestone #12](https://github.com/chrischeng-c4/axiom/milestone/12).

### Generated-client protocol parity

- ID: `generated-client-protocol-parity`
- Outcome: Generated TypeScript, Python, and Rust clients preserve Lumen's
  supported operation media types, streaming contract, error contract, and
  schema types.
- Boundary: `libs/openapi-codegen` owns non-JSON operation modeling, NDJSON
  streaming methods, typed structured errors, cross-language schema mapping,
  and target dependency manifests. Lumen owns the service-specific integration
  contract. Dynamic KSA token loading remains the separate
  `projected-ksa-client-auth` outcome. Publishing language packages remains a
  non-goal.
- Completion evidence: All three generated clients send an NDJSON reindex body,
  consume progress events through a bounded streaming API, decode representative
  `401`, `413`, `429`, and `500` bodies into one typed error shape, preserve
  Lumen unions and enums including Rust `QueryNode`, and install from complete
  generated dependency metadata. Tests cover success, malformed stream data,
  early disconnect, error redaction, and POST fallback without buffering the
  whole stream.
- Tracking: [Milestone #13](https://github.com/chrischeng-c4/axiom/milestone/13).

### Strict generated-client gates

- ID: `strict-generated-client-gates`
- Outcome: One required gate proves the generated TypeScript, Python, and Rust
  clients all compile and run the same Lumen public-API journey.
- Boundary: The gate uses pinned interpreters, compilers, runtimes, and package
  dependencies. A missing prerequisite fails setup instead of returning from a
  language branch. Local convenience tests may keep explicit skip behavior, but
  they cannot be cited as the three-language release gate.
- Completion evidence: A controlled negative test removes each language
  prerequisite and proves the required gate fails. The normal gate records all
  three executed languages and passes create, index, QUERY with POST fallback,
  search, stats, delete, and collection-drop behavior for each generated client.
- Tracking: [Milestone #13](https://github.com/chrischeng-c4/axiom/milestone/13).

### Generated-client request resilience

- ID: `generated-client-request-resilience`
- Outcome: Generated TypeScript, Python, and Rust clients apply one documented
  Lumen request policy without requiring each application to rebuild it.
- Boundary: The generated clients support QUERY with the permanent POST
  fallback, typed errors, `Retry-After`, bounded exponential backoff with
  jitter, deadlines, and cancellation. They retry read requests only when the
  protocol marks them safe. They retry writes only when the
  `Idempotency-Key` contract is active and the request has a key. A token
  rotation race can retry only a safe request or a keyed write. The library
  provides operation metadata and hooks. Lumen owns the service policy.
- Completion evidence: Cross-language tests cover QUERY and POST fallback,
  every documented retry status, `Retry-After`, jitter bounds, deadline and
  cancellation propagation, read retry, unkeyed write refusal, keyed write
  replay, ambiguous mutation failure, token rotation during a request, typed
  redacted errors, and a strict three-language gate with no silent skip.
- Tracking: [Milestone #14](https://github.com/chrischeng-c4/axiom/milestone/14).

### Generated-client source-integration helpers

- ID: `generated-client-source-integration-helpers`
- Outcome: Generated clients can hydrate ordered Lumen hits through one
  caller-supplied bulk fetch callback without moving source ownership into
  Lumen.
- Boundary: The helper accepts ordered IDs and a callback that loads an ID list.
  It performs no PostgreSQL query itself. It avoids N+1 fetches, restores Lumen
  hit order, keeps search metadata attached, and reports missing source records
  without inventing them. The caller owns source authorization, projection,
  transaction boundaries, and freshness.
- Completion evidence: TypeScript, Python, and Rust tests prove one bulk
  callback for one page, deterministic order restoration, duplicate and missing
  ID handling, callback failure and cancellation, preservation of score and
  cursor metadata, no source credential in generated state, and no hidden
  per-record fetch.
- Tracking: [Milestone #14](https://github.com/chrischeng-c4/axiom/milestone/14).

### Versioned client workload template

- ID: `versioned-client-workload-template`
- Outcome: Each Lumen release provides a copyable Kubernetes Kustomize template
  for a client workload that uses the Managed KSA and private-CA contract.
- Boundary: The template uses an existing ServiceAccount and sets
  `automountServiceAccountToken: false`. It mounts one projected token with
  audience `lumen.axiom.dev` at the fixed token path. It mounts the
  operator-published public CA ConfigMap without `subPath`. Only the endpoint
  and CA path enter environment variables. The token value never enters an
  environment variable or Secret. An egress NetworkPolicy is optional. The
  operator and admission webhooks do not mutate the client Deployment. The
  existing `k8s/overlays/template` remains a Standalone runtime template.
- Completion evidence: Release fixtures pin the template version and prove
  `kubectl kustomize` output, server-side dry-run, token audience and path,
  disabled default token mount, visible CA rotation without Pod restart,
  optional egress policy, no created namespace, KSA, or token Secret, and no
  token value in rendered YAML, environment, arguments, logs, Events, or
  status.
- Tracking: [#3799](https://github.com/chrischeng-c4/axiom/issues/3799) (`lumen@0.4.36`).

### Fleet production convergence

- ID: `fleet-production-convergence`
- Outcome: Managed Fleet reconciliation reacts to Fleet and child changes,
  isolates each entry failure, and reports materialized, searchable, writable,
  replication, configuration, capacity, and degraded state without conflating
  them.
- Boundary: Reusable ownership, adoption, prune, status-projection, and watch
  mechanics move to `libs/service-k8s::fleet`. Lumen supplies child objects,
  protected topology projection, and health mapping. `Ready` continues to mean
  searchable. `Writable`, `ReplicationReady`, `ConfigReady`, `CapacityReady`,
  and `Degraded` remain separate conditions. A low-frequency repair requeue
  remains as a safety net.
- Completion evidence: Library and Lumen tests prove Fleet and child watches,
  leader failover, duplicate-target rejection, adoption safety, per-entry API
  failure isolation, ready and degraded counts, desired and observed generation
  reporting, retained-orphan warnings, and recovery after a failed dependency
  returns.
- Tracking: [#3796](https://github.com/chrischeng-c4/axiom/issues/3796) (`lumen@0.4.33`).

### Fleet safe rollout

- ID: `fleet-safe-rollout`
- Outcome: A Fleet update advances through declared instances in order and
  stops before the next instance while the current child has not converged.
- Boundary: `rolloutPolicy.maxConcurrent` defaults to `1` and
  `rolloutPolicy.paused` defaults to `false`. Progress requires the child
  observed generation to catch up and searchable `Ready=True`. The first
  version has no automatic wall-clock timeout. A failed child resumes
  automatically after recovery. This policy orders child runtimes. It does not
  control Raft members inside one runtime.
- Completion evidence: Reconcile tests prove one-at-a-time default rollout,
  explicit concurrency, manual pause, generation gating, Ready gating, stop on
  degradation, leader failover, and automatic continuation after recovery.
- Tracking: [#3797](https://github.com/chrischeng-c4/axiom/issues/3797) (`lumen@0.4.34`).

### GKE regional production profile

- ID: `gke-regional-production-profile`
- Outcome: Lumen has one certified Managed production profile on GKE Standard
  Regional while Standalone remains the formal local-development entry.
- Boundary: The profile uses three zones, private ClusterIP serving, Dataplane
  V2 with NetworkPolicy, Kubernetes-native resources and placement, persistent
  storage, operator-managed rollout, backup and restore, KSA request identity,
  and separate Workload Identity Federation for Google API access. The platform
  owns the cluster, VPC, ComputeClass, StorageClass, issuer, bucket, and
  monitoring backend. Current zonal acceptance is only prior evidence. It
  cannot certify this profile. Autopilot and non-GKE Kubernetes are outside
  this first certification.
- Completion evidence: A regional GKE acceptance environment proves install,
  Fleet and direct reconciliation, one-Pod persistent non-HA operation,
  three-voter placement across three zones, private serving, NetworkPolicy,
  KSA auth, WIF-backed operator duties, certificate and CA rotation, backup and
  restore, node drain, Pod loss, zone loss, interrupted rollout, recovery, and
  deterministic teardown without using a GCE machine type in the core CRD.
- Tracking: [#3803](https://github.com/chrischeng-c4/axiom/issues/3803) (`lumen@0.4.41`).

### Per-shard failure-domain placement

- ID: `per-shard-failure-domain-placement`
- Outcome: Every production three-voter shard places its replicas on distinct
  hosts and across three zones without blocking safe node sharing by unrelated
  shards or runtimes.
- Boundary: Hard host anti-affinity compares only replicas of the same shard.
  Three-voter shards require three distinct zones. Whole-runtime topology spread
  can remain a soft packing hint, but it is not the HA guarantee. A one-Pod
  profile does not claim node-failure or zone-failure availability. Two voters
  remain a compatibility shape and are not production HA.
- Completion evidence: Render, scheduler, and regional failure tests prove
  shard-scoped labels and selectors, hard same-shard host separation,
  three-zone voter placement, allowed node sharing across different shards and
  runtimes, correct unschedulable status when domains are insufficient, and
  continued quorum through one Pod, node, or zone loss.
- Tracking: [#3801](https://github.com/chrischeng-c4/axiom/issues/3801) (`lumen@0.4.38`).

### Quorum-safe runtime rollout

- ID: `quorum-safe-runtime-rollout`
- Outcome: A Managed runtime changes one Raft member at a time and waits for the
  shard to recover before it changes the next member.
- Boundary: The runtime controller, not Fleet, owns this inner rollout. It gates
  progress on desired and observed generation, searchable `Ready`, quorum, and
  replication readiness. A three-voter shard uses quorum-safe PDB availability.
  PDB protects voluntary eviction only. It does not constrain a StatefulSet
  rolling update. The controller must therefore drive or partition the member
  rollout explicitly.
- Completion evidence: Multi-member tests prove one-at-a-time default updates,
  generation and readiness gates, quorum and replication recovery, pause on
  member loss, recovery and continuation, leader movement, voluntary drain
  within PDB limits, direct StatefulSet update safety, interrupted operator
  recovery, and no second unavailable voter.
- Tracking: [#3802](https://github.com/chrischeng-c4/axiom/issues/3802) (`lumen@0.4.39`).

### Kubernetes-native placement

- ID: `kubernetes-native-placement`
- Outcome: New Managed manifests express placement through Kubernetes resource
  requests, StorageClass, node selector, tolerations, and topology intent.
- Boundary: The public contract contains no GCE machine type or node-pool
  lifecycle. A GKE profile selects a platform-owned custom ComputeClass through
  its standard node selector. The platform owns ComputeClass configuration and
  node auto-provisioning. Stateful ComputeClasses disable active migration.
  StorageClasses use `WaitForFirstConsumer`. Existing
  `placement.initialMachineType` and the capacity catalog remain a legacy
  compatibility path. New manifests do not require that catalog.
- Completion evidence: Additive CRD, render, and controller tests prove that
  new manifests use only Kubernetes-native fields and never read the catalog,
  old manifests keep materializing through the existing catalog, resource and
  storage requests reach each Pod and PVC, placement intent maps to standard
  scheduling fields, and a GKE profile selects a custom ComputeClass without
  placing a GCE machine type in the core API.
- Tracking: [#3801](https://github.com/chrischeng-c4/axiom/issues/3801) (`lumen@0.4.38`).

### Managed runtime certificates

- ID: `managed-runtime-certificates`
- Outcome: The Lumen operator requests, owns, and rotates separate serving and
  peer leaf certificates for each Managed runtime.
- Boundary: The platform supplies the issuer, CA policy, and any Workload
  Identity Federation permission needed to call an external CA service. Lumen
  derives serving identities from Service DNS and peer identities from the Raft
  topology. It stores leaf material in runtime Secrets and keeps serving and
  peer certificates separate. The runtime keeps its existing hot reload. The
  pre-created Secret path remains a compatibility path. Private keys never
  enter Fleet, status, Events, or logs.
- Completion evidence: Controller and regional tests prove identity derivation,
  initial issue, serving and peer Secret separation, renewal before expiry,
  issuer outage and recovery, hot reload without unsafe member fan-out,
  ownership and adoption refusal, redaction, compatibility with pre-created
  Secrets, and no private key outside the owned Secret volume.
- Tracking: [#3800](https://github.com/chrischeng-c4/axiom/issues/3800) (`lumen@0.4.37`).

### Managed client trust

- ID: `managed-client-trust`
- Outcome: The operator publishes the public serving trust for each runtime in
  every namespace that contains an allowed client ServiceAccount.
- Boundary: One runtime-specific CA ConfigMap is shared by the allowed KSAs in
  one namespace. Root rotation publishes old and new roots together during the
  overlap. The operator prunes only ConfigMaps that it owns. Adoption or prune
  conflict sets `ClientTrustReady=False`, which blocks complete Managed
  readiness. The first version does not depend on ClusterTrustBundle
  projection. It creates no namespace, ServiceAccount, client Deployment, or
  token Secret.
- Completion evidence: Cross-namespace controller tests prove create, update,
  multi-KSA sharing, root overlap, removal after the overlap, exact ownership,
  adoption refusal, stale-object prune, retained foreign objects, per-object
  failure recovery, `ClientTrustReady` gating, public-only data, and no change
  to client Deployments.
- Tracking: [#3800](https://github.com/chrischeng-c4/axiom/issues/3800) (`lumen@0.4.37`).

### Managed data retention

- ID: `managed-data-retention`
- Outcome: Fleet child deletion and instance data deletion become separate,
  explicit policies with Retain defaults.
- Boundary: `prunePolicy` keeps controlling the child `Lumen` only. A new
  `Lumen.spec.dataRetentionPolicy` controls PVC objects and defaults to Retain.
  Delete uses a finalizer and selects only PVCs owned by that instance. The
  StorageClass reclaim policy continues to decide the PV lifecycle.
- Completion evidence: Reconcile tests prove default child deletion leaves
  PVCs, explicit data Delete removes only exact instance PVCs, unrelated PVCs
  remain, failed cleanup retains the finalizer and publishes a condition, and
  retry succeeds without broad deletion.
- Tracking: [#3093](https://github.com/chrischeng-c4/axiom/issues/3093) (`lumen@0.4.40`).

### Managed embedded data durability

- ID: `managed-embedded-data-durability`
- Outcome: A one-replica Managed Lumen runtime persists its index and AOF on
  the retained PVC child path.
- Boundary: The operator keeps the storage parent mount and adds only the
  exact `data` child mount for the embedded Raft store. This does not recover
  data that an earlier node-local runtime already lost.
- Completion evidence: Render and restart tests prove the exact parent and
  child PVC mounts, a persisted checkpoint, pre-restart search after Pod
  replacement, legacy PVC adoption, and fail-closed corruption handling.
- Tracking: [Milestone #7](https://github.com/chrischeng-c4/axiom/milestone/7).

### Deterministic consensus conformance

- ID: `deterministic-consensus-conformance`
- Outcome: Raft recovery and membership safety have deterministic adversarial
  replay evidence.
- Boundary: The deterministic host shares cold-start, persist, apply-ready,
  and peer-lane primitives but starts no transport, task, wall-clock, or sleep.
- Completion evidence: The fixed corpus, replay parser, and harness-only
  mutants prove safety across bounded elections, partitions, restarts,
  compactions, and membership transitions.
- Tracking: [Milestone #7](https://github.com/chrischeng-c4/axiom/milestone/7).

### Bounded Raft shutdown and failover

- ID: `bounded-raft-shutdown-and-failover`
- Outcome: Raft shutdown and leader failover complete through explicit bounded
  recovery behavior.
- Boundary: The runtime keeps quorum and does not hide unfinished work behind
  an unbounded background task.
- Completion evidence: Fault tests prove drain, leadership transfer, timeout,
  restart, and acknowledged-write behavior under the stated bounds.
- Tracking: [Milestone #8](https://github.com/chrischeng-c4/axiom/milestone/8).

### Distributed search routing and merge

- ID: `distributed-search-routing-and-merge`
- Outcome: Lumen routes distributed search work and merges results through one
  declared failure and ordering contract.
- Boundary: The contract does not claim partial results as complete and keeps
  caller source-record ownership outside Lumen.
- Completion evidence: Multi-shard tests prove routing, merge order, failure,
  cursor, retry, and resource cleanup behavior.
- Tracking: [Milestone #18](https://github.com/chrischeng-c4/axiom/milestone/18).

### Regional topology migration and backup

- ID: `regional-topology-migration-and-backup`
- Outcome: A regional runtime can migrate topology and create recoverable
  backups through one declared consistency contract.
- Boundary: A PVC snapshot alone does not prove regional recovery or safe
  cutover.
- Completion evidence: Regional drills prove backup, restore, topology
  cutover, failure handling, and data-consistency checks.
- Tracking: Not assigned.

### Regional upgrade rollback and recovery

- ID: `regional-upgrade-rollback-and-recovery`
- Outcome: A regional runtime can upgrade, roll back, and recover through a
  tested operational contract.
- Boundary: The outcome does not assume that every upgrade is forward-only.
- Completion evidence: Regional drills prove upgrade, rollback, restart,
  recovery, and acknowledged-write safety.
- Tracking: Not assigned.

### Fleet foundation extraction

- ID: `fleet-foundation-extraction`
- Outcome: Fleet's reusable foundation has a clear library boundary while
  Lumen keeps its own search and Raft policy.
- Boundary: The extraction does not move Lumen product behavior into a generic
  Fleet mechanism.
- Completion evidence: Compatibility and integration tests prove the extracted
  API preserves declared Fleet convergence behavior.
- Tracking: Not assigned.

## Later outcomes

### GKE Autopilot certification

- ID: `gke-autopilot-certification`
- Outcome: Lumen has a separate, evidence-backed production support tier for
  GKE Autopilot after the Standard Regional profile is complete.
- Boundary: This outcome must adapt Lumen's stateful placement, disruption,
  storage, certificate, backup, and operational contract to Autopilot
  constraints. It cannot reuse the Standard Regional certification by
  assertion. The Kubernetes-native public API remains the same.
- Completion evidence: A dedicated Autopilot acceptance environment proves the
  supported topology, scheduling, storage, private networking, KSA access,
  certificate and CA rotation, backup and restore, upgrades, disruption drills,
  quotas, and documented limitations without a Standard-only node contract.
- Tracking: Not assigned.

### Vector hybrid facets

- ID: `vector-hybrid-facets`
- Outcome: kNN and RRF searches can compute governed facets over one documented
  candidate and match scope.
- Boundary: This outcome starts only after exact non-vector facets and
  distributed convergence are complete. It must define whether facets use the
  candidate set, fused set, or a separate exact filter set. It cannot silently
  present approximate candidate counts as exact full-match counts.
- Completion evidence: Single-shard and distributed tests cover filtered kNN,
  every RRF leg, candidate limits, recall fixtures, exact-versus-approximate
  labels, timeout, memory, and failure behavior against a declared oracle.
- Tracking: Not assigned.

### Membership-aware replica autoscaling

- ID: `membership-aware-replica-autoscaling`
- Outcome: Lumen can add or remove one replica layer across every shard after
  sustained load or capacity signals.
- Boundary: The controller must change Raft membership before it changes the
  StatefulSet. It must keep quorum, honor disruption policy, and roll back or
  stop on a failed transition.
- Completion evidence: A multi-node failure test proves scale-out, scale-in,
  leader movement, restart recovery, and no acknowledged-write loss.
- Tracking: [#3815](https://github.com/chrischeng-c4/axiom/issues/3815) (`lumen@0.5.1`).

### High-availability shard expansion

- ID: `high-availability-shard-expansion`
- Outcome: The automatic shard-split workflow can move data while
  `replicasPerShard` is greater than one.
- Boundary: Every new shard must establish its Raft group and quorum before
  routing ownership changes.
- Completion evidence: A replicated-cluster test proves split, restart,
  retry, and rollback behavior while reads and writes continue.
- Tracking: [#3816](https://github.com/chrischeng-c4/axiom/issues/3816) (`lumen@0.5.2`).

### Protocol compatibility policy

- ID: `protocol-compatibility-policy`
- Outcome: Lumen publishes one versioning and deprecation policy for its HTTP
  operations, schemas, headers, errors, media types, and generated source.
- Boundary: The policy preserves the existing permanent POST twins for QUERY.
  It defines additive and breaking changes, deprecation signals and minimum
  windows, version negotiation, generated-client compatibility, and required
  release notes. It does not promise published npm, PyPI, or crates.io packages.
- Completion evidence: A machine-readable compatibility gate compares two
  contract snapshots and classifies representative additive, deprecated, and
  breaking changes. Versioned integration tests prove the supported overlap
  window, and release fixtures prove every accepted breaking change carries the
  required version and migration note.
- Tracking: [Milestone #23](https://github.com/chrischeng-c4/axiom/milestone/23).

## Non-goals

### Embedding-model execution

- ID: `embedding-model-execution`
- Reason: Lumen accepts raw vector fields and owns vector indexing, kNN, and RRF
  search. The caller or its ingest pipeline chooses and runs the embedding
  model. This keeps model lifecycle, accelerators, batching, and source-data
  policy outside the search-service boundary.

### Source document storage and hydration

- ID: `source-document-storage-and-hydration`
- Reason: The caller's database or object store remains authoritative. Lumen
  returns ordered source IDs and search metadata. The caller loads records by
  those IDs and restores Lumen's order.

### Built-in source connectors

- ID: `built-in-source-connectors`
- Reason: CDC, outbox delivery, source-specific offsets, freshness, and replay
  belong to caller-owned adapters. Lumen keeps one public indexing contract
  instead of owning PostgreSQL or vendor connector lifecycles.

### General aggregation and OLAP

- ID: `general-aggregation-and-olap`
- Reason: Lumen provides bounded search facets and top-level metrics. It does
  not provide arbitrary group-by, nested metrics, histograms, percentiles,
  cardinality, pipelines, or an analytics execution engine.

### Cross-collection joins and ranking

- ID: `cross-collection-joins-and-ranking`
- Reason: Cross-collection search returns independent result sets. Lumen does
  not provide joins, one merged ranking, or one merged cursor across
  collections.

### Operator-created namespaces

- ID: `operator-created-namespaces`
- Reason: Namespace lifecycle includes quota, policy, identity, and ownership.
  The cluster platform provisions target namespaces before Lumen uses them.

### Generic horizontal pod autoscaling

- ID: `generic-horizontal-pod-autoscaling`
- Reason: HPA changes pod count. It does not perform Raft membership or shard
  ownership transitions. Lumen needs a domain-aware controller for this work.

### Operator-created Kubernetes clusters

- ID: `operator-created-kubernetes-clusters`
- Reason: Lumen manages search workloads inside a supplied cluster. Cluster
  provisioning and node-pool lifecycle belong to the cloud or platform layer.

### Standalone high availability

- ID: `standalone-high-availability`
- Reason: Standalone is the local, test, and single-process path. Managed mode
  owns fixed replicated topology, disruption policy, and control-plane recovery.

### Published generated SDK packages

- ID: `published-generated-sdk-packages`
- Reason: Lumen publishes OpenAPI and an on-demand source generator. Consumers
  can vendor the output. Maintaining npm, PyPI, and crates.io release trains is
  outside the Lumen product contract.
