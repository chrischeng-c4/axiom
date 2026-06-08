# Design Decisions
<!-- type: decision-log lang: markdown -->

This log captures foundational design decisions that affect the whole crate. Decisions are numbered D1, D2, ... and referenced by ID from other specs.

Status legend:
- 🔒 **locked** — implemented or near-term blocker; changing requires a migration
- 🟡 **tentative** — rationale set, open to revision before implementation
- ⬜ **open** — tracked but not yet decided

## D1 — Bitemporal: valid_time + transaction_time 🟡

### Context

The current `TemporalRange { valid_from, valid_to }` is **uni-temporal** — it tracks when a fact is true in the world (valid_time / application_time) but not when the system learned of the fact (transaction_time / system_time).

For OSINT workloads, both dimensions matter:
- "Alice worked at Acme from 2020 to 2022" — valid_time
- "We first recorded this on 2024-03-01; corrected the end date on 2024-05-01" — transaction_time

Uni-temporal systems lose the audit trail on correction. Bitemporal preserves "what did we believe at time T?" queries — essential for decisions based on a prior knowledge state.

### Options

| Option | Model | Pros | Cons |
|--------|-------|------|------|
| A — Uni-temporal (current) | `valid_from, valid_to` | Simple; matches human intuition; ~½ storage | Corrections destroy history; no audit |
| B — Bitemporal | Add `tx_from, tx_to`; updates create new rows with old row's `tx_to = now` | Standard SQL:2011; non-destructive corrections; "as-of" queries | ~2× storage; every query adds tx_time predicate; UX complexity |
| C — Event-sourced (Datomic-style) | Facts are immutable datoms with `tx_time` only; retractions are datoms | Perfect audit; unified model; time-travel trivial | Forces rewrite; queries must reconstruct state; high write amplification for updates |

### Decision

**Adopt Option B — bitemporal.** Add `tx_from: DateTime<Utc>` and `tx_to: Option<DateTime<Utc>>` to Entity and Relation.

Defaults:
- On create: `tx_from = Utc::now(), tx_to = None` (open-ended — this row is current)
- On update: old row gets `tx_to = Utc::now()`; new row created with `tx_from = Utc::now(), tx_to = None`, sharing the same `id` but with incremented `version`
- Queries add an optional `as_of_tx: Option<DateTime<Utc>>`; default is "current" (rows where `tx_to = None`)

Valid-time (`valid_from / valid_to`) continues as today — it captures real-world validity.

### Rationale

- Audit / chain-of-custody is a first-class requirement for OSINT (D4 provenance depends on this)
- Option C's event-sourced model would force rewriting the DashMap-primary hot path; too invasive for a Phase 2.5 introduction
- Doubled storage is acceptable — WAL + snapshot already absorb this cost; RAM hot path holds only `tx_to = None` rows
- Standard SQL:2011 semantics means the model is well-understood; ports to Datalog queries cleanly

### Impact

- **Phase 2.5 Buffer Pool**: page format must reserve slots for `tx_from, tx_to`
- **Phase 3 Query API**: every query takes optional `as_of_tx`; default is current
- **Phase 3 Temporal Index**: consider separate interval trees for valid_time and tx_time, or composite 2D structure
- **Existing Phase 1/2 code**: `update_entity` gains the bitemporal step (old-row freeze + new-row insert); current in-place mutation is dropped
- **Migration**: existing persisted snapshots (if any) get `tx_from = snapshot.created_at, tx_to = None` on load

### Open questions

- Should `valid_to = None` mean "still valid" or "indefinite"? (Current code: "indefinite". Bitemporal convention: "still valid until revised".) — lean toward current code's semantics
- Do we need a third "decision_time" (when an analyst acted on the fact)? — out of scope for v1; can be modeled as a property if needed

---

## D2 — Unified query + rule language: Datalog subset 🟡

### Context

Two surfaces need a query language:
1. **Ad-hoc queries** from analysts / applications (currently a fluent Rust builder, aspirational `query-api.md`)
2. **Inference rules** in Phase 5 (currently unspecified)

Historically, DBs use different languages for each (SQL for queries, Datalog / rules / stored procedures for inference). This creates the "two-language problem": analysts must learn both; rules can't easily be tested as queries; the engine does double implementation work.

### Options

| Option | Language | Pros | Cons |
|--------|----------|------|------|
| A — Fluent Rust API only | `engine.find_entities().of_type(...).active_at(...)` | Type-safe; zero parsing | Rust clients only; no external tooling; inference needs separate path |
| B — Cypher-subset | `MATCH (a:Person)-[:MET_WITH]->(b) WHERE ...` | Familiar; huge ecosystem | Full parser non-trivial; weak recursion; doesn't unify with rules |
| C — SPARQL-subset | `SELECT ?a ?b WHERE { ?a :met-with ?b . ... }` | Standard; federation-friendly | RDF-centric; verbose; doesn't unify with rules |
| D — Datalog subset | `met_with(?a, ?b), type(?a, :Person), active_at(?a, $t)` | Rules and queries same language; recursion first-class; proven scalable | Less familiar; needs parser + evaluator |

### Decision

**Adopt Option D — Datalog subset — for external query + inference surface.** Keep Option A (fluent Rust) as the internal / in-process API.

The Datalog subset supports:
- Extensional predicates (EDB): `entity(?id, ?type, ?name)`, `relation(?rid, ?source, ?target, ?type)`, `active_at(?id, $t)`
- Intensional predicates (IDB): user-defined via rules — `high_contact(?a, ?b) :- met_with(?a, ?b), met_with(?a, ?b), met_with(?a, ?b), within_48h(...)`
- Conjunction, negation-as-failure, stratified recursion
- Aggregation (`count`, `sum`, `min`, `max`) as extensions
- **Explicit temporal operators**: `as_of_valid(?t)`, `as_of_tx(?t)`, `during(?from, ?to)`

Queries are just single-rule programs: `?- met_with(alice, ?b), active_at(?b, t).`

### Rationale

- Unifies query + rule evaluation — one engine, one language, shared optimizer
- Datalog recursion is a natural fit for transitive-closure queries (`ancestor`, `funded_by_transitively`) that Cypher handles awkwardly and SQL handles verbosely
- Stratified negation is simpler to implement correctly than general Cypher semantics
- Confidence scores (D3) attach cleanly to Datalog: each derived tuple carries a confidence computed from its premises
- Parser can start tiny (~500 lines) and grow; no SPARQL/Cypher spec compliance burden

### Impact

- **Phase 3 Query API**: spec out the parser (grammar) + evaluator (semi-naive or magic-set)
- **Phase 5 Inference Engine**: consumes the same IR; rules are persisted as text
- **Phase 3 Temporal Index**: must be accessible as an EDB predicate (indexed lookup path for `active_at`)
- **External bindings**: Python / HTTP clients send Datalog text; server returns rows

### Open questions

- Do we support variable-length paths (`X(*)*` in Cypher)? Datalog gets this via recursion — yes
- Aggregation syntax: use `count(?b)` in head, or a separate `agg` operator? — defer to Phase 3 spec
- Negation-as-failure vs. stratified — start with stratified-only; easier to validate termination

---

## D3 — Confidence propagation math: product rule (default) + per-rule override 🟡

### Context

Each relation has `confidence: f64 ∈ [0, 1]`. When derived relations are produced by rules, their confidence must be computed from the premises' confidences.

Example: rule `transitively_funds(?a, ?c) :- funds(?a, ?b), funds(?b, ?c)`. If `funds(alice, bob)` has conf 0.8 and `funds(bob, charlie)` has conf 0.7, what is the conf of `transitively_funds(alice, charlie)`?

### Options

| Option | Formula | Interpretation | When appropriate |
|--------|---------|----------------|------------------|
| A — Fuzzy min | `conf = min(c1, c2, ...)` | "chain is as strong as its weakest link" | Conservative; worst-case bound |
| B — Fuzzy product | `conf = c1 × c2 × ...` | "independent evidence; compound uncertainty" | Default for independent sources |
| C — Dempster-Shafer | Belief intervals `[bel, pl]` combined via DS rule | Proper uncertainty reasoning | Sensor fusion; probably overkill |
| D — Bayesian | `P(H\|E) = P(E\|H) × P(H) / P(E)` | Explicit priors + likelihoods | Requires priors; open-world issues |

### Decision

**Default: Option B — product rule.** Allow per-rule override via a `combine:` annotation in the rule:

```
transitively_funds(?a, ?c) [combine: min] :-
  funds(?a, ?b),
  funds(?b, ?c).
```

### Rationale

- Product rule is explainable (multiply the confidences), tractable (no fixpoint issues), and assumes independence — a reasonable default for unlinked sources
- Per-rule override lets domain experts express "this chain should be conservative" (use min) or "corroboration strengthens confidence" (custom combine)
- Dempster-Shafer adds complexity without clear win for OSINT's shallow rule chains (typically depth ≤ 3)
- Bayesian requires priors that we rarely have

### Impact

- **Phase 5 Inference**: rule evaluator tracks a `conf: f64` alongside each tuple; combines per rule's annotation (default product)
- **Phase 3 Query API**: aggregations of confidence use `sum` / `avg` / `max` operators over confidence field
- **Storage**: no change — confidence is already `f64` on Relation

### Open questions

- Confidence on **negated** premises? (Classical Datalog: negation is crisp. Fuzzy Datalog: `conf = 1 - confidence_of_positive`.) — pick 1.0 (negation-as-failure is crisp) for v1
- User-defined `combine` functions? — probably yes in Phase 5; start with a fixed vocabulary (`product`, `min`, `max`, `avg`)

---

## D4 — Structured provenance: `Vec<SourceRef>` on every fact 🟡

### Context

In OSINT every fact traces to a source. Current `properties: HashMap<String, PropertyValue>` could hold provenance but nothing enforces structure. Queries like "show me all facts sourced from document X" or "how confident are we that Alice works at Acme?" become ad-hoc.

### Options

| Option | Model | Pros | Cons |
|--------|-------|------|------|
| A — Ad-hoc via properties | `properties["source"] = "doc-123"` | No schema change; flexible | Unstructured; can't query efficiently; easy to forget |
| B — Structured `provenance: Vec<SourceRef>` on Entity + Relation | First-class field | Queryable; enforceable; explicit | Schema change; storage cost for rarely-multi-source entities |
| C — Reified statements (RDF-style quoting) | Every fact is its own entity with its own facts about it | Maximally flexible; same pattern as for RDF quads | Doubles entity count; deep query indirection |

### Decision

**Option B — structured provenance.** Add to both Entity and Relation:

```rust
pub struct SourceRef {
    /// EntityId of a Document-type entity (or external URI if source is not yet in the graph)
    pub source: SourceIdentifier,
    /// How this fact was extracted (manual / NLP / rule-derived / imported)
    pub method: ExtractionMethod,
    /// When the fact was extracted / asserted
    pub extracted_at: DateTime<Utc>,
    /// Extractor's reported confidence (may differ from overall fact confidence)
    pub confidence: f64,
}

pub enum SourceIdentifier {
    Entity(EntityId),       // internal Document entity
    External(String),       // URI, DOI, URL — not yet in graph
}

pub enum ExtractionMethod {
    Manual,                 // human analyst asserted
    NlpExtraction(String),  // model name / version
    RuleDerived(String),    // rule id that produced the fact
    Imported(String),       // source format / importer id
}
```

### Rationale

- Provenance becomes a first-class query predicate: `provenance(?fact, ?src)` in Datalog
- Multiple sources per fact (corroboration) is a fundamental OSINT pattern — `Vec<SourceRef>` is the right shape
- RDF-style quoting (C) is more elegant in theory but forces every fact to be 2 entities + 1 relation — 3× storage and 3× query indirection
- Keeps the Entity/Relation core model intact; provenance is additive

### Impact

- **Data model**: Entity + Relation gain `provenance: Vec<SourceRef>` (possibly empty for pre-provenance data)
- **Phase 2 WAL**: `GraphOp::CreateEntity` payload grows; existing WAL entries read as empty provenance
- **Phase 3 Query**: new EDB predicate `provenance(?fact_id, ?source_id, ?method, ?extracted_at, ?conf)`
- **Phase 5 Rule-derived facts**: method = `RuleDerived(rule_id)` automatically set; source = the IDs of the premise facts

### Open questions

- Index on `source`? (i.e. "all facts from doc-X") — yes, add in Phase 3
- How does SourceRef interact with D1 tx_time? (A new SourceRef appended at tx_time T is the tx-time signal.) — treat `extracted_at` as the tx_time witness for that assertion

---

## D5 — GPU is a storage tier, scheduled by buffer manager 🟡

### Context

Typical GPU-accelerated DBs treat the GPU as a compute add-on: user invokes `compute_pagerank()`, data gets shipped to VRAM, result comes back, done. This is fine for one-off batch jobs but suboptimal for continuous graph-analytic workloads where the hot subgraph should live on the device.

### Options

| Option | Model | Pros | Cons |
|--------|-------|------|------|
| A — GPU as compute add-on (cuGraph-style) | Explicit `to_gpu()` / `from_gpu()`; user controls staging | Simple; clear cost model | Repeated transfer cost; bad for iterative workloads |
| B — GPU as fourth tier | Buffer manager manages VRAM alongside RAM + mmap; hot pages migrate to VRAM | Amortizes transfer; fits iterative algorithms | Complex scheduler; VRAM contention; hard to predict |
| C — GPU dedicated subgraph | User declares a subgraph as "GPU-resident"; it stays on device until invalidated | Predictable; simple eviction model | Manual tuning; analyst must know workload shape |

### Decision

**Option C as v1, migrating toward B.**

Phase 4 v1:
- Analyst marks a subgraph as GPU-resident via a query annotation (`#[gpu_pin]`) or API call
- The buffer manager (Phase 2.5) loads matching pages to VRAM; retains until explicitly unpinned or RAM pressure forces drop
- Compute shaders (PageRank, BFS) operate on the pinned subgraph in CSR (Compressed Sparse Row) format

Future (Phase 4.5+):
- Add automatic tier promotion based on access frequency (mimic Option B)
- VRAM eviction uses LRU with a wire-up to CPU-side buffer pool

### Rationale

- Option A's per-query transfer is a poor fit for intelligence workloads that run many queries against a stable subgraph
- Option B's full auto-scheduling is right long-term but too much surface for Phase 4; needs a workload corpus to tune
- Option C is a middle ground: explicit user intent, same data path, can evolve to B without API break

### Impact

- **Phase 2.5 Buffer Pool**: must expose a "pin to tier" API that Phase 4 can target
- **Phase 4 GPU**: implements CSR staging + PageRank/BFS kernels against pinned subgraphs
- **Phase 3 Query API**: gains `#[gpu_pin]` annotation on persistent query views

### Open questions

- Multi-GPU? Out of scope; single-device assumed
- AMD / Intel GPU support via wgpu? wgpu is cross-vendor so yes in principle; tune shaders for NVIDIA first

---

## D6 — Apply-first lazy-log WAL ordering 🔒

### Context & Decision

Phase-2 ships apply-first lazy-log: `engine.rs` mutations update the in-memory DashMap and ack the caller immediately; `log_op()` `try_send`s to a background thread that serializes, buffers, and batched-fsyncs on an interval. Inverse of ARIES (fsync before ack). **Apply-first is the default**; `flush()` is the opt-in sync barrier for durability-critical callers.

### Rationale & Impact

- Throughput: mutation latency = DashMap insert, not disk fsync; crash window bounded to acked-but-unflushed tail (acceptable for OSINT analytics where re-ingest is cheap)
- Mirrors LMDB / RocksDB MemTable+WAL: in-memory hot path, durability as a separable concern
- **Phase 2.5+ (aspirational)**: durable-first / ARIES-style mode may be added behind a config flag for workloads that cannot tolerate the unflushed tail

---

## Cross-decision consistency check

- D1 (bitemporal) + D4 (provenance) → `extracted_at` in SourceRef aligns with `tx_from` on the created row; corrections produce a new SourceRef with later `extracted_at`
- D2 (Datalog) + D3 (confidence) → each derived tuple carries a confidence computed via D3's rule; Datalog engine threads this through naturally (tuple = (bindings, confidence))
- D2 (Datalog) + D4 (provenance) → derived tuples' provenance = set of premise tuples' provenances; chain is queryable
- D1 (bitemporal) + D5 (GPU) → GPU-pinned subgraph must specify `as_of_tx` / `as_of_valid`; subgraph is invalidated when a new fact in the time window is inserted
- D6 (apply-first WAL) + D1 (bitemporal) → `tx_from` is stamped at apply time, not at fsync time

## Revision log

| Date       | Decision | Change                                 |
|------------|----------|----------------------------------------|
| 2026-04-16 | D1–D5    | Initial draft (tentative — open to revision before Phase 3 impl begins) |
| 2026-04-16 | D6       | Document apply-first lazy-log WAL ordering shipped in Phase 2 (locked) |
