# Competitive Landscape
<!-- type: analysis lang: markdown -->

## Thesis

`cclab-ctx-inf-db` occupies an empty quadrant in the database market: a single-node embeddable database that combines **bitemporal graph storage**, **GPU-accelerated graph analytics**, **confidence-scored relations**, and a **declarative inference engine**, targeted at open-source intelligence (OSINT) and context-inference workloads.

Every component exists somewhere; no existing system combines all four with first-class provenance for intelligence analysis.

## Feature matrix — who has what

<!-- type: comparison lang: markdown -->

| System             | Graph | Bitemporal | GPU analytics | Inference | Confidence | Provenance | Embeddable | Rust-native |
|--------------------|:-----:|:----------:|:-------------:|:---------:|:----------:|:----------:|:----------:|:-----------:|
| **ctx-inf-db**     | ✅    | ✅         | ✅ (Phase 4)  | ✅ (Phase 5) | ✅       | ✅         | ✅         | ✅          |
| Neo4j              | ✅    | ⚠ via plugin | ❌         | ⚠ Cypher only | ❌    | ⚠ manual   | ❌         | ❌          |
| ArangoDB           | ✅    | ❌         | ❌            | ⚠ AQL only | ❌       | ⚠ manual   | ❌         | ❌          |
| Kuzu               | ✅    | ❌         | ❌            | ❌        | ❌         | ❌         | ✅         | ❌ (C++)    |
| SurrealDB          | ✅    | ⚠ time-travel beta | ❌    | ⚠ SurrealQL | ❌      | ❌         | ✅         | ✅          |
| Datomic            | ⚠ EAV | ✅         | ❌            | ✅ Datalog | ❌        | ✅ audit   | ❌ (JVM)   | ❌          |
| TerminusDB         | ✅ RDF | ✅        | ❌            | ⚠ WOQL    | ❌         | ✅ Git-like | ❌        | ❌          |
| XTDB / Crux        | ⚠ RDF | ✅         | ❌            | ✅ Datalog | ❌        | ⚠         | ❌ (JVM)   | ❌          |
| JanusGraph         | ✅    | ❌         | ❌            | ⚠ Gremlin | ❌        | ❌         | ❌ distributed | ❌      |
| TimescaleDB        | ❌    | ⚠ time-based | ❌          | ❌        | ❌         | ❌         | ❌         | ❌          |
| DuckDB             | ⚠ ext | ❌         | ⚠ experimental | ❌     | ❌         | ❌         | ✅         | ❌          |
| Apache AGE (PG)    | ✅    | ⚠ via PG   | ❌            | ⚠ Cypher  | ❌         | ❌         | ❌         | ❌          |
| Blazegraph / RDF   | ✅    | ⚠ reification | ❌         | ✅ SPARQL | ❌         | ⚠ named graph | ❌     | ❌          |
| cuGraph (RAPIDS)   | ✅    | ❌         | ✅            | ❌        | ❌         | ❌         | ⚠ library  | ❌          |
| Memgraph           | ✅    | ❌         | ❌            | ⚠ Cypher  | ❌         | ❌         | ✅         | ❌          |

Legend: ✅ native first-class · ⚠ partial / via plugin / beta · ❌ absent

### Rows that come closest

- **XTDB / Datomic**: bitemporal + Datalog inference. Missing: GPU, confidence scores, Rust, embeddable single-binary.
- **TerminusDB**: bitemporal + graph + Git-like versioning. Missing: GPU, confidence, inference beyond WOQL.
- **Kuzu**: embeddable graph DB with good analytics perf. Missing: everything temporal and inference.
- **Neo4j + GDS plugin**: mature graph + some analytics. Missing: bitemporal, confidence, embeddable, GPU.

No row scores ≥ 6/8. `ctx-inf-db` targets 8/8 by Phase 5.

## Why the combination matters for OSINT

OSINT / intelligence analysis workloads have four characteristics that existing systems fit poorly:

### 1. Facts are dated *twice*
- **valid_time**: when the thing happened in the real world ("Alice worked at Acme from 2020 to 2022")
- **transaction_time**: when we learned about it ("we recorded this fact on 2024-03-01; on 2024-05-01 we corrected the end date to 2021")

Uni-temporal DBs lose the audit trail when records are corrected. Bitemporal preserves both dimensions — essential for "what did we believe at the time of decision X" queries.

### 2. Facts have confidence, not just truth
A source says "Bob funded Charlie." Is that source reliable? Was it single-sourced or corroborated? Most graph DBs treat edges as boolean (exists / doesn't). OSINT needs `confidence: f64` on every edge, propagated through derivations.

### 3. Graph analytics dominate the working set
Centrality, community detection, pattern matching over 10M–100M-entity graphs. CPU-only analytics start to bottleneck in the 1M–10M range. GPU-accelerated PageRank / BFS / matrix ops (cuGraph-style) give 10–100× speedup. But cuGraph is a Python library, not a DB — you have to export, compute, import. ctx-inf-db bakes GPU into the engine.

### 4. Derived facts matter as much as asserted facts
Analyst doesn't just store raw facts; they derive new claims via rules:
- "If X met with Y 3+ times in 48h, flag as high-contact."
- "If organization A funds organization B, and B funds C, then A transitively funds C."

These are inference rules. Datomic has Datalog; most other DBs don't. ctx-inf-db's Phase 5 inference engine is the first-class path for this.

### 5. Provenance is mandatory, not optional
Every fact must trace to a source document, with extraction method and confidence. Most DBs treat provenance as metadata ("add a field"); ctx-inf-db treats it as a required, structured attribute (see [design-decisions.md §D4](./design-decisions.md#d4--structured-provenance-vecsourceref-on-every-fact-)).

## Non-goals (by design)

We are explicitly **not** trying to compete with:

- **Distributed graph DBs** (JanusGraph, DGraph, Neptune) — single-node only; scale is vertical (big box with GPU), not horizontal cluster
- **General-purpose OLTP** (PostgreSQL, MySQL) — not a transactional record-of-truth; can be a sink, not the source
- **Data warehouses** (Snowflake, BigQuery) — no SQL, no columnar analytics
- **Vector / embedding DBs** (Pinecone, Weaviate) — no vector search; embeddings are out of scope (could integrate externally)
- **Full-text search engines** (Elasticsearch, Tantivy) — only tokenized name search; offload full text to external engine
- **RDF triple stores for semantic web** (Virtuoso, GraphDB) — not RDF-compatible; uses native Entity/Relation model

These boundaries keep the scope tractable for a single-node Rust crate.

## Target use cases

| Use case                          | How ctx-inf-db fits                                                                 |
|-----------------------------------|-------------------------------------------------------------------------------------|
| Journalistic investigation        | Entity graphs with sources; "who funded whom, and when did we learn?" queries       |
| Threat intelligence               | IOC → actor → campaign mapping with confidence; inference rules for attribution     |
| Corporate due-diligence           | Ownership / board / policy graphs with valid_time (when roles changed) and tx_time  |
| Compliance / sanctions screening  | Relationship-of-interest detection with provenance chains                           |
| Academic research (network sci)   | Bitemporal social networks; GPU-accelerated community detection over time slices    |
| Cyber-physical systems forensics  | Temporal event correlation with inference rules for anomaly detection               |

## Key design differentiators (cross-reference to [design-decisions.md](./design-decisions.md))

- **D1 — Bitemporality**: valid_time + transaction_time on every entity and relation (not just entities)
- **D2 — Unified query + rule language**: Datalog-subset used for both ad-hoc queries and inference rules (no two-language problem)
- **D3 — Confidence math**: product-rule propagation by default, per-rule override; confidence is a first-class field, not a property
- **D4 — Structured provenance**: `Vec<SourceRef>` on every fact, not a blob of properties
- **D5 — GPU as engine tier, not add-on**: VRAM is the fourth storage tier (alongside RAM/disk/WAL), scheduled by the buffer manager

## Open questions

- **Multi-user concurrency**: Single writer (current DashMap) vs. MVCC with snapshot isolation — defer to Phase 3+
- **Replication**: WAL shipping to a read replica (standby for BI / dashboards) — possible Phase 6, not priority
- **Secondary indexes**: Beyond adjacency + type + temporal, do we need composite indexes for high-cardinality property queries? Likely yes in Phase 3
- **Python-compatible bindings via Mamba**: Would expand audience dramatically; tracked separately from core crate
