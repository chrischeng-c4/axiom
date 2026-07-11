---
id: lumen-production-claim-closure-ec
summary: Production claim closure mappings for Lumen capability claims that are already covered by existing tests, vat/rig runs, kustomize builds, or tool evidence.
fill_sections: [e2e-test]
---

# EC: Production Claim Closure

These EC cases map README production claims to existing executable proof
commands. Several claims intentionally share a command: AW claim closure treats
the same passing command as evidence for every production claim it covers.

## External Contracts
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: lumen-claim-cli-service-process-interface
    capability_id: cli-interface
    claim_id: service-process-interface
    contract_id: cli-service-process-interface
    category: behavior
    command: "cargo test -p lumen --test api_e2e -- --nocapture"
    assertions:
      - "The long-running service exposes health, readiness, version, metrics, indexing, and search routes through the binary-served API."
  - id: lumen-claim-cli-deployment-operator-command-surface
    capability_id: cli-interface
    claim_id: deployment-operator-command-surface
    contract_id: cli-deployment-operator-command-surface
    category: behavior
    command: "cargo test -p lumen --features operator --test operator_render -- --nocapture"
    assertions:
      - "The operator-facing command surface renders CRD and serving objects used by the deployment path."

  - id: lumen-claim-cli-standard-llm-entrypoint
    capability_id: cli-standard-surface
    claim_id: shared-llm-entrypoint-surface
    contract_id: cli-standard-llm-entrypoint
    category: behavior
    command: "cargo test -p lumen --test spec_cli llm_outline_maps_agent_topics -- --exact --nocapture"
    assertions:
      - "The shared `lumen llm` entrypoint publishes the agent topic set through the standard CLI convention."
  - id: lumen-claim-cli-standard-upgrade-check
    capability_id: cli-standard-surface
    claim_id: shared-upgrade-check-surface
    contract_id: cli-standard-upgrade-check
    category: behavior
    command: "cargo test -p lumen --test cli_convention issue_create_comment_and_upgrade_check_outputs_are_chainable -- --exact --nocapture"
    assertions:
      - "The shared upgrade check surface remains available and emits an explicit terminal marker."
  - id: lumen-claim-cli-standard-issue-surface
    capability_id: cli-standard-surface
    claim_id: shared-issue-search-view-create-comment-surface
    contract_id: cli-standard-issue-surface
    category: behavior
    command: "cargo test -p lumen --test cli_convention issue_help_lists_search_view_create_comment -- --exact --nocapture"
    assertions:
      - "The shared issue group exposes search, view, create, and comment under `lumen issue`."

  - id: lumen-claim-chainable-artifact-render-follow-ups
    capability_id: chainable-output-conformance
    claim_id: artifact-render-follow-ups
    contract_id: chainable-artifact-render-follow-ups
    category: behavior
    command: "cargo test -p lumen --test cli_convention chainable_output_next_line_file_writing_vs_stream -- --exact --nocapture"
    assertions:
      - "Artifact render commands write exactly one runnable trailing `next:` line when writing files and keep stream mode raw."
  - id: lumen-claim-chainable-backup-export-import-next
    capability_id: chainable-output-conformance
    claim_id: backup-export-import-next-contract
    contract_id: chainable-backup-export-import-next
    category: behavior
    command: "cargo test -p lumen --features backup --test cli_convention backup_export_import_outputs_are_chainable -- --exact --nocapture"
    assertions:
      - "Backup, export, and import helpers emit machine-readable next or terminal markers through the built binary."
  - id: lumen-claim-chainable-issue-upgrade-terminal
    capability_id: chainable-output-conformance
    claim_id: shared-issue-upgrade-terminal-markers
    contract_id: chainable-issue-upgrade-terminal
    category: behavior
    command: "cargo test -p lumen --test cli_convention issue_create_comment_and_upgrade_check_outputs_are_chainable -- --exact --nocapture"
    assertions:
      - "Shared issue dry-run paths and upgrade check terminate with explicit `next: done` markers."

  - id: lumen-claim-ec-generated-inventory-dispatch
    capability_id: ec-gates-configured
    claim_id: aw-ec-generated-inventory-and-dispatch
    contract_id: ec-generated-inventory-dispatch
    category: behavior
    command: "./target/debug/aw ec check --project lumen"
    assertions:
      - "The generated AW EC inventory stays in sync with claim tests and dispatch commands."
  - id: lumen-claim-ec-vat-managed-runners
    capability_id: ec-gates-configured
    claim_id: vat-managed-meter-and-rig-runners
    contract_id: ec-vat-managed-runners
    category: behavior
    command: "cd projects/lumen && ../../target/debug/vat run ec-efficiency-meter"
    assertions:
      - "The vat-managed meter runner remains executable for Lumen efficiency EC dispatch."
  - id: lumen-claim-ec-claim-closure-evidence
    capability_id: ec-gates-configured
    claim_id: external-contract-claim-closure-evidence
    contract_id: ec-claim-closure-evidence
    category: behavior
    command: "./target/debug/aw ec check --project lumen"
    assertions:
      - "The production claim-closure document remains synchronized with generated EC cases."

  - id: lumen-claim-competitor-feature-search-breadth
    capability_id: competitor-feature-parity
    claim_id: search-feature-breadth
    contract_id: competitor-feature-search-breadth
    category: behavior
    command: "cargo test -p lumen --test api_e2e --test vector_e2e --test hash_hamming --test collapse_nested -- --nocapture"
    assertions:
      - "The API, vector, hash, duplicate, and nested search surfaces execute correctly across the replacement feature set."
  - id: lumen-claim-competitor-feature-schema-metadata
    capability_id: competitor-feature-parity
    claim_id: schema-and-metadata-breadth
    contract_id: competitor-feature-schema-metadata
    category: behavior
    command: "cargo test -p lumen --test drop_field_e2e --test reindex_stream_e2e --test stats_metadata_e2e -- --nocapture"
    assertions:
      - "Schema lifecycle, reindex/replay, and stats/metadata behavior pass the production conformance tests."

  - id: lumen-claim-competitor-performance-envelope
    capability_id: competitor-performance
    claim_id: perf-gate-envelope-absolute-latency-throughput-floors
    contract_id: competitor-performance-envelope
    category: efficiency
    command: "cargo test -p lumen --test perf_gate -- --nocapture"
    assertions:
      - "Absolute latency and throughput floors stay within the ratcheted perf gate envelope."
  - id: lumen-claim-competitor-performance-external-comparison
    capability_id: competitor-performance
    claim_id: external-pg-and-opensearch-arena-comparison
    contract_id: competitor-performance-external-comparison
    category: efficiency
    command: "cd projects/lumen && ../../target/debug/vat run ec-efficiency-meter"
    assertions:
      - "The vat efficiency runner executes the Lumen-only regression path against retained Postgres/OpenSearch-calibrated floors; explicit calibration runners refresh peers only on demand."
  - id: lumen-claim-competitor-performance-depth-invariant
    capability_id: competitor-performance
    claim_id: depth-invariant-filter-sort-pagination
    contract_id: competitor-performance-depth-invariant
    category: efficiency
    command: "cargo test -p lumen --test lumen_bench_cli --test perf_gate_vs_db -- --nocapture"
    assertions:
      - "The Lumen-only deep-page and filter/sort perf gates stay depth-invariant against the retained calibrated floors without rerunning peer databases by default."

  - id: lumen-claim-long-running-log-fanout
    capability_id: long-running-stability
    claim_id: log-fan-out-rebuild-from-log
    contract_id: long-running-log-fanout
    category: stability
    command: "cargo test -p lumen --test wal_nats_e2e -- --nocapture"
    assertions:
      - "A late or second node can replay the published write stream and converge with live writes; shard-group topology is now dogfooded by the operator kind profiles."
  - id: lumen-claim-long-running-kustomize-base
    capability_id: long-running-stability
    claim_id: kustomize-base-overlays-hpa
    contract_id: long-running-kustomize-base-overlays
    category: behavior
    command: "kustomize build projects/lumen/k8s/base && kustomize build projects/lumen/k8s/overlays/dev && kustomize build projects/lumen/k8s/overlays/staging && kustomize build projects/lumen/k8s/overlays/prod && kustomize build projects/lumen/k8s/operator"
    assertions:
      - "The base, dev, staging, prod, and operator kustomize surfaces render valid Kubernetes manifests."
  - id: lumen-claim-long-running-stateless-kind
    capability_id: long-running-stability
    claim_id: kind-api-recovery-no-relay
    contract_id: long-running-stateless-kind-dogfood
    category: stability
    command: "projects/lumen/scripts/kind-e2e.sh"
    assertions:
      - "The live kind dogfood path runs Lumen only, without building or deploying Relay, and proves the serving API recovers after a pod restart; operator mode also proves shardCount=2 with replicasPerShard=1 and replicasPerShard=3 storage topology."

  - id: lumen-claim-security-bearer-auth
    capability_id: security-hardening
    claim_id: bearer-token-auth-lumen-auth
    contract_id: security-bearer-auth
    category: security
    command: "cargo test -p lumen --test auth_e2e --test authz_matrix_e2e -- --nocapture"
    assertions:
      - "Bearer-token auth rejects invalid callers and accepts valid tokens under LUMEN_AUTH=required."
  - id: lumen-claim-security-rbac-matrix
    capability_id: security-hardening
    claim_id: role-based-authz-matrix-per-route
    contract_id: security-rbac-matrix
    category: security
    command: "cargo test -p lumen --test authz_matrix_e2e --test api_e2e -- --nocapture"
    assertions:
      - "Per-route RBAC enforces read/write/admin permissions and bounds result/page sizes."
  - id: lumen-claim-security-query-safety
    capability_id: security-hardening
    claim_id: adversarial-query-safety
    contract_id: security-query-safety
    category: security
    command: "cargo test -p lumen --test coverage_gaps_e2e search_security_query_injection_rejects_bad_queries -- --nocapture"
    assertions:
      - "Malformed, deeply nested, and adversarial query shapes remain bounded and do not panic."
  - id: lumen-claim-security-score-confidentiality
    capability_id: security-hardening
    claim_id: score-confidentiality
    contract_id: security-score-confidentiality
    category: security
    command: "cargo test -p lumen --test coverage_gaps_e2e search_security_result_leak_respects_collection_boundaries -- --nocapture"
    assertions:
      - "Scores and hit existence do not leak across collection boundaries."
  - id: lumen-claim-security-tls-rustls
    capability_id: security-hardening
    claim_id: tls-rustls
    contract_id: security-tls-rustls
    category: security
    command: "cargo test -p lumen --lib tls"
    assertions:
      - "The rustls-backed TLS surface passes the runtime TLS gate."

  - id: lumen-claim-http2-client-route-list
    capability_id: http2-api-list
    claim_id: client-search-and-index-route-list
    contract_id: http2-client-route-list
    category: behavior
    command: "cargo test -p lumen --test api_e2e -- --nocapture"
    assertions:
      - "Search and index HTTP routes are exposed and exercised through the binary-served API tests."
  - id: lumen-claim-http2-ops-route-list
    capability_id: http2-api-list
    claim_id: ops-metadata-probe-and-metrics-route-list
    contract_id: http2-ops-route-list
    category: behavior
    command: "cargo test -p lumen --test api_e2e -- --nocapture"
    assertions:
      - "Health, readiness, OpenAPI, metrics, and version routes are exposed and exercised."
  - id: lumen-claim-http2-offline-spec-list
    capability_id: http2-api-list
    claim_id: offline-spec-openapi-list
    contract_id: http2-offline-spec-list
    category: behavior
    command: "cargo test -p lumen --test spec_cli -- --nocapture"
    assertions:
      - "The offline spec commands publish the supported HTTP API inventory."

  - id: lumen-claim-standard-service-probe-routes
    capability_id: standard-operational-endpoints
    claim_id: service-http-standard-probe-routes
    contract_id: standard-service-probe-routes
    category: behavior
    command: "cargo test -p lumen --test api_e2e -- --nocapture"
    assertions:
      - "The service exposes health, readiness, version, metrics, indexing, and search routes through the binary-served API."
  - id: lumen-claim-standard-live-openapi-swagger
    capability_id: standard-operational-endpoints
    claim_id: live-openapi-and-swagger-ui-surface
    contract_id: standard-live-openapi-swagger
    category: behavior
    command: "cargo test -p lumen --test api_e2e openapi_spec_served -- --exact --nocapture && cargo test -p lumen --test coverage_gaps_e2e s8_swagger_docs_endpoint_returns_html -- --exact --nocapture"
    assertions:
      - "The live service serves OpenAPI JSON and Swagger UI against the operational route surface."
  - id: lumen-claim-standard-offline-openapi
    capability_id: standard-operational-endpoints
    claim_id: offline-openapi-matches-operational-surface
    contract_id: standard-offline-openapi
    category: behavior
    command: "cargo test -p lumen --test spec_cli openapi_is_valid_json_with_search_path -- --exact --nocapture"
    assertions:
      - "The offline `lumen spec` OpenAPI output remains valid and includes the operational search route."

  - id: lumen-claim-search-core-planner
    capability_id: search-core
    claim_id: query-planner-boolean-eval-roaring-postings
    contract_id: search-core-planner
    category: behavior
    command: "cargo test -p lumen --test planner_diff -- --nocapture"
    assertions:
      - "The planner keeps boolean evaluation and roaring-posting behavior aligned with brute-force expectations."
  - id: lumen-claim-search-core-filter-sort
    capability_id: search-core
    claim_id: filter-sort-early-termination
    contract_id: search-core-filter-sort
    category: efficiency
    command: "cargo test -p lumen --test perf_gate_vs_db -- --nocapture"
    assertions:
      - "Filter/sort early-termination behavior is covered by the ratcheted database comparison gate."

  - id: lumen-claim-lexical-bm25
    capability_id: lexical-search
    claim_id: bm25-ranking-and-analyzers
    contract_id: lexical-bm25-ranking-analyzers
    category: behavior
    command: "cargo test -p lumen --test perf_gate_vs_db -- --nocapture"
    assertions:
      - "BM25 ranking and analyzer behavior pass the ratcheted performance/conformance comparison."

  - id: lumen-claim-exact-term-range-set
    capability_id: exact-filter-search
    claim_id: term-range-set-early-termination
    contract_id: exact-term-range-set
    category: behavior
    command: "cargo test -p lumen --test perf_gate_vs_db -- --nocapture"
    assertions:
      - "Term, range, and set filter behavior stays within the exact/filter search gate."
  - id: lumen-claim-exact-wide-range-filter
    capability_id: exact-filter-search
    claim_id: wide-range-filter-index-on-disk-sorted-value-range
    contract_id: exact-wide-range-filter
    category: behavior
    command: "cargo test -p lumen --test perf_gate_vs_db -- --nocapture"
    assertions:
      - "Wide range filters over sorted disk-backed values pass the exact/filter gate."

  - id: lumen-claim-vector-hnsw
    capability_id: vector-hash-search
    claim_id: hnsw-vector-knn-cpu
    contract_id: vector-hnsw-cpu
    category: behavior
    command: "cargo test -p lumen --test vector_e2e -- --nocapture"
    assertions:
      - "CPU vector kNN returns ordered nearest-neighbor results and preserves restore behavior."
  - id: lumen-claim-vector-filtered-knn
    capability_id: vector-hash-search
    claim_id: filtered-knn-no-recall-collapse
    contract_id: vector-filtered-knn
    category: behavior
    command: "cargo test -p lumen --test vector_e2e -- --nocapture"
    assertions:
      - "Filtered kNN returns the nearest vector within the filter without recall collapse."
  - id: lumen-claim-vector-hash-hamming
    capability_id: vector-hash-search
    claim_id: hash-hamming-search
    contract_id: hash-hamming-search
    category: behavior
    command: "cargo test -p lumen --test hash_hamming -- --nocapture"
    assertions:
      - "Hash Hamming search returns bounded-distance matches over the hash index."

  - id: lumen-claim-hybrid-rrf
    capability_id: hybrid-search
    claim_id: rrf-fusion-node-planner-integration
    contract_id: hybrid-rrf-planner
    category: behavior
    command: "cargo test -p lumen --test hybrid_rrf -- --nocapture"
    assertions:
      - "Lexical and semantic result lists are fused through RRF while preserving per-leg filters."

  - id: lumen-claim-duplicates-group-by
    capability_id: duplicate-nested-search
    claim_id: duplicates-group-by
    contract_id: duplicates-group-by
    category: behavior
    command: "cargo test -p lumen --test api_e2e duplicates_finds_groups -- --exact --nocapture"
    assertions:
      - "Duplicate detection returns groups of external IDs sharing a field value."
  - id: lumen-claim-nested-collapse
    capability_id: duplicate-nested-search
    claim_id: nested-group-has-child-collapse
    contract_id: nested-group-has-child-collapse
    category: behavior
    command: "cargo test -p lumen --test collapse_nested -- --nocapture"
    assertions:
      - "Nested has_child/group/collapse behavior passes the data-table search tests."

  - id: lumen-claim-schema-ddl-drop-field
    capability_id: schema-ops-lifecycle
    claim_id: schema-ddl-drop-field-drain
    contract_id: schema-ddl-drop-field-drain
    category: behavior
    command: "cargo test -p lumen --test drop_field_e2e --test drop_drain_e2e -- --nocapture"
    assertions:
      - "Collection DDL, online drop-field drain, and drain readiness semantics pass."
  - id: lumen-claim-schema-reindex-replay
    capability_id: schema-ops-lifecycle
    claim_id: reindex-replay-stream
    contract_id: schema-reindex-replay
    category: behavior
    command: "cargo test -p lumen --test reindex_stream_e2e -- --nocapture"
    assertions:
      - "Reindex/replay stream behavior indexes items and reports progress/errors correctly."
  - id: lumen-claim-schema-stats-metadata
    capability_id: schema-ops-lifecycle
    claim_id: stats-metadata
    contract_id: schema-stats-metadata
    category: behavior
    command: "cargo test -p lumen --test stats_metadata_e2e -- --nocapture"
    assertions:
      - "Stats and per-field metadata match indexed data and byte attribution."

  - id: lumen-claim-elastic-disk-tier
    capability_id: elastic-scale
    claim_id: ram-hot-disk-all-columnar-mmap-segment-tier-embedded-single-node-log
    contract_id: elastic-disk-tier
    category: efficiency
    command: "target/debug/meter test -- -p lumen --test disk_scale_proof -- --ignored"
    assertions:
      - "The disk-scale proof keeps the full corpus on disk-backed segments while bounded hot state remains in memory."

  - id: lumen-claim-backup-rdb-store
    capability_id: backup-restore
    claim_id: rdb-snapshot-restore-localfsrdbstore
    contract_id: backup-rdb-store
    category: behavior
    command: "cargo test -p lumen --test backup_restore_e2e -- --nocapture"
    assertions:
      - "RDB snapshots restore through the LocalFsRdbStore baseline as a cold restore and future bootstrap seed surface."
  - id: lumen-claim-backup-periodic-snapshotter
    capability_id: backup-restore
    claim_id: periodic-snapshotter-serve
    contract_id: backup-periodic-snapshotter
    category: behavior
    command: "cargo test -p lumen --test backup_restore_e2e -- --nocapture"
    assertions:
      - "The serving process snapshot loop and restore path remain covered by the backup/restore e2e gate; live replica synchronization remains raft-owned."

  - id: lumen-claim-observability-prometheus-metrics
    capability_id: observability
    claim_id: prometheus-metrics-endpoint
    contract_id: observability-prometheus-metrics
    category: behavior
    command: "cargo test -p lumen --test api_e2e metrics_exposes_prometheus_text -- --exact --nocapture"
    assertions:
      - "The /metrics endpoint emits Prometheus text with the expected scrape content type."
  - id: lumen-claim-observability-servicemonitor-rule
    capability_id: observability
    claim_id: servicemonitor-prometheusrule-bundle
    contract_id: observability-servicemonitor-rule
    category: behavior
    command: "kustomize build projects/lumen/k8s/overlays/prod"
    assertions:
      - "The production overlay renders the ServiceMonitor and PrometheusRule bundle."
  - id: lumen-claim-observability-otlp
    capability_id: observability
    claim_id: otlp-traces-and-metrics
    contract_id: observability-otlp
    category: behavior
    command: "cargo test -p lumen --test api_e2e -- --nocapture"
    assertions:
      - "The serving process keeps observability configuration and metadata routes wired."

  - id: lumen-claim-k8s-kustomize-base
    capability_id: kubernetes-native-deployment
    claim_id: kustomize-base-overlays-hpa
    contract_id: k8s-kustomize-base-overlays
    category: behavior
    command: "kustomize build projects/lumen/k8s/base && kustomize build projects/lumen/k8s/overlays/dev && kustomize build projects/lumen/k8s/overlays/staging && kustomize build projects/lumen/k8s/overlays/prod && kustomize build projects/lumen/k8s/operator"
    assertions:
      - "The Kubernetes base, overlays, HPA/PDB, and operator manifests render successfully."
  - id: lumen-claim-k8s-operator-reconcile
    capability_id: kubernetes-native-deployment
    claim_id: lumen-crd-reconcile-loop-kube-rs-operator
    contract_id: k8s-operator-reconcile
    category: behavior
    command: "cargo test -p lumen --features operator --test operator_render -- --nocapture"
    assertions:
      - "The kube-rs operator render path proves rendering topology conformance: Lumen CRD inputs map to serving resources, including storage-pressure reshard policy, status phases, and fixed storage topology (rendering only — the live reconcile loop, reshard driver, and admin verbs are covered by the dedicated reshard-durability gate)."
  - id: lumen-claim-k8s-operator-storage-topology-reshard
    capability_id: kubernetes-native-deployment
    claim_id: operator-owned-storage-topology-and-reshard-status
    contract_id: k8s-operator-storage-topology-reshard
    category: behavior
    command: "cargo test -p lumen --features operator --test operator_render -- --nocapture"
    assertions:
      - "The operator render gate proves rendering topology conformance: fixed StatefulSet storage topology and reshard status exposure (rendering only — reshard driver execution, admin verbs, and migration durability are covered by the dedicated reshard-durability gate)."
  - id: lumen-claim-k8s-stateless-kind
    capability_id: kubernetes-native-deployment
    claim_id: kind-api-recovery-no-relay
    contract_id: k8s-stateless-kind-dogfood
    category: stability
    command: "projects/lumen/scripts/kind-e2e.sh"
    assertions:
      - "The live kind dogfood path runs Lumen only, without building or deploying Relay, and proves the serving API recovers after a pod restart; operator mode also proves shardCount=2 with replicasPerShard=1 and replicasPerShard=3 storage topology."

  - id: lumen-claim-dynamic-versioned-virtual-bucket-map
    capability_id: dynamic-shard-topology
    claim_id: versioned-virtual-bucket-shard-map
    contract_id: dynamic-versioned-virtual-bucket-map
    category: behavior
    command: "cargo test -p lumen --lib routing -- --nocapture"
    assertions:
      - "Versioned virtual-bucket routing remains the stable shard ownership contract."
  - id: lumen-claim-dynamic-storage-pressure-split-policy
    capability_id: dynamic-shard-topology
    claim_id: storage-pressure-operator-split-policy
    contract_id: dynamic-storage-pressure-split-policy
    category: behavior
    command: "cargo test -p lumen --features operator --test operator_render -- --nocapture"
    assertions:
      - "The operator render gate proves rendering topology conformance: storage-pressure reshard recommendations compute correctly without changing HPA-owned serving scale (rendering only — reshard driver execution, admin verbs, and migration durability are covered by the dedicated reshard-durability gate)."
  - id: lumen-claim-dynamic-multi-shard-replica-kind
    capability_id: dynamic-shard-topology
    claim_id: multi-shard-replica-kind-e2e
    contract_id: dynamic-multi-shard-replica-kind
    category: stability
    command: "projects/lumen/scripts/kind-e2e.sh"
    assertions:
      - "The live kind dogfood path covers multi-shard and replicated-shard operator profiles."
  - id: lumen-claim-dynamic-reshard-durability
    capability_id: dynamic-shard-topology
    claim_id: reshard-driver-admin-verb-and-cold-start-migration-durability
    contract_id: dynamic-reshard-durability
    category: behavior
    command: "cargo test -p lumen --features operator --test reshard_driver_e2e -- --nocapture && cargo test -p lumen --test reshard_admin_e2e -- --nocapture && cargo test -p lumen --lib segment_rdb -- --nocapture"
    assertions:
      - "The checkpointed reshard phase driver (state machine, checkpoint-gated cutover, no-transition guard), the four reshard/backup admin verbs (scoped export, additive apply, source eviction, on-demand checkpoint) including idempotency and auth, and cold-start durability of applied/evicted reshard mutations all pass — keeping the migration-path regression class #1389 fixed under standing gate."

  - id: lumen-claim-topology-empty-pvc-bootstrap-seed
    capability_id: replica-sync-bootstrap
    claim_id: empty-pvc-object-store-seed-before-raft-catch-up
    contract_id: topology-empty-pvc-bootstrap-seed
    category: behavior
    command: "cargo test -p lumen --bin lumen bootstrap_seed_file_restores_snapshot_before_catchup -- --nocapture"
    assertions:
      - "A fresh serving process restores a configured SnapshotV1 seed before WAL or raft catch-up."

  - id: lumen-claim-agent-offline-spec
    capability_id: agent-offline-integration
    claim_id: lumen-spec-schema-openapi-json-yaml-json-schema-offline
    contract_id: agent-offline-spec
    category: behavior
    command: "cargo test -p lumen --test spec_cli -- --nocapture"
    assertions:
      - "Offline schema commands produce valid OpenAPI JSON/YAML and JSON-schema output for agents."
  - id: lumen-claim-agent-query-catalog
    capability_id: agent-offline-integration
    claim_id: query-shape-cookbook-field-analyzer-catalog
    contract_id: agent-query-catalog
    category: behavior
    command: "cargo test -p lumen --test spec_cli -- --nocapture"
    assertions:
      - "The offline query-shape and field/analyzer catalogs remain deterministic for agent ingestion."
  - id: lumen-claim-agent-llm-topics
    capability_id: agent-offline-integration
    claim_id: lumen-llm-agent-topics-outline-workflow-integration-quickstart-recipes
    contract_id: agent-llm-topics
    category: behavior
    command: "cargo test -p lumen --test spec_cli -- --nocapture"
    assertions:
      - "The offline LLM outline, workflow, integration, quickstart, and recipe topics remain available."
```
