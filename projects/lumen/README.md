# lumen

## Brief

A K8s-native, **log-replicated search specialist**. Five flavors of
"find":

- **Exact** — `keyword` / `number` / `set`
- **Lexical** — `text` (BM25, with tokenize built in)
- **Semantic** — `vector` (CPU: HNSW + exact flat brute-force)
- **Perceptual / structural** — `hash` (pHash / SimHash / b-bit MinHash, Hamming distance)
- **Duplicates** — find which `external_id`s share the same value (a search-flavor of group-by; bounded, posting-list-cheap)

The caller owns the representation:

- Embeddings? **Caller** runs CLIP / BGE / Whisper / VideoMAE; lumen never owns a model artefact.
- Perceptual hashes? **Caller** runs `imagehash` / `datasketch`; lumen indexes the bits.
- Lexical tokenization? **lumen** does it — that's the one place caller doesn't compute (`whitespace_lower` / `ngram` / `jieba`).

The caller also owns the **source of truth**: lumen is a parallel derived index,
never the system of record or an analytics engine — documents are *not* a lumen
concept, only the caller's `external_id` is.

- **Log-driven, derived, rebuildable**. A write is *published to a log*,
  not applied where it lands; every serving node tails the log and folds
  it into its own index. Lossable but rebuildable from the log + the
  caller.
- **Client API on `:7373`** (HTTP/1.1 + HTTP/2 cleartext — REST clients
  need nothing special; see [HTTP](#http--clients)).
- **Sharded**: `hash(collection_id, routing_key || external_id)` selects a
  virtual bucket, and a versioned operator-owned shard map assigns buckets to
  physical storage shards. `shardCount` controls storage ownership,
  `replicasPerShard` controls HA/raft quorum per shard, and HPA never changes
  data ownership.
- **Agent-first offline integration surface**: `lumen spec` emits the exact
  machine schema, including `lumen spec --format openapi-yaml` for LLM-readable
  OpenAPI, while `lumen llm --topic outline`, `lumen llm --topic workflow`,
  `lumen llm --topic integration`, `lumen llm --topic quickstart`, and
  `lumen llm --topic recipes` let an agent pick the smallest context needed to
  wire lumen into an app without a docs site or running server.

## Capabilities

The RuntimeTool baseline capabilities selected by `aw.toml` are mandatory for
this long-running service class. They do not replace Lumen's product
capabilities; search, schema/ops, scale, deployment, observability, backup, and
agent integration remain first-class domain roots.

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| CLI Interface | 4143 | implemented | verified | conformance | ready | mandatory baseline: serve/spec/llm/dockerfile/k8s command surfaces |
| CLI Standard Surface | 1164 | implemented | verified | conformance | ready | mandatory baseline: shared `cli-std` llm/upgrade/issue surface, distinct from Lumen domain commands |
| Chainable Output Conformance | 1142 | implemented | verified | conformance | ready | mandatory baseline: operational CLI outputs emit next/done without wrapping raw artifact/data streams |
| Competitive Search Feature Parity | - | implemented | verified | conformance | ready | mandatory baseline: search-side replacement breadth vs pg/OpenSearch/MongoDB |
| Competitive Search Performance | - | implemented | verified | conformance | ready | mandatory baseline: Lumen-only perf regression passes in vat against retained pg/OpenSearch-calibrated floors |
| Long-Running Stability | - | implemented | verified | dogfood | ready | mandatory baseline: log rebuild, k8s/operator, backup/restore, observability, and soak gates |
| Security Hardening | - | implemented | verified | negative | ready | mandatory baseline: bearer/RBAC/TLS/query safety gates exist |
| HTTP/2 API List | 4143 | implemented | verified | conformance | ready | mandatory baseline: concise HTTP/2 route list plus offline spec/OpenAPI commands |
| Standard Operational Endpoints | 1166 | implemented | verified | conformance | ready | mandatory baseline: one-port `/healthz`, `/readyz`, `/metrics`, `/openapi.json`, `/docs` surface plus offline `lumen spec` evidence |
| EC Gates Configured | 1165 | implemented | verified | conformance | ready | mandatory baseline: aw.toml, vat runners, claim tests, and external-contract claim closure stay wired together |
| Search Core | - | implemented | verified | conformance | ready | domain: pure search index returning ranked external_ids only |
| Lexical Search | - | implemented | verified | conformance | ready | domain: BM25 and analyzer-backed text search |
| Exact & Filter Search | - | implemented | verified | conformance | ready | domain: keyword, number, set, boolean, range, and sorted filters |
| Vector & Hash Search | 4141 | implemented | verified | conformance | ready | domain: CPU vector kNN, filtered kNN, and Hamming hash search |
| Hybrid Search | 4139 | implemented | verified | conformance | ready | domain: lexical+semantic RRF fusion |
| Duplicate & Nested Search | - | implemented | verified | conformance | ready | domain: duplicates, group/has_child/collapse, exists, and CJK substring cases |
| Schema & Ops Lifecycle | - | implemented | verified | conformance | ready | domain: collection DDL, drop-field drain, reindex/replay, stats, and metadata |
| Elastic Scale | - | implemented | verified | conformance | ready | domain: RAM-hot/disk-all columnar mmap segment tier |
| Dynamic Shard Topology | 1179 | implemented | verified | conformance, dogfood | ready | domain: versioned virtual-bucket shard map, bounded snapshot-batch reshard movement, operator-managed shard split policy, and multi-shard kind proof |
| Backup & Restore | - | implemented | verified | conformance | ready | domain: RDB snapshots and bounded cold start |
| Replica Sync & Bootstrap | 1181 | implemented | passing | conformance | ready | domain: raft replica sync semantics plus empty-PVC snapshot/object seed before raft catch-up |
| Observability | - | implemented | verified | conformance | ready | domain: Prometheus metrics, ServiceMonitor/alerts, and opt-in OTLP |
| Kubernetes-Native Deployment | - | implemented | verified | dogfood | ready | domain: kustomize manifests, Lumen CRD, and kube-rs operator |
| Agent Offline Integration | 4143 | implemented | verified | conformance | ready | domain: installed binary self-onboards agents with spec and llm topics |

### CLI Interface

ID: cli-interface
Type: RuntimeTool
Surfaces: CLI: `lumen serve` - long-running search service process.; CLI: `lumen spec` - offline OpenAPI/JSON-schema contract.; CLI: `lumen llm` - offline agent integration topics.; CLI: `lumen dockerfile render` - source/release image artifacts.; CLI: `lumen k8s crd render`, `lumen k8s operator render|run`, and `lumen k8s instance render` - cluster API, control-plane, and app-namespace deployment surfaces.; HTTP: `POST /index`, `POST /search`, `/openapi.json`, `/healthz`, `/readyz`, `/metrics` - binary-served API surface.
EC Dimensions: behavior: `cargo test -p lumen --test spec_cli` - offline CLI contract; API probe/OpenAPI/metrics evidence is tracked by named api_e2e subtests because the full api_e2e suite currently has an unrelated unsupported-sort regression
Root WI: 4143
Status: verified
Required Verification: conformance
Promise:
Expose lumen as one long-running binary with stable service, schema, agent,
OpenAPI, and deployment-facing command surfaces.
Gate Inventory:
- projects/lumen/tests/spec_cli.rs; projects/lumen/tests/api_e2e.rs (health_and_ready, openapi_spec_served, metrics_exposes_prometheus_text); projects/lumen/src/bin/lumen.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| service-process-interface | epic | - | implemented | passing | conformance | projects/lumen/src/bin/lumen.rs<br>projects/lumen/tests/api_e2e.rs |
| lumen-spec-schema-openapi-json-yaml-json-schema-offline | epic | 4143 | implemented | passing | conformance | projects/lumen/tests/spec_cli.rs |
| query-shape-cookbook-field-analyzer-catalog | epic | 4143 | implemented | passing | conformance | projects/lumen/tests/spec_cli.rs |
| lumen-llm-agent-topics-outline-workflow-integration-quickstart-recipes | epic | 4143 | implemented | passing | conformance | projects/lumen/tests/spec_cli.rs |
| deployment-operator-command-surface | epic | - | implemented | passing | conformance | projects/lumen/src/bin/lumen.rs<br>projects/lumen/src/operator |

### CLI Standard Surface

ID: cli-standard-surface
Type: RuntimeTool
Surfaces: CLI: `lumen llm` - shared offline agent self-doc topic surface required by the ecosystem CLI convention.; CLI: `lumen upgrade` - shared self-update and `--check` surface provided through `cli-std`.; CLI: `lumen issue search`, `lumen issue view`, `lumen issue create`, and `lumen issue comment` - shared tracker read/write/follow-up surface scoped to `app:lumen`, separate from Lumen's domain commands.
EC Dimensions: behavior: `cargo test -p lumen --test cli_convention help_ships_standard_issue_group_not_report_issue -- --exact` - top-level help keeps the standard `llm`, `upgrade`, and `issue` groups visible; behavior: `cargo test -p lumen --test cli_convention issue_help_lists_search_view_create_comment -- --exact` - the issue group exposes the shared search/view/create/comment verbs; behavior: `cargo test -p lumen --test cli_convention issue_create_comment_and_upgrade_check_outputs_are_chainable -- --exact` - shared issue create/comment and upgrade check remain runnable in offline smoke mode; behavior: `cargo test -p lumen --test spec_cli llm_outline_maps_agent_topics -- --exact` - the shared `llm` entrypoint still publishes the agent topic set.
Root WI: 1164
Status: verified
Required Verification: conformance
Promise:
Ship the mandatory shared `cli-std` surface every ecosystem CLI owes without
blurring it into Lumen-specific serve/spec/dockerfile/k8s/data-movement
commands.
Gate Inventory:
- projects/lumen/tests/cli_convention.rs; projects/lumen/tests/spec_cli.rs; projects/lumen/src/bin/lumen.rs; libs/cli-std/src/issue.rs; libs/cli-std/src/upgrade.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| shared-llm-entrypoint-surface | epic | 1164 | implemented | passing | conformance | projects/lumen/tests/spec_cli.rs<br>projects/lumen/src/bin/lumen.rs |
| shared-upgrade-check-surface | epic | 1164 | implemented | passing | conformance | projects/lumen/tests/cli_convention.rs<br>libs/cli-std/src/upgrade.rs |
| shared-issue-search-view-create-comment-surface | epic | 1164 | implemented | passing | conformance | projects/lumen/tests/cli_convention.rs<br>libs/cli-std/src/issue.rs |

### Chainable Output Conformance

ID: chainable-output-conformance
Type: RuntimeTool
Surfaces: CLI: `lumen dockerfile render --out`, `lumen k8s crd|operator|instance render --out`, `lumen spec gen --out`, `lumen backup`, `lumen export --out`, `lumen import`, `lumen issue ...`, and `lumen upgrade --check` - operational/artifact-producing commands that expose a runnable continuation or an explicit terminal marker.; CLI: `lumen dockerfile render` without `--out`, `lumen k8s ... render` without `--out`, `lumen export` without `--out`, `lumen spec`, and `lumen llm` - streamed artifact/domain payloads that intentionally stay unwrapped.
EC Dimensions: behavior: `cargo test -p lumen --test cli_convention` - shared chainable harness over the default dry-run/file-writing CLI surfaces; behavior: `cargo test -p lumen --features backup --test cli_convention backup_export_import_outputs_are_chainable -- --exact` - backup/export/import next/terminal markers through the built binary
Root WI: 1142
Status: verified
Required Verification: conformance
Promise:
Keep Lumen's operational CLI outputs lightweight but chainable: file-writing
commands end with a trailing `next: <command>`, machine-readable admin helpers
emit a top-level JSON `next`, and terminal dry-run/read paths end with an
explicit terminal marker. Raw artifact/data streams stay as raw bytes, not AW
envelopes.
Gate Inventory:
- projects/lumen/tests/cli_convention.rs; projects/lumen/src/bin/lumen.rs; libs/cli-std/src/issue.rs; libs/cli-std/src/upgrade.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| artifact-render-follow-ups | epic | 1142 | implemented | passing | conformance | projects/lumen/tests/cli_convention.rs<br>projects/lumen/src/bin/lumen.rs |
| backup-export-import-next-contract | epic | 1142 | implemented | passing | conformance | projects/lumen/tests/cli_convention.rs<br>projects/lumen/src/bin/lumen.rs |
| shared-issue-upgrade-terminal-markers | epic | 1142 | implemented | passing | conformance | projects/lumen/tests/cli_convention.rs<br>libs/cli-std/src/issue.rs<br>libs/cli-std/src/upgrade.rs |

### Competitive Search Feature Parity

ID: competitor-feature-parity
Type: RuntimeTool
Surfaces: HTTP: `POST /index`, `POST /search` - OLTP-derived search API.; Rust API: lumen engine/query planner - search execution over caller-owned external IDs.; CLI: `lumen serve` - hosts the search API.
EC Dimensions: behavior: `cargo test -p lumen` - search planner, field type, query, and API conformance
Root WI: -
Status: verified
Required Verification: conformance
Promise:
Lumen covers the search-side replacement breadth expected from this runtime
class: exact/filter, BM25, vector, hybrid, hash, duplicates, nested/data-table,
schema lifecycle, and API metadata over caller-owned external IDs.
Gate Inventory:
- projects/lumen/tests/planner_diff.rs; projects/lumen/tests/vector_e2e.rs; projects/lumen/tests/hash_hamming.rs; projects/lumen/tests/collapse_nested.rs; projects/lumen/tests/stats_metadata_e2e.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| search-feature-breadth | epic | - | implemented | passing | conformance | projects/lumen/tests/planner_diff.rs<br>projects/lumen/tests/vector_e2e.rs<br>projects/lumen/tests/hash_hamming.rs<br>projects/lumen/tests/collapse_nested.rs |
| query-planner-boolean-eval-roaring-postings | epic | - | implemented | passing | conformance | projects/lumen/tests/planner_diff.rs |
| schema-and-metadata-breadth | epic | - | implemented | passing | conformance | projects/lumen/tests/drop_field_e2e.rs<br>projects/lumen/tests/reindex_stream_e2e.rs<br>projects/lumen/tests/stats_metadata_e2e.rs |

### Competitive Search Performance

ID: competitor-performance
Type: RuntimeTool
Surfaces: Bench: `projects/lumen/scripts/bench_vs_db.py` - pg/OpenSearch/MongoDB comparison.; Bench: `lumen-bench run --types sorted_page_deep` - filter+sort deep-page keyset regression cell.; Rig/Meter: `projects/lumen/vat.toml` and EC efficiency cube - load and resource attribution.; HTTP: `POST /search` - performance-relevant search surface.
EC Dimensions: efficiency: `rig + meter + arena` - latency, throughput, RSS, footprint, and competitor comparison; behavior: `cargo test -p lumen --test perf_gate --test perf_gate_vs_db` - perf gate conformance
Root WI: -
Status: verified
Required Verification: conformance, dogfood
Promise:
Keep lumen's speed and footprint claims tied to ratcheted tests and competitor
comparisons against Postgres/OpenSearch/MongoDB instead of local-only anecdotes.
Gate Inventory:
- projects/lumen/tests/perf_gate.rs; projects/lumen/tests/perf_gate_vs_db.rs; projects/lumen/tests/perf-baseline.json; projects/lumen/src/bin/lumen-bench.rs; projects/lumen/tests/rig/cases/load/data_table_browse.toml; projects/lumen/scripts/bench_vs_db.py; projects/arena/examples/lumen-vs-pg.toml; projects/arena/examples/lumen-vs-opensearch.toml

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| perf-gate-envelope-absolute-latency-throughput-floors | epic | - | implemented | passing | conformance | projects/lumen/tests/perf_gate.rs |
| competitive-regression-gate-beat-pg-os-per-cell-ratcheting | epic | - | implemented | passing | conformance | projects/lumen/tests/perf_gate_vs_db.rs<br>projects/lumen/tests/perf-baseline.json |
| depth-invariant-filter-sort-pagination | change | 10 | implemented | passing | conformance | projects/lumen/src/bin/lumen-bench.rs<br>projects/lumen/tests/perf_gate_vs_db.rs<br>projects/lumen/tests/rig/cases/load/data_table_browse.toml |
| external-pg-and-opensearch-arena-comparison | epic | - | implemented | passing | dogfood | projects/lumen/vat.toml<br>projects/lumen/tests/perf_gate_vs_db.rs<br>projects/lumen/tests/perf-baseline.json<br>projects/arena/examples/lumen-vs-pg.toml<br>projects/arena/examples/lumen-vs-opensearch.toml |

### Long-Running Stability

ID: long-running-stability
Type: RuntimeTool
Surfaces: CLI: `lumen serve` - long-running search service process.; K8s: `projects/lumen/k8s`, `lumen k8s crd/operator/instance`, and `Lumen` operator - declarative deployment and reconcile surface.; HTTP: `/healthz`, `/readyz`, `/metrics` - probes and observability surface.; Log: Lumen WAL / raft-host - rebuildable derived-index mutation stream.
EC Dimensions: stability: `rig` - resilience, endurance, load, and recovery scenarios; behavior: `projects/lumen/scripts/kind-e2e.sh` - k8s/operator dogfood gate
Root WI: -
Status: verified
Required Verification: conformance, dogfood
Promise:
Run as a long-lived derived-index service that rebuilds from the log, survives
pod fault scenarios, exposes usable probes and observability, and keeps
latency/resource behavior stable over soak.
Gate Inventory:
- projects/lumen/tests/rig/cases/resilience; projects/lumen/tests/rig/cases/endurance; projects/lumen/tests/backup_restore_e2e.rs; projects/lumen/scripts/kind-e2e.sh; projects/lumen/k8s; projects/lumen/src/operator

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| log-fan-out-rebuild-from-log | epic | - | implemented | passing | dogfood | projects/lumen/src/raft_sm.rs<br>libs/raft-host/src/host.rs |
| search-p99-survives-fault-and-recovers | epic | - | implemented | passing | dogfood | projects/lumen/tests/rig/cases/resilience |
| graceful-degradation-under-overload | epic | - | implemented | passing | dogfood | projects/lumen/tests/rig/cases/load<br>projects/lumen/tests/rig/config/pins |
| no-fd-socket-thread-leak | epic | - | implemented | passing | dogfood | projects/lumen/tests/rig/cases/endurance |
| no-latency-drift-over-soak | epic | - | implemented | passing | dogfood | projects/lumen/tests/rig/cases/endurance |
| kustomize-base-overlays-hpa | epic | - | implemented | passing | conformance | projects/lumen/k8s |
| lumen-crd-reconcile-loop-kube-rs-operator | epic | - | implemented | passing | conformance | projects/lumen/src/operator<br>projects/lumen/tests/operator_render.rs |
| kind-api-recovery-no-relay | epic | - | implemented | passing | dogfood | projects/lumen/scripts/kind-e2e.sh |
| meta-api-health-ready-metrics-version | epic | - | implemented | passing | conformance | projects/lumen/tests/api_e2e.rs |

### Security Hardening

ID: security-hardening
Type: SecurityTool
Surfaces: HTTP: lumen API - bearer-token auth, RBAC, and query boundary.; Peer transport: rustls/mTLS config - long-running cluster transport security.; Guard: future negative security inventory.
EC Dimensions: security: `guard` - auth/RBAC/query-safety/security findings gate; behavior: `cargo test -p lumen --test auth_e2e --test authz_matrix_e2e` - security behavior conformance
Root WI: -
Status: verified
Required Verification: conformance, negative
Promise:
Keep the long-running search service safe by enforcing API auth/RBAC, preserving
collection/result confidentiality, rejecting unsafe query shapes, and keeping
TLS/mTLS transport configuration testable.
Gate Inventory:
- projects/lumen/tests/auth_e2e.rs; projects/lumen/tests/authz_matrix_e2e.rs; projects/lumen/tests/coverage_gaps_e2e.rs; projects/lumen/src/tls.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| bearer-token-auth-lumen-auth | epic | - | implemented | passing | conformance | projects/lumen/tests/auth_e2e.rs |
| role-based-authz-matrix-per-route | epic | - | implemented | passing | conformance | projects/lumen/tests/authz_matrix_e2e.rs |
| adversarial-query-safety | epic | - | implemented | passing | negative | projects/lumen/tests/coverage_gaps_e2e.rs |
| score-confidentiality | epic | - | implemented | passing | negative | projects/lumen/tests/coverage_gaps_e2e.rs |
| tls-rustls | epic | - | implemented | passing | smoke | `cargo test -p lumen tls`<br>projects/lumen/src/tls.rs |

### HTTP/2 API List

ID: http2-api-list
Type: Service
Surfaces: HTTP: `POST /index`, `POST /search`, collection/schema/stats/reindex/replay routes, `/openapi.json`, `/healthz`, `/readyz`, `/metrics` - concise HTTP/2 API list for clients and operators.; CLI: `lumen spec` and `lumen spec --format openapi-yaml` - offline API/schema inventory.
EC Dimensions: behavior: `cargo test -p lumen --test spec_cli` - offline API/schema inventory; behavior: named `api_e2e` subtests - served OpenAPI, health, readiness, and metrics smoke
Root WI: 4143
Status: verified
Required Verification: conformance
Promise:
Publish Lumen's supported HTTP/2 API surface as a compact endpoint inventory
and offline spec commands, without making OpenAPI completeness the capability
definition.
Gate Inventory:
- projects/lumen/README.md#api-surface; projects/lumen/tests/spec_cli.rs; projects/lumen/tests/api_e2e.rs (health_and_ready, openapi_spec_served, metrics_exposes_prometheus_text)

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| client-search-and-index-route-list | epic | - | implemented | passing | conformance | projects/lumen/README.md#api-surface; projects/lumen/tests/api_e2e.rs |
| ops-metadata-probe-and-metrics-route-list | epic | - | implemented | passing | conformance | projects/lumen/tests/api_e2e.rs |
| offline-spec-openapi-list | epic | 4143 | implemented | passing | conformance | projects/lumen/tests/spec_cli.rs |

### Standard Operational Endpoints

ID: standard-operational-endpoints
Type: Service
Surfaces: HTTP: `/healthz`, `/readyz`, `/metrics`, `/openapi.json`, `/docs` - auth-exempt liveness, readiness, scrape, live-spec, and Swagger UI endpoints served on the same listener as the data plane via `service_http::standard_probe_routes`.; CLI: `lumen spec` and `lumen spec --format openapi-yaml` - offline OpenAPI evidence for the same operational contract when no server is running.
EC Dimensions: behavior: `cargo test -p lumen --test api_e2e health_and_ready -- --exact` - liveness and steady-state readiness surface; behavior: `cargo test -p lumen --test api_e2e readyz_reports_draining -- --exact` - drain flips readiness to 503 while `/healthz` stays live; behavior: `cargo test -p lumen --test api_e2e metrics_exposes_prometheus_text -- --exact` - Prometheus scrape surface; behavior: `cargo test -p lumen --test api_e2e openapi_spec_served -- --exact` - live one-port OpenAPI endpoint; behavior: `cargo test -p lumen --test coverage_gaps_e2e s8_swagger_docs_endpoint_returns_html -- --exact` - Swagger UI is served and points at the live spec; behavior: `cargo test -p lumen --test spec_cli openapi_is_valid_json_with_search_path -- --exact` - offline `lumen spec` emits the same auth-exempt operational route inventory
Root WI: 1166
Status: verified
Required Verification: conformance
Promise:
Expose the standard one-port operational surface the service trait requires:
shared probe, metrics, live-spec, and Swagger UI endpoints stay available on
the main listener, while `lumen spec` mirrors the same OpenAPI contract
offline.
Gate Inventory:
- projects/lumen/src/api.rs; projects/lumen/tests/api_e2e.rs (health_and_ready, readyz_reports_draining, metrics_exposes_prometheus_text, openapi_spec_served); projects/lumen/tests/coverage_gaps_e2e.rs (s8_swagger_docs_endpoint_returns_html); projects/lumen/tests/spec_cli.rs (openapi_is_valid_json_with_search_path)

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| service-http-standard-probe-routes | epic | 1166 | implemented | passing | conformance | projects/lumen/src/api.rs<br>projects/lumen/tests/api_e2e.rs |
| live-openapi-and-swagger-ui-surface | epic | 1166 | implemented | passing | conformance | projects/lumen/tests/api_e2e.rs<br>projects/lumen/tests/coverage_gaps_e2e.rs |
| offline-openapi-matches-operational-surface | epic | 1166 | implemented | passing | conformance | projects/lumen/tests/spec_cli.rs<br>projects/lumen/README.md#openapi |

### EC Gates Configured

ID: ec-gates-configured
Type: Devops
Surfaces: Config: `projects/lumen/aw.toml` - AW EC inventory, generated claim catalog, and dispatch commands for behavior/efficiency/stability verification.; Config: `projects/lumen/vat.toml` - vat-managed `rig-*` and `ec-efficiency*` runners backing the rig and meter EC tools.; Docs: `projects/lumen/external-contracts/claim-closure/production-claims.md` - claim-closure mappings from README promises to executable EC commands.; Tests: `projects/lumen/tests/behavior_lumen_claim_*.rs`, `projects/lumen/tests/efficiency_lumen_claim_*.rs`, `projects/lumen/tests/stability_lumen_claim_*.rs`, and `projects/lumen/tests/security_lumen_claim_*.rs` - generated claim evidence stubs tied back to the EC inventory.
EC Dimensions: behavior: `./target/debug/aw ec check --project lumen` - aw.toml/generated-case inventory stays in sync with claim tests; behavior: `./target/debug/aw ec review --project lumen` - typed capabilities keep required EC dimensions covered; efficiency: `cd projects/lumen && ../../target/debug/vat run ec-efficiency-meter` - meter-wrapped Lumen-only efficiency gate dispatch; stability: `cd projects/lumen && ../../target/debug/vat run rig-resilience` - vat-managed rig stability dispatch
Root WI: 1165
Status: verified
Required Verification: conformance
Promise:
Keep Lumen's service-trait EC baseline explicit and runnable: AW knows where the
claim inventory lives, vat owns the meter/rig gate runners, and
external-contract claim closure maps each production claim to concrete
executable evidence.
Gate Inventory:
- projects/lumen/aw.toml; projects/lumen/vat.toml; projects/lumen/external-contracts/claim-closure/production-claims.md; projects/lumen/tests/behavior_lumen_claim_cli_service_process_interface.rs; projects/lumen/tests/efficiency_lumen_claim_competitor_performance_external_comparison.rs; projects/lumen/tests/stability_lumen_claim_long_running_log_fanout.rs; projects/lumen/tests/security_lumen_claim_security_bearer_auth.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| aw-ec-generated-inventory-and-dispatch | epic | 1165 | implemented | passing | conformance | projects/lumen/aw.toml |
| vat-managed-meter-and-rig-runners | epic | 1165 | implemented | passing | conformance | projects/lumen/vat.toml<br>projects/lumen/tests/rig/cases/resilience<br>projects/lumen/tests/rig/cases/endurance<br>projects/lumen/tests/rig/config/pins |
| external-contract-claim-closure-evidence | epic | 1165 | implemented | passing | conformance | projects/lumen/external-contracts/claim-closure/production-claims.md<br>projects/lumen/tests/behavior_lumen_claim_cli_service_process_interface.rs<br>projects/lumen/tests/efficiency_lumen_claim_competitor_performance_external_comparison.rs<br>projects/lumen/tests/stability_lumen_claim_long_running_log_fanout.rs<br>projects/lumen/tests/security_lumen_claim_security_bearer_auth.rs |

### Search Core

ID: search-core
Type: Service
Surfaces: HTTP: `POST /index` + `POST /search` - client API for indexing caller-owned records and querying ranked external_id results.; CLI: `lumen serve` - search service process.
EC Dimensions: behavior: `cargo test -p lumen --test planner_diff` - query planner conformance
Root WI: -
Status: verified
Required Verification: conformance
Promise:
Input a query with relevance, filters, and sort, and output ranked/sorted
`external_id`s only. Lumen never stores or returns caller documents.
Gate Inventory:
- projects/lumen/tests/planner_diff.rs; projects/lumen/scripts/bench_vs_db.py

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| query-planner-boolean-eval-roaring-postings | epic | - | implemented | passing | conformance | projects/lumen/tests/planner_diff.rs |
| filter-sort-early-termination | epic | - | implemented | passing | conformance | projects/lumen/scripts/bench_vs_db.py<br>projects/lumen/src/bin/lumen-bench.rs<br>projects/lumen/tests/perf_gate_vs_db.rs |

### Lexical Search

ID: lexical-search
Type: Service
Surfaces: HTTP: `POST /search` - text BM25 query surface.; CLI: `lumen serve` - analyzer-backed planner.
EC Dimensions: behavior: `cargo test -p lumen` - BM25 analyzer/ranking conformance; efficiency: `meter` - BM25 search profile
Root WI: -
Status: verified
Required Verification: conformance
Promise:
BM25 ranking over `text`, with tokenization built in through whitespace, ngram,
and jieba analyzers.
Gate Inventory:
- projects/lumen/tests/perf_gate_vs_db.rs; projects/lumen/src/storage.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| bm25-ranking-and-analyzers | epic | - | implemented | passing | conformance | projects/lumen/tests/perf_gate_vs_db.rs<br>projects/lumen/src/storage.rs |

### Exact & Filter Search

ID: exact-filter-search
Type: Service
Surfaces: HTTP: `POST /search` - keyword, number, set, boolean, range, and sort filters.; CLI: `lumen serve` - exact/filter planner.
EC Dimensions: behavior: `cargo test -p lumen` - term/range/set planner conformance; efficiency: `meter` - filter and range profile
Root WI: -
Status: verified
Required Verification: conformance
Promise:
Support keyword terms, number ranges, set membership, boolean composition, and
sort/filter early termination at roaring-bitmap and sorted-column speed.
Gate Inventory:
- projects/lumen/tests/perf_gate_vs_db.rs; projects/lumen/src/storage.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| term-range-set-early-termination | epic | - | implemented | passing | conformance | projects/lumen/tests/perf_gate_vs_db.rs |
| wide-range-filter-index-on-disk-sorted-value-range | epic | - | implemented | passing | conformance | projects/lumen/tests/perf_gate_vs_db.rs<br>projects/lumen/src/storage.rs |

### Vector & Hash Search

ID: vector-hash-search
Type: Service
Surfaces: HTTP: `POST /search` - vector kNN, filtered kNN, and hash Hamming query surface.; CLI: `lumen serve` - vector/hash planner.
EC Dimensions: behavior: `cargo test -p lumen --test vector_e2e --test hash_hamming` - vector/hash conformance; efficiency: `meter` - kNN profile
Root WI: 4141
Status: verified
Required Verification: conformance
Promise:
Index caller-owned embeddings and perceptual/structural hashes, then answer CPU
vector kNN, filter-correct kNN, and Hamming search without owning model
artifacts.
Gate Inventory:
- projects/lumen/tests/vector_e2e.rs; projects/lumen/tests/hash_hamming.rs; projects/lumen/tests/perf_gate_vs_db.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| hnsw-vector-knn-cpu | epic | - | implemented | passing | conformance | projects/lumen/tests/vector_e2e.rs |
| filtered-knn-no-recall-collapse | epic | 4141 | implemented | passing | conformance | projects/lumen/tests/vector_e2e.rs |
| hash-hamming-search | epic | - | implemented | passing | conformance | projects/lumen/tests/hash_hamming.rs |

### Hybrid Search

ID: hybrid-search
Type: Service
Surfaces: HTTP: `POST /search` - RRF hybrid lexical+semantic query surface.; CLI: `lumen serve` - hybrid planner.
EC Dimensions: behavior: `cargo test -p lumen --test hybrid_rrf` - RRF fusion conformance
Root WI: 4139
Status: verified
Required Verification: conformance
Promise:
Fuse lexical BM25 and semantic vector rankings with Reciprocal Rank Fusion,
keeping filters inside each leg so the kNN leg remains filter-correct.
Gate Inventory:
- projects/lumen/tests/hybrid_rrf.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| rrf-fusion-node-planner-integration | epic | 4139 | implemented | passing | conformance | projects/lumen/tests/hybrid_rrf.rs |

### Duplicate & Nested Search

ID: duplicate-nested-search
Type: Service
Surfaces: HTTP: `POST /search` - duplicate, group, has_child, collapse, exists, and CJK substring query surface.; CLI: `lumen serve` - nested/data-table planner.
EC Dimensions: behavior: `cargo test -p lumen --test collapse_nested` - nested planner and data-table conformance
Root WI: -
Status: verified
Required Verification: conformance
Promise:
Cover Airtable-style data tables and duplicate/group use cases with
posting-list-cheap duplicates, nested has_child/group queries, collapse, exists,
and CJK substring search.
Gate Inventory:
- projects/lumen/tests/collapse_nested.rs; projects/lumen/tests/api_e2e.rs; projects/lumen/tests/properties.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| duplicates-group-by | epic | - | implemented | passing | conformance | projects/lumen/tests/api_e2e.rs |
| nested-group-has-child-collapse | epic | - | implemented | passing | conformance | projects/lumen/tests/collapse_nested.rs |

### Schema & Ops Lifecycle

ID: schema-ops-lifecycle
Type: Service
Surfaces: HTTP: collection DDL, drop-field, reindex, replay, stats, and metadata API routes.; CLI: `lumen serve` - schema/ops lifecycle endpoints.
EC Dimensions: behavior: `cargo test -p lumen --test drop_field_e2e --test reindex_stream_e2e --test stats_metadata_e2e` - schema and ops lifecycle conformance
Root WI: -
Status: verified
Required Verification: conformance
Promise:
Provide the operational surface beyond search: collection DDL, online
drop-field drain, reindex/replay stream, and stats/metadata introspection.
Gate Inventory:
- projects/lumen/tests/drop_field_e2e.rs; projects/lumen/tests/drop_drain_e2e.rs; projects/lumen/tests/reindex_stream_e2e.rs; projects/lumen/tests/stats_metadata_e2e.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| schema-ddl-drop-field-drain | epic | - | implemented | passing | conformance | projects/lumen/tests/drop_field_e2e.rs<br>projects/lumen/tests/drop_drain_e2e.rs |
| reindex-replay-stream | epic | - | implemented | passing | conformance | projects/lumen/tests/reindex_stream_e2e.rs |
| stats-metadata | epic | - | implemented | passing | conformance | projects/lumen/tests/stats_metadata_e2e.rs |

### Elastic Scale

ID: elastic-scale
Type: Service
Surfaces: Storage: columnar mmap segment tier - RAM=hot/disk=all storage path.; CLI: `lumen serve` - segment-backed persistence mode.
EC Dimensions: behavior: `cargo test -p lumen --test disk_scale_proof` - disk/RAM boundedness and reopen conformance; efficiency: `meter` - RSS/footprint profile
Root WI: -
Status: verified
Required Verification: conformance
Promise:
Keep hot working sets in RAM while the full indexed corpus lives on disk-backed
columnar mmap segments, with deterministic reopen from local log/checkpoints.
Gate Inventory:
- projects/lumen/tests/disk_scale_proof.rs; projects/lumen/src/storage.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| ram-hot-disk-all-columnar-mmap-segment-tier-embedded-single-node-log | epic | - | implemented | passing | conformance | projects/lumen/tests/disk_scale_proof.rs<br>projects/lumen/src/storage.rs |

### Dynamic Shard Topology

ID: dynamic-shard-topology
Type: Service
Surfaces: CRD/operator: `spec.shardCount`, `spec.replicasPerShard`, `spec.voterCount`, and reshard policy fields - storage ownership and HA topology.; Routing: versioned virtual-bucket map - `bucket = hash(collection_id, routing_key || external_id) % virtualBucketCount`; Search: scatter/gather when no routing key is supplied, targeted shard search when a routing key is supplied.
EC Dimensions: behavior: `cargo test -p lumen --lib routing::tests` - versioned virtual-bucket shard map and bounded reshard batch conformance; behavior: `cargo test -p lumen --features operator --test operator_render` - operator-owned reshard policy, storage topology, status, and shard-map CRD/render conformance; stability: `projects/lumen/scripts/kind-e2e.sh` - live operator dogfood for shardCount=2 with replicasPerShard=1 and replicasPerShard=3
Root WI: 1179
Status: verified
Required Verification: conformance, dogfood
Promise:
Scale storage by moving virtual buckets between physical shards under an
operator-controlled workflow, while keeping replica HA and HPA-driven query
capacity separate from data ownership.
Gate Inventory:
- #1179 dynamic shard topology epic; #1182 versioned virtual-bucket shard map; #1180 operator reshard policy and storage topology control; projects/lumen/src/routing.rs; projects/lumen/src/reshard.rs; projects/lumen/src/operator; projects/lumen/tests/operator_render.rs; projects/lumen/scripts/kind-e2e.sh

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| versioned-virtual-bucket-shard-map | epic | 1182 | implemented | passing | conformance | projects/lumen/src/routing.rs<br>projects/lumen/tests/operator_render.rs |
| storage-pressure-operator-split-policy | epic | 1180 | implemented | passing | conformance | projects/lumen/src/operator<br>projects/lumen/tests/operator_render.rs |
| multi-shard-replica-kind-e2e | epic | 1179 | implemented | passing | dogfood | projects/lumen/scripts/kind-e2e.sh |

### Backup & Restore

ID: backup-restore
Type: Service
Surfaces: CLI: `lumen serve` - snapshot restore and periodic snapshot loop.; Rust API: `LocalFsRdbStore` - local snapshot sink implementation.; Admin/backup path: external snapshot bytes written through service-backup sinks for cold DR seed.
EC Dimensions: behavior: `cargo test -p lumen --test backup_restore_e2e` - snapshot/restore conformance
Root WI: -
Status: verified
Required Verification: conformance
Promise:
Write RDB snapshots to a pluggable sink as a cold-start and disaster-recovery
baseline. Live replicas synchronize through raft log/snapshot mechanics; backup
artifacts seed cold restore and future empty-PVC bootstrap, not normal replica
replication.
Gate Inventory:
- projects/lumen/tests/backup_restore_e2e.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| rdb-snapshot-restore-localfsrdbstore | epic | - | implemented | passing | conformance | projects/lumen/tests/backup_restore_e2e.rs |
| periodic-snapshotter-serve | epic | - | implemented | passing | smoke | projects/lumen/src/bin/lumen.rs |

### Replica Sync & Bootstrap

ID: replica-sync-bootstrap
Type: Service
Surfaces: RaftHost: leader forwarding, append/apply, install snapshot, compaction, and follower catch-up.; Lumen engine state machine: committed `WalRecord` bytes applied into shard-local index state.; Backup/seed: exact `file://` or backup-enabled `s3://bucket/key` SnapshotV1 seed before WAL/raft delta catch-up for empty PVCs.
EC Dimensions: stability: existing raft and backup tests cover live replica convergence and cold restore separately; behavior: `cargo test -p lumen --bin lumen bootstrap_seed_file_restores_snapshot_before_catchup -- --nocapture` - empty-PVC seed restore before catch-up; behavior: `cargo test -p service-backup` - shared exact object fetch contract
Root WI: 1181
Status: verified
Required Verification: conformance, dogfood
Promise:
Make replica behavior agent-readable: existing PVC restarts replay local raft
state/logs, replacement replicas seed from snapshot/object storage before raft
delta catch-up, and disaster recovery restores from external backup without
confusing backup with live replica synchronization.
Gate Inventory:
- #1181 empty-PVC replica bootstrap seed path; projects/lumen/src/bin/lumen.rs; projects/lumen/src/raft.rs; projects/lumen/src/raft_sm.rs; libs/raft-host; libs/service-backup

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| raft-log-replica-sync-existing-pvc | epic | - | implemented | passing | conformance | projects/lumen/src/raft.rs<br>projects/lumen/src/raft_sm.rs<br>libs/raft-host |
| external-backup-disaster-recovery-seed | epic | - | implemented | passing | conformance | projects/lumen/tests/backup_restore_e2e.rs |
| empty-pvc-object-store-seed-before-raft-catch-up | epic | 1181 | implemented | passing | conformance | projects/lumen/src/bin/lumen.rs<br>libs/service-backup/src/source.rs |

### Observability

ID: observability
Type: Devops
Surfaces: HTTP: `/metrics` - Prometheus text-format scrape endpoint.; K8s: ServiceMonitor + PrometheusRule manifests.; Config: `LUMEN_OTLP_ENDPOINT` - opt-in OTLP traces/metrics export.
EC Dimensions: behavior: `cargo test -p lumen` - metrics endpoint and observability wiring conformance
Root WI: -
Status: verified
Required Verification: conformance
Promise:
Expose metrics and telemetry surfaces for long-running operations: Prometheus
pull metrics, kustomize scrape/alert resources, structured logs, and opt-in
OTLP traces/metrics.
Gate Inventory:
- projects/lumen/tests/api_e2e.rs; projects/lumen/k8s/components/observability; projects/lumen/compose.yaml

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| prometheus-metrics-endpoint | epic | - | implemented | passing | smoke | projects/lumen/tests/api_e2e.rs |
| servicemonitor-prometheusrule-bundle | epic | - | implemented | passing | smoke | projects/lumen/k8s/components/observability |
| otlp-traces-and-metrics | epic | - | implemented | passing | conformance | projects/lumen/src/bin/lumen.rs<br>projects/lumen/compose.yaml |

### Kubernetes-Native Deployment

ID: kubernetes-native-deployment
Type: Devops
Surfaces: K8s: `projects/lumen/k8s` - kustomize base, overlays, HPA, PDB, ServiceMonitor.; K8s: `Lumen` CRD + kube-rs operator - declarative reconcile surface.
EC Dimensions: behavior: `cargo test -p lumen --features operator --test operator_render` - offline operator render conformance; stability: `projects/lumen/scripts/kind-e2e.sh` - live operator dogfood
Root WI: -
Status: verified
Required Verification: conformance, dogfood
Promise:
Ship both namespaced kustomize deployment artifacts and a CRD/operator path for
declarative reconcile. The default operator watches one namespace and owns
storage topology, reshard phases, and status conditions for Lumen instances in
that namespace; cluster-wide operation is an optional platform mode. HPA may
scale stateless or near-stateless query/read workers, but never changes shard
ownership.
Gate Inventory:
- projects/lumen/k8s; projects/lumen/src/operator; projects/lumen/tests/operator_render.rs; projects/lumen/scripts/kind-e2e.sh

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| kustomize-base-overlays-hpa | epic | - | implemented | passing | conformance | projects/lumen/k8s |
| lumen-crd-reconcile-loop-kube-rs-operator | epic | - | implemented | passing | conformance | projects/lumen/src/operator<br>projects/lumen/tests/operator_render.rs |
| kind-api-recovery-no-relay | epic | - | implemented | passing | dogfood | projects/lumen/scripts/kind-e2e.sh |
| operator-owned-storage-topology-and-reshard-status | epic | 1180 | implemented | passing | conformance | projects/lumen/src/operator<br>projects/lumen/tests/operator_render.rs |

### Agent Offline Integration

ID: agent-offline-integration
Type: AgentFirst
Surfaces: CLI: `lumen spec` + `lumen spec --format openapi-yaml` + `lumen llm --topic outline` + `lumen llm --topic workflow` + `lumen llm --topic integration` + `lumen llm --topic quickstart` + `lumen llm --topic recipes` - offline self-description and agent onboarding commands.
EC Dimensions: behavior: `cargo test -p lumen --test spec_cli` - offline schema and LLM topic conformance
Root WI: 4143
Status: verified
Required Verification: conformance
Promise:
An installed `lumen` binary self-onboards an agent offline: `lumen spec` emits
machine schemas and query catalogs, while `lumen llm --topic <topic>` emits
workflow, integration, quickstart, recipes, and non-goal topics.
Gate Inventory:
- projects/lumen/tests/spec_cli.rs; projects/lumen/src/spec.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| lumen-spec-schema-openapi-json-yaml-json-schema-offline | epic | - | implemented | passing | conformance | projects/lumen/tests/spec_cli.rs |
| query-shape-cookbook-field-analyzer-catalog | epic | - | implemented | passing | conformance | projects/lumen/tests/spec_cli.rs |
| lumen-llm-agent-topics-outline-workflow-integration-quickstart-recipes | epic | 4143 | implemented | passing | conformance | projects/lumen/tests/spec_cli.rs |

## Benchmarks

### Performance contract — enforced & ratcheting

Beating Postgres and OpenSearch on search is a **standing CI commitment, not a
one-time measurement**: `tests/perf_gate_vs_db.rs` drives lumen, Postgres
(`tokio-postgres`) and OpenSearch (`reqwest`) against one byte-identical corpus
and **fails the build** if lumen loses any *gated* search cell. The authoritative
thresholds live in **`tests/perf-baseline.json`**; full methodology, per-tier
numbers, resource columns, and reproduction live in
**[`docs/benchmarks-scale.md`](docs/benchmarks-scale.md)**.

How the comparison stays honest (separate metrics, never conflated):

- **End-to-end, single-client** is the gated metric — lumen and OpenSearch share
  HTTP/JSON so the transport tax cancels. pg's binary wire beats HTTP/JSON on
  cheap btree point/range lookups on loopback, so those cells are **HTTP-EXEMPT**
  (annotated) and gated instead through a **native prepared-binary** path (Rust
  wire over Unix socket) — the cheap predicates still carry a hard floor.
- **Concurrent qps (10/100/1000)** and **write-path qps** are report-only by
  default; `LUMEN_GATE_COMPARE_PEERS=1 LUMEN_PERF_STRICT=1` strict-gates the peer
  rows recorded in `perf-baseline.json`. Co-located CI keeps them report-only
  until CPU isolation; isolated-host repeats are the release-stable bar.

Each cell carries a threshold in `perf-baseline.json`: a **WIN cell** must hold
`max(1.0, 0.8 × recorded margin)` — a **ratchet**, so improving a cell locks the
new bar and it can only get better. **HTTP-EXEMPT cells** (pg btree lookups on
loopback) are separately gated by `pg_native` floors through the native path.
**Scale tiers:** 1K smoke/trend, **10K routine AW/release regression**,
**100K explicit release-local calibration**, and 1M release-soak/research only.
The historical 1M proof is retained evidence; refresh it only with an explicit
soak (`LUMEN_GATE_RELEASE_SOAK=1` or `LUMEN_GATE_N=1000000`).

**Current status — GREEN** (routine gate defaults to 10K Lumen-only regression;
retained historical N=1M in-memory + disk-tier peer evidence). Representative
serial search margins (full set, qps 10/100/1000 tiers, and history in
[`docs/benchmarks-scale.md`](docs/benchmarks-scale.md) / `perf-baseline.json`):

| Cell | vs Postgres | vs OpenSearch (in-mem) | vs OpenSearch (disk) |
|---|---:|---:|---:|
| `text_bm25` | 815× | 4.5× | 23.0× |
| `text_and` | 96.9× | 7.7× | 10.9× |
| `filtered_search` | 61.4× | 7.3× | 4.6× |
| `filter_sort` | 43.9× | 4.1× | 6.0× |
| `pure_sort` | 83.6× | 3.9× | 5.2× |
| `kw_term` | EXEMPT¹ | 4.0× | 9.3× |
| `range` | EXEMPT¹ | 5.2× | 11.3× |
| `bool_filter` | EXEMPT¹ | 5.2× | 6.6× |

¹ pg cheap btree predicates are HTTP-EXEMPT; gated via the native prepared-binary
path — `kw_term` 6.2×, `range` 2.9×, `bool_filter` 39.6× vs pg prepared Unix socket.
Every OpenSearch cell holds a 3.0× WIN baseline (2.4× floor after the ratchet);
paced qps tiers stay ahead of OpenSearch on every WIN cell.

**Write path** — `tests/write_qps.rs` drives the real HTTP `POST /index`; the
legacy NATS/JetStream row remains the historical write-path comparison while
the serving/operator HA path uses Lumen-owned raft. Latest historical 100-worker JetStream run: **8.5× vs
Postgres**, **3.4× vs OpenSearch**, 0 errors. `LUMEN_PERF_STRICT=1` strict-gates
the write margins only when peer services are explicitly present; per-mode
numbers and tuning history live in `benchmarks-scale.md`.

### Footprint & stability

- **Index ~28.8 bytes/doc at 1M** — 5–7× smaller on disk than Postgres /
  OpenSearch; reported as a first-class disk-size metric alongside
  `pg_total_relation_size` and OpenSearch `_stats/store`.
- **RAM=hot/disk=all proven** (`tests/disk_scale_proof.rs`): a reopened
  collection's resident growth is ~30–47% of full-in-RAM and **does not grow with
  N** (forward payload demand-paged off the mmap).
- **Resident ~168 MB vs OpenSearch ~1.4 GB** (~8× smaller); tail p99
  `text_bm25` **1.0 ms** vs OpenSearch ~18 ms (no GC vs JVM pauses).
- **Stability:** 2M sustained searches held RSS flat with zero failed/errored/
  timed-out requests (Rust, no GC; mmap'd segments demand-paged by the kernel).

Full row-count x qps scaling, footprint tables, and retained vs-pg / vs-OS
breakdowns live in **[`docs/benchmarks-scale.md`](docs/benchmarks-scale.md)**.
Routine checks use the Lumen-only vat runner; peer comparisons are refreshed
only through explicit calibration/soak runners when a benchmark cell or peer
configuration changes.

## Data model

There are exactly three concepts on the wire:

| Concept       | What it is                                                |
|---------------|-----------------------------------------------------------|
| `Collection`  | A namespace + a schema (a map of field name → field type) |
| `Field`       | One typed column inside a collection                      |
| `external_id` | An opaque string chosen by the caller; lumen never mints it |

There is **no `Document`**. lumen does not store original field values
beyond what the inverted index needs to answer search and duplicate
queries. Hydrating search hits back to full records is the caller's
responsibility against its own store.

## Field types

Schema-first DDL. The declared `FieldType` deterministically picks the
index structure — there is no separate "index options" knob and no
auto-inference.

| FieldType | Index built on write                                                          | Query support              | Duplicate detection |
|-----------|-------------------------------------------------------------------------------|----------------------------|---------------------|
| `text`    | Tokenized inverted index (`token → sorted posting`); analyzer per field       | `match` (BM25, bag-of-words) | No                  |
| `keyword` | Exact inverted index (whole value as one term)                                | `term`, `terms`            | Yes                 |
| `number`  | Sorted inverted index (range-scannable)                                       | `term`, `range`            | Yes                 |
| `set`     | Multi-keyword (one posting per element)                                       | `term` (matches any element) | Yes (per element) |
| `vector`  | Dense `[f32; dim]` + ANN graph (HNSW CPU default; exact flat CPU brute-force) | `knn { vector, k }` with `cosine` / `dot` / `l2` metric | No |
| `hash`    | Caller-supplied 64-bit perceptual/structural hash stored as hex bits         | `hamming { hash, max_distance }` | No; use `hamming` for near-duplicate lookup |

Analyzers available for `text`: `jieba` (Chinese), `whitespace_lower`
(English / generic), `ngram` (configurable min/max). A field is bound
to one analyzer at declaration time.

A field cannot be both `text` and `keyword`. If both are needed (e.g.
"search by email substring *and* find duplicate emails"), declare two
fields and write twice — this keeps write amplification predictable.

## Search concept boundaries

The parity promise is search-side breadth over Lumen's declared contract, not
an implicit claim that every PostGIS/OpenSearch/MongoDB search feature already
exists. These concepts are explicit so agents can choose the right engine or
adapter boundary:

| Concept | Disposition |
|---------|-------------|
| Geo / spatial search | **Roadmap candidate.** Use PostGIS/MongoDB/OpenSearch or a caller-owned geospatial prefilter today, then pass matching `external_id`s to lumen. |
| Phrase / proximity queries | **Roadmap candidate.** Current `match` is bag-of-words BM25 over analyzer tokens, not phrase order or slop. |
| Fuzzy / typo tolerance | **Roadmap candidate.** No edit-distance automaton today; for coarse prefix/substring recall, use the `ngram` analyzer recipe. |
| Synonyms | **Caller-owned.** Expand queries before calling lumen or write normalized companion fields; there is no managed synonym dictionary/analyzer. |
| Autocomplete / suggest | **Recipe.** Declare a dedicated `text` field with `analyzer: "ngram"` and run `match`; lumen returns candidate `external_id`s, not suggestion payloads. |
| Highlighting | **Non-goal.** Search responses contain only `external_id` + `score`; lumen does not store source text to return snippets/fragments. |
| Per-field / per-clause boost | **Boundary.** No arbitrary boost knob today; use separate fields/query legs plus `rrf`, then rerank in the caller if needed. |
| Document TTL / expiry | **Caller-owned lifecycle.** Delete/reindex expired `external_id`s from the source-of-truth event stream; collection soft-delete grace is not per-document TTL. |

## API surface

All endpoints are HTTP/2 JSON. The authoritative request / response
schemas are served by a running pod at `GET /openapi.json`. Offline
codegen pipes that spec out of the `lumen-openapi-dump` binary; see
[OpenAPI](#openapi) below.

### Schema (DDL)

```
PUT /collections/{id}
{
  "fields": {
    "bio":       { "type": "text",    "analyzer": "jieba" },
    "email":     { "type": "keyword" },
    "tags":      { "type": "keyword", "multi": true },
    "age":       { "type": "number" },
    "embedding": { "type": "vector",  "dim": 768, "metric": "cosine",
                   "backend": "hnsw-cpu", "quantize": "sq" },
    "avatar_phash": { "type": "hash" }
  }
}
→ 200 { "collection_id": "users", "version": 1, "fields_count": 6 }
```

Online: adding a new field is immediate (postings start empty).
Re-declaring an existing field with the same spec is a no-op (PUT is
upsert-merge). Changing a field's type is rejected — drop the field
(`DELETE /collections/{id}/fields/{name}`) and re-add. `vector` field
configuration (`dim` / `metric` / `backend` / `quantize`) is immutable
for the field's lifetime. `hash` has no schema-time hash-kind parameter:
the caller computes pHash, SimHash, b-bit MinHash, or another 64-bit signature
and writes it as a 16-hex-character string (optional `0x` prefix accepted).

### Index (write)

```
POST /collections/{id}/index
{
  "items": [
    { "external_id": "u_123", "field": "bio",   "value": "senior engineer in Taipei" },
    { "external_id": "u_123", "field": "email", "value": "a@x.com" },
    { "external_id": "u_123", "field": "tags",  "value": ["rust","db"] },
    { "external_id": "u_123", "field": "avatar_phash", "value": "f0e1d2c3b4a59687" }
  ],
  "request_id": "..."        // optional, dedup TTL 5 min
}
→ 200 { "indexed": 4, "bytes_written": { "bio": 412, "email": 33, "tags": 88, "avatar_phash": 12 }, "shard_lag_ms": 4 }
```

Re-writing `(external_id, field)` fully re-indexes that field. There
is no partial update.

### Delete

```
DELETE /collections/{id}/index/{external_id}             → 204    # all fields
DELETE /collections/{id}/index/{external_id}?field=bio   → 204    # one field
```

### Search

```
POST /collections/{id}/search
{
  "query": {
    "and": [
      { "match": { "field": "bio",  "text": "engineer taipei", "op": "and" } },
      { "term":  { "field": "tags", "value": "rust" } },
      { "range": { "field": "age",  "gte": 25, "lt": 40 } }
    ]
  },
  "limit": 20,
  "cursor": null
}
→ 200 {
  "hits": [
    { "external_id": "u_123", "score": 4.21 },
    { "external_id": "u_087", "score": 3.95 }
  ],
  "total": 217,        // estimate; ">10000" when truncated
  "cursor": "eyJvZmZzZXQiOjIwfQ==",
  "took_ms": 6
}
```

Search responses **only carry `external_id` + `score`** — never field
values. There is no `_source`.

**Pagination is keyset (search-after), depth-invariant.** The `cursor` is an
opaque token bound to the query that produced it: echo it back unchanged to
get the next page. For sorted (single number field) and score-ranked results
the token carries the LAST hit's position, so every page **seeks** —
O(log n) on the sorted index — instead of skipping; deep pages cost the same
as page 1 (measured at depth 50k over 100k docs: 86µs vs 28.7ms offset
skip). Stop when `cursor` is null. Legacy `{"offset":N}` tokens keep working
(O(offset) skip). Note: when continuing from a keyset cursor with
`track_total: true`, `total` counts the REMAINING matches from the cursor,
not the full set — read the full total off the first page.

### Duplicates

```
POST /collections/{id}/duplicates
{ "field": "email", "min_group_size": 2, "limit": 100 }
→ 200 {
  "groups": [
    { "value": "a@x.com", "external_ids": ["u_123","u_456","u_789"] },
    { "value": "b@y.com", "external_ids": ["u_201","u_990"] }
  ],
  "truncated": false,
  "took_ms": 12
}
```

`text` / `vector` fields do not support duplicates (semantics undefined).

### Exists / Duplicated (presence & collision filters)

Two query nodes for presence and collision. Both compose inside `and` / `or` /
`not` like any other leaf, so arbitrary combinations ("non-blank email **and**
duplicate phone") need no bespoke endpoint.

```
POST /collections/{id}/search
{
  "query": {
    "and": [
      { "exists":     { "field": "email" } },                      // email is non-blank
      { "duplicated": { "field": "phone", "min_group_size": 2 } }  // phone collides with another doc
    ]
  }
}
```

| Node | Matches |
|------|---------|
| `exists` | docs holding any value for `field`; `not exists` = "is empty" |
| `duplicated` | docs whose `field` value is shared by ≥ `min_group_size` docs (`min_group_size` defaults to / floors at 2) |

Both cover `keyword` / `number` / `set` fields. `text` / `vector` / `hash` are
rejected (presence/equality is undefined there — declare a `keyword` companion
field for a text "is empty" / duplicate filter).

`duplicated` vs the `/duplicates` endpoint: the endpoint returns *grouped*
results (`value → external_ids`) for an audit view; the `duplicated` query node
returns a *flat, composable* doc set you can intersect with other predicates in
one search.

### kNN (vector search)

```
POST /collections/{id}/search
{
  "query": {
    "knn": {
      "field": "embedding",
      "vector": [0.12, -0.04, ...],
      "k": 10
    }
  },
  "limit": 10
}
→ 200 {
  "hits": [
    { "external_id": "u_123", "score": 0.94 },
    { "external_id": "u_087", "score": 0.91 }
  ],
  "total": 10,
  "took_ms": 3
}
```

Scores are direction-normalised so higher = better regardless of
metric (`cosine` / `dot` use the raw similarity; `l2` reports
negated distance). `knn` can be composed inside `and` / `or` /
`not` with the other query nodes.

### Schema lifecycle

```
PUT    /collections/{id}                          # create or upsert-extend
DELETE /collections/{id}/fields/{field_name}      # online field drop
DELETE /collections/{id}                          # soft-delete (24h grace)
DELETE /collections/{id}?force=true               # immediate physical drop
GET    /collections                               # list (filtered by RBAC)
```

### Admin & ops

```
GET  /admin/backup                                # full SnapshotV1 JSON dump
POST /admin/restore                               # replace state from a snapshot
POST /admin/backup/local                          # snapshot → LocalFsSink (path + prefix)
GET  /debug/cluster                               # pod/shard/role/peers/replication-lag
GET  /metrics                                     # Prometheus text format
GET  /healthz                                     # liveness
GET  /readyz                                      # readiness (503 while draining)
GET  /openapi.json                                # live OpenAPI spec
GET  /docs                                        # Swagger UI (interactive "Try it out")
```

### Stats

Engine **metadata** about one collection. Per the v1 non-goals, this
describes the *index* (size, cardinality, cache health) — not the
caller's data. There are no aggregations here.

```
GET /collections/{id}/stats
→ 200 {
  "documents_indexed": 1234567,
  "fields": {
    "email": { "type": "keyword", "unique_terms": 1233110, "bytes": 40128830 },
    "bio":   { "type": "text",    "unique_terms": 482113,  "bytes": 32108920, "avg_doc_len": 28.4 },
    "age":   { "type": "number",  "unique_terms": 81,      "bytes": 9876543 }
  },
  "storage": { "total_bytes": 82114293 },
  "cache":   { "posting_hit_ratio": 0.87 },
  "last_indexed_at": "2026-05-28T16:42:11Z"
}
```

`last_indexed_at` is the typical "did my writes land?" probe — caller
writes N docs, then asserts `documents_indexed == N` and
`last_indexed_at` advanced. For Prometheus-shaped continuous
monitoring, `/metrics` carries the same numbers as gauges.

## HTTP & clients

The client API speaks **HTTP/1.1 and HTTP/2 cleartext (h2c) on the same
port** (`auto`) — the server accepts both, no flag needed. **HTTP/2 is the
recommended connection for serving**: one connection multiplexes many concurrent
streams, which is how lumen sustains its high-QPS search/index throughput. The
three setups, in order of preference:

- **Production (behind TLS) — HTTP/2 by default, for free.** An ingress / mesh
  terminating TLS negotiates h2 via ALPN, so every client gets it transparently.
  This is the recommended deployment.
- **Cleartext (dev / in-cluster) — h2c is opt-in.** h2c can't auto-negotiate (no
  ALPN), so a client must enable prior-knowledge (see table). A lumen connection
  *pool* over h2c is what the benchmark throughput numbers use.
- **Zero-driver fallback — plain HTTP/1.1 always works**, no special client:
  `requests`, `httpx`, `fetch`, `curl`, any REST client (lumen ships no client
  SDK — it's pure REST/OpenAPI; see `lumen llm`).

| Client | HTTP/1.1 | h2c (cleartext) opt-in | h2 over TLS (prod) |
|--------|----------|------------------------|--------------------|
| Python `requests` | ✅ | ✗ (no h2 support) | ✗ |
| Python `httpx` | ✅ | `pip install "httpx[http2]"` + `Client(http2=True)` | ✅ ALPN |
| `curl` | ✅ | `--http2-prior-knowledge` | `--http2` |
| Go `net/http` | ✅ | needs `x/net/http2` h2c transport | ✅ ALPN |
| browser (Swagger `/docs`) | ✅ | ✗ (browsers require TLS) | ✅ ALPN |

### Auth

Production deployments should run the server with:

```env
LUMEN_AUTH=required
LUMEN_TOKEN_REGISTRY_FILE=/var/run/secrets/lumen/token-registry.json
```

The registry file is a JSON map of bearer token to subject/roles, mounted from a
Kubernetes Secret. On GKE, keep GCP Secret Manager as the source of truth and
materialize that file through External Secrets Operator or Secret Store CSI.
Lumen reads the registry at startup; token rotation should roll the serving pods
or use a Secret reloader controller.

`token-registry.json` shape:

```json
{
  "admin-token": {
    "subject": "platform-admin",
    "roles": { "*": "admin" }
  },
  "product-reader-token": {
    "subject": "products-reader",
    "roles": { "products": "read" }
  }
}
```

Role values are `read`, `write`, and `admin`; `*` grants across all collections.
Clients only need:

```env
LUMEN_URL=http://lumen.<namespace>.svc.cluster.local:7373
LUMEN_TOKEN=<token>
```

and send `Authorization: Bearer <token>` on API requests. Probe/spec/scrape
routes (`/healthz`, `/readyz`, `/metrics`, `/openapi.json`, `/docs`) stay
auth-exempt.

## OpenAPI

| Artefact              | When to use                                                  |
|-----------------------|--------------------------------------------------------------|
| `GET /openapi.json`   | Live spec from a running pod — codegen against an actual env |
| `GET /docs`           | Interactive Swagger UI ("Try it out")                        |
| `lumen spec`          | Offline OpenAPI JSON from the installed binary               |
| `lumen spec --format openapi-yaml` | Offline OpenAPI YAML for agent review         |
| `lumen spec --format json-schema` | Component schemas plus operational schemas such as `TokenRegistry` |
| `lumen spec gen --lang ts\|py\|rust --out <dir>` | In-tree typed client generation |

`lumen spec` and the live endpoint generate from the same Rust code
(`#[derive(utoipa::OpenApi)]` on `api::ApiDoc`). There is no committed OpenAPI
snapshot; the binary and live endpoint are the source of truth.

Generated Python clients include pydantic models plus a stdlib HTTP/2 runtime.
For auth-enabled deployments:

```python
from generated_api import Client

client = Client("http://lumen.default.svc.cluster.local:7373", auth_token="...")
```

`default_headers={"Authorization": "Bearer ..."}` is also supported. The
generated `h2c_runtime.py` exposes unary `request()` and bidirectional
`stream()` APIs; Lumen's current OpenAPI routes are unary, so generated
`client.py` uses `request()` today and the streaming surface is forward-looking
runtime capacity for services that add streaming operations.

## Design notes (from the retired HA.md, 2026-07)

Durable decisions folded from the retired `HA.md`; its session-era "Original
design notes (openraft)" framing was already superseded by the shipped
`raft-core`/`raft-host` implementation and is dropped as historical.

lumen is a **log-replicated, derived, rebuildable search index**: the caller
still owns the source of truth, and lumen indexes the caller's `external_id`s.
The deployment boundary changed once `libs/raft-core` existed: multi-pod lumen
owns its own write ordering and replica synchronization instead of requiring
an external broker as the default HA path. Mode split:

- **standalone**: one pod, embedded WAL, direct apply.
- **primary-replica**: multiple lumen pods, `raft-core` elects a leader, the
  leader owns the ordered write log, and followers replicate/apply the same
  raw `WalRecord::encode()` bytes.

`lumen serve --wal auto` is the production default: it starts embedded when no
k8s replica topology is present, and switches to raft when
`REPLICAS_PER_SHARD > 1` is injected by the operator/StatefulSet. The storage
topology contract is `totalPods = shardCount * replicasPerShard`:
`replicasPerShard` selects the HA mode for each shard group, while
`shardCount` selects how many physical storage shards own the corpus. A
deployment with `shardCount > 1` and `replicasPerShard = 1` is sharded but not
raft-replicated; a deployment with `shardCount = 1` and `replicasPerShard > 1`
is one shard with raft replicas. StatefulSet pod ordinals map deterministically
to `shardIndex = ordinal % shardCount` and
`replicaIndex = ordinal / shardCount`.

The operator never passes special cluster flags — topology comes from the
downward API (`POD_NAME`, `POD_NAMESPACE`, `SHARD_COUNT`,
`REPLICAS_PER_SHARD`, `VOTER_COUNT`, `LUMEN_HEADLESS_SERVICE`): one serving pod
renders a standalone Deployment + HPA, `replicasPerShard > 1` renders stable
serving StatefulSets + headless Services. For local multi-node work,
`LUMEN_PEERS=host:port,...` overrides headless DNS so several
`lumen serve --wal raft` processes can run on one machine.

Dynamic shard growth is an operator workflow, not a direct HPA response. The
default routing contract uses virtual buckets:
`bucket = hash(collection_id, routing_key || external_id) % virtualBucketCount`,
then a versioned bucket-to-physical-shard map decides ownership. Search without
a routing key scatters/gathers across shards; search with a routing key can
target one shard. Operators should prepare a split around storage pressure
(for example 50% of the configured shard ceiling), start or recommend split
based on growth and safety windows, treat high utilization as urgent, and avoid
auto-split when the max shard size or max shard count is unknown.

Raft responsibility is split by crate/module: `libs/raft-core` (consensus
state machine and log semantics), `libs/raft-host` (h2c peer transport,
leader forwarding, snapshot install, log compaction — snapshot upload/pruning
policy lives in `libs/service-backup`), `projects/lumen/src/raft_sm.rs`
(committed write records → engine mutations, snapshot produce/restore), and
`projects/lumen/src/raft.rs` (API-facing cluster/debug DTOs, read-consistency
parsing). Legacy broker-backed write logs are not part of the Lumen
deployment archetype; the NATS backend is compatibility/test surface only, and
Relay WAL support has been removed from Lumen.

Bootstrap modes are intentionally distinct. A restarted pod with its PVC
replays local raft state, snapshots, and logs. A new empty-PVC replica can catch
up through leader snapshot install and AppendEntries today, but the production
path is to seed from object-store/shard snapshot first and then catch up the
raft delta, with operator-visible progress and rate limits. External backup is
the cold disaster-recovery and seed surface; it is not the normal live replica
synchronization path.
