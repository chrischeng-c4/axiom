// CODEGEN-BEGIN
//! Competitive perf-regression GATE — native Rust, no Python/GIL.
//!
//! The standing commitment (README `ops-speed`): **lumen keeps its own latency
//! floors every release.** Postgres/OpenSearch comparisons are calibrated
//! evidence, not a per-run dependency; set `LUMEN_GATE_COMPARE_PEERS=1` only
//! when adding/changing a benchmark cell or explicitly refreshing peer data.
//! Each engine is driven over its own native client so the comparison is honest:
//!   - lumen      → reqwest against an in-test axum server over an in-process Engine
//!   - Postgres   → tokio-postgres (its own wire protocol)
//!   - OpenSearch → reqwest (`_bulk` + `_search`)
//!
//! The gate compares END-TO-END client latency for BOTH peers (concurrency
//! throughput is the next increment). lumen + OpenSearch share the HTTP/JSON
//! protocol class, so the transport tax cancels and e2e is the fair, float-precise
//! engine comparison — far better than OpenSearch's integer-ms `took`, which is too
//! coarse for the ~1ms text/filter cells and flaps the ratio between runs. pg's
//! binary protocol beats lumen's HTTP+JSON on cheap point/range predicates over
//! loopback, so those cells are EXEMPT (annotated) rather than gated. lumen's
//! engine-only `took_us` is still collected for reference (the `lum_eng` column).
//!
//! Gate types (per cell, per peer) live in `e2e/perf-baseline.json`:
//!   WIN    — must hold `max(1.0, ratchet * baseline)`; dropping below FAILS the build.
//!   TARGET — should win but does not yet; reported RED, does NOT fail (drives the work).
//!   EXEMPT — opponent home-turf; reported with a reason, never fails.
//!
//! MUST run with `--release`: debug numbers are too noisy for perf floors. The
//! readiness default is N=10k and Lumen-only so AW/release does not repeatedly
//! remeasure peer engines. Set `LUMEN_GATE_COMPARE_PEERS=1 LUMEN_GATE_N=100000`
//! for explicit release-local calibration, and `LUMEN_GATE_RELEASE_SOAK=1` or
//! `LUMEN_GATE_N=1000000` for the historical 1M soak evidence.
//!
//! Ignored by default because it is a release-mode perf gate:
//!   cargo test --release -p lumen --test perf_gate_vs_db -- --ignored --nocapture
//!   LUMEN_GATE_COMPARE_PEERS=1 cargo test --release -p lumen --test perf_gate_vs_db competitive_perf_gate -- --ignored --exact --nocapture
//!   LUMEN_GATE_RELEASE_SOAK=1 cargo test --release -p lumen --test perf_gate_vs_db -- --ignored --nocapture
//!   LUMEN_GATE_COMPARE_PEERS=1 LUMEN_PERF_STRICT=1 cargo test --release -p lumen --test perf_gate_vs_db competitive_perf_gate -- --ignored --exact --nocapture
//!
//! ## Contracts inherited from the retired EC shells
//!
//! These 4 sentences were the whole of the `// Contract:` comment in 4 AW-EC shells
//! under `apps/lumen/e2e/`, each of which ran `cargo test -p lumen --test
//! perf_gate_vs_db` in a subprocess and asserted the child's exit status. `cargo test
//! -p lumen` already runs this target directly, so the shells added a second, nested
//! run and nothing else. They were deleted on 2026-08-20 with the EC machinery they
//! belonged to, and the sentence is the only thing they held that nothing else did.
//! Each line below is prefixed with the EC id its shell was filed under.
//!
//! - `lumen-claim-exact-term-range-set` — Term, range, and set filter behavior stays
//!   within the exact/filter search gate.
//! - `lumen-claim-exact-wide-range-filter` — Wide range filters over sorted disk-backed
//!   values pass the exact/filter gate.
//! - `lumen-claim-lexical-bm25` — BM25 ranking and analyzer behavior pass the ratcheted
//!   performance/conformance comparison.
//! - `lumen-claim-search-core-filter-sort` — Filter/sort early-termination behavior is
//!   covered by the ratcheted database comparison gate.

use std::sync::Arc;
use std::time::{Duration, Instant};

use serde_json::{json, Value};
use tokio::sync::{Mutex, Semaphore};
use tokio::task::JoinSet;

const SEED: u64 = 1234;
const WARMUP: usize = 5;
const REPS: usize = 50;
const SORTED_PAGE_DEEP_SIZE: usize = 1_000;
const DEFAULT_GATE_N: usize = 10_000;
const RELEASE_SOAK_GATE_N: usize = 1_000_000;
const DEFAULT_SCALE_ROWS: &[usize] = &[1_000, 10_000, 100_000];
const DEFAULT_SCALE_MAX_ROWS: usize = 100_000;
const SCALE_READ_CELLS: &[&str] = &["range", "filter_sort", "keyword_sort", "sorted_page_deep"];
const SCALE_CURSOR_PAGE_SIZE: u32 = 100;
const SCALE_RECLAIMER_DRAIN_TIMEOUT: Duration = Duration::from_secs(30);
// EC env override: vat exports LUMEN_BENCH_PG_DSN / LUMEN_BENCH_OS_URL when it
// provisions pg + OpenSearch; fall back to the local-dev defaults otherwise.
fn pg_dsn() -> String {
    std::env::var("LUMEN_BENCH_PG_DSN")
        .unwrap_or_else(|_| "host=/tmp dbname=lumenbench".to_string())
}
fn os_url() -> String {
    std::env::var("LUMEN_BENCH_OS_URL").unwrap_or_else(|_| "http://localhost:9200".to_string())
}
fn require_db_peers() -> bool {
    env_flag_enabled("LUMEN_REQUIRE_DB_PEERS")
}

fn compare_peers_enabled() -> bool {
    env_flag_enabled("LUMEN_GATE_COMPARE_PEERS") || env_flag_enabled("LUMEN_GATE_CALIBRATE_PEERS")
}

const CELLS: &[&str] = &[
    "text_bm25",
    "text_and",
    "filtered_search",
    "kw_term",
    "range",
    "bool_filter",
    "filter_sort",
    "sorted_page_deep",
    "pure_sort",
    // Presence + collision filters (DataTable composite search). `exists` is a
    // sparse-field presence scan; `duplicated` finds docs whose value collides.
    // Start un-baselined → judge() treats them as EXEMPT (numbers reported, never
    // fails) until a stable ratio is promoted to a `win` floor in perf-baseline.json.
    "exists",
    "duplicated",
    // Vector cells — only run with LUMEN_GATE_VECTOR=1 (parse_gate_cells filters
    // them out of the default gate; OS has no k-NN plugin so only pgvector is the peer).
    "knn",
    "filtered_knn",
];

const PG_CHEAP_CELLS: &[&str] = &["kw_term", "range", "bool_filter"];

const CITIES: [&str; 50] = [
    "taipei",
    "tokyo",
    "osaka",
    "seoul",
    "shanghai",
    "beijing",
    "shenzhen",
    "singapore",
    "bangkok",
    "jakarta",
    "manila",
    "hanoi",
    "kualalumpur",
    "delhi",
    "mumbai",
    "dubai",
    "london",
    "paris",
    "berlin",
    "madrid",
    "rome",
    "amsterdam",
    "zurich",
    "vienna",
    "stockholm",
    "oslo",
    "helsinki",
    "dublin",
    "lisbon",
    "prague",
    "warsaw",
    "athens",
    "istanbul",
    "moscow",
    "newyork",
    "boston",
    "chicago",
    "seattle",
    "austin",
    "denver",
    "sanfrancisco",
    "losangeles",
    "toronto",
    "vancouver",
    "mexico",
    "saopaulo",
    "buenosaires",
    "sydney",
    "melbourne",
    "auckland",
];

const VOCAB: &[&str] = &[
    "system",
    "data",
    "query",
    "index",
    "search",
    "scale",
    "latency",
    "throughput",
    "memory",
    "vector",
    "cluster",
    "shard",
    "replica",
    "stream",
    "record",
    "schema",
    "field",
    "value",
    "token",
    "service",
    "network",
    "protocol",
    "request",
    "response",
    "cache",
    "buffer",
    "thread",
    "design",
    "build",
    "deploy",
    "test",
    "verify",
    "monitor",
    "profile",
    "optimize",
    "refactor",
    "model",
    "train",
    "infer",
    "embed",
    "rank",
    "score",
    "filter",
    "match",
    "boolean",
    "nested",
    "alpha",
    "beta",
    "gamma",
    "delta",
    "omega",
    "sigma",
    "theta",
    "kappa",
    "lambda",
    "phi",
    "rho",
    "north",
    "south",
    "east",
    "west",
    "prime",
    "core",
    "edge",
    "node",
    "leaf",
    "root",
    "path",
    "graph",
    "tree",
    "fast",
    "slow",
    "warm",
    "cold",
    "hot",
    "dense",
    "sparse",
    "exact",
    "dynamic",
];

const LCG_A: u64 = 6_364_136_223_846_793_005;
const LCG_C: u64 = 1_442_695_040_888_963_407;

struct Lcg(u64);
impl Lcg {
    fn new(s: u64) -> Self {
        Self(s.wrapping_mul(LCG_A) ^ 0x9E37_79B9_7F4A_7C15)
    }
    fn for_doc(doc: usize) -> Self {
        let mut rng = Self::new(SEED);
        let steps = u64::try_from(doc)
            .expect("doc offset fits u64")
            .checked_mul(10)
            .expect("doc offset rng steps fit u64");
        rng.advance(steps);
        rng
    }
    fn advance(&mut self, mut steps: u64) {
        let mut cur_mult = LCG_A;
        let mut cur_plus = LCG_C;
        let mut acc_mult = 1u64;
        let mut acc_plus = 0u64;
        while steps > 0 {
            if steps & 1 != 0 {
                acc_mult = acc_mult.wrapping_mul(cur_mult);
                acc_plus = acc_plus.wrapping_mul(cur_mult).wrapping_add(cur_plus);
            }
            cur_plus = cur_mult.wrapping_add(1).wrapping_mul(cur_plus);
            cur_mult = cur_mult.wrapping_mul(cur_mult);
            steps >>= 1;
        }
        self.0 = acc_mult.wrapping_mul(self.0).wrapping_add(acc_plus);
    }
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(LCG_A).wrapping_add(LCG_C);
        self.0
    }
    fn pick<'a>(&mut self, a: &[&'a str]) -> &'a str {
        a[(self.next() >> 33) as usize % a.len()]
    }
}

struct Doc {
    eid: String,
    bio: String,
    city: &'static str,
    age: i32,
    /// Sparse field for the `exists` cell: present on 25% of docs (i%4==0), NULL
    /// otherwise. Lets the presence filter measure a *selective* `exists`, not a
    /// degenerate "every doc has it" scan.
    note: Option<&'static str>,
    /// 128-dim unit embedding for the `knn` / `filtered_knn` cells. Only populated
    /// when LUMEN_GATE_VECTOR=1 — the HNSW build (lumen + pgvector) is expensive,
    /// so the default search gate stays vector-free and fast.
    embedding: Option<Vec<f32>>,
}

/// Deterministic corpus with the same probe selectivities as scripts/bench_vs_db.py:
/// 'engineer' in i%40 (~2.5%), 'rust' in i%80 (~1.25%), city[i%50] ('taipei' ~2%),
/// age in [30,40) ~16%. Filler words are VOCAB (never contain the probe tokens), so
/// selectivity is exact regardless of the filler RNG. No embeddings (search gate).
fn gen_doc(i: usize, rng: &mut Lcg) -> Doc {
    let mut words: Vec<&str> = (0..10).map(|_| rng.pick(VOCAB)).collect();
    if i % 40 == 0 {
        words.push("engineer");
    }
    if i % 80 == 0 {
        words.push("rust");
    }
    Doc {
        eid: format!("d{i}"),
        bio: words.join(" "),
        city: CITIES[i % 50],
        age: 18 + ((i as i64 * 7919) % 63) as i32,
        // present on 25% of docs → `exists`/`not exists` is selective; `city`
        // (50-way, every value repeats) drives the `duplicated` cell instead.
        note: if i % 4 == 0 { Some("present") } else { None },
        embedding: None, // populated by gen_corpus only when LUMEN_GATE_VECTOR=1
    }
}

// ---- vector (knn / filtered_knn) cells — opt-in via LUMEN_GATE_VECTOR=1 ----
const VEC_DIM: usize = 128;
const KNN_K: usize = 10;
const VEC_SEED: u64 = 0x5EED_3EC7;

fn vector_enabled() -> bool {
    std::env::var("LUMEN_GATE_VECTOR")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

/// Deterministic unit (L2-normalized) vector — cosine cares about direction only.
fn unit_vec(seed: u64) -> Vec<f32> {
    let mut rng = Lcg::new(seed);
    let mut v: Vec<f32> = (0..VEC_DIM)
        .map(|_| ((rng.next() >> 33) as f64 / (1u64 << 31) as f64 * 2.0 - 1.0) as f32)
        .collect();
    let norm = v
        .iter()
        .map(|x| (*x as f64) * (*x as f64))
        .sum::<f64>()
        .sqrt() as f32;
    if norm > 0.0 {
        for x in &mut v {
            *x /= norm;
        }
    }
    v
}

fn gen_embedding(i: usize) -> Vec<f32> {
    unit_vec(VEC_SEED ^ (i as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15))
}

/// Fixed query vector for the knn cells (deterministic, unit-norm).
fn query_vec() -> Vec<f32> {
    unit_vec(VEC_SEED ^ 0xDEAD_BEEF)
}

fn gen_corpus(n: usize) -> Vec<Doc> {
    let mut rng = Lcg::new(SEED);
    let mut docs: Vec<Doc> = (0..n).map(|i| gen_doc(i, &mut rng)).collect();
    if vector_enabled() {
        for (i, d) in docs.iter_mut().enumerate() {
            d.embedding = Some(gen_embedding(i));
        }
    }
    docs
}

#[test]
fn lcg_skip_ahead_matches_sequential_doc_stream() {
    let mut seq = Lcg::new(SEED);
    for i in 0..10_000 {
        let want = gen_doc(i, &mut seq);
        if matches!(i, 0 | 1 | 17 | 999 | 10_000) || i % 997 == 0 {
            let mut jumped = Lcg::for_doc(i);
            let got = gen_doc(i, &mut jumped);
            assert_eq!(got.eid, want.eid);
            assert_eq!(got.bio, want.bio);
            assert_eq!(got.city, want.city);
            assert_eq!(got.age, want.age);
            assert_eq!(got.note, want.note);
        }
    }
}

// ---------------------------------------------------------------------------
// per-engine query bodies (mirror bench_vs_db.py exactly)
// ---------------------------------------------------------------------------
fn lumen_query(cell: &str) -> Value {
    match cell {
        "text_bm25" => json!({"query":{"match":{"field":"bio","text":"engineer"}},"limit":10}),
        "text_and" => {
            json!({"query":{"match":{"field":"bio","text":"engineer rust","op":"and"}},"limit":10})
        }
        "kw_term" => json!({"query":{"term":{"field":"city","value":"taipei"}},"limit":10}),
        "range" => json!({"query":{"range":{"field":"age","gte":30,"lt":40}},"limit":10}),
        "bool_filter" => {
            json!({"query":{"and":[{"term":{"field":"city","value":"taipei"}},{"range":{"field":"age","gte":30,"lt":40}}]},"limit":10})
        }
        "filtered_search" => {
            json!({"query":{"and":[{"match":{"field":"bio","text":"engineer"}},{"term":{"field":"city","value":"taipei"}},{"range":{"field":"age","gte":30,"lt":40}}]},"limit":10})
        }
        "filter_sort" => {
            json!({"query":{"term":{"field":"city","value":"taipei"}},"sort":[{"field":"age","order":"asc"}],"limit":10,"track_total":false})
        }
        "sorted_page_deep" => {
            json!({"query":{"range":{"field":"age","gte":0}},"sort":[{"field":"age","order":"asc"}],"limit":SORTED_PAGE_DEEP_SIZE,"track_total":false})
        }
        "pure_sort" => {
            json!({"query":{"range":{"field":"age"}},"sort":[{"field":"age","order":"asc"}],"limit":10,"track_total":false})
        }
        "exists" => json!({"query":{"exists":{"field":"note"}},"limit":10}),
        "duplicated" => {
            json!({"query":{"duplicated":{"field":"city","min_group_size":2}},"limit":10})
        }
        "knn" => {
            json!({"query":{"knn":{"field":"embedding","vector":query_vec(),"k":KNN_K}},"limit":KNN_K})
        }
        // filter-correct kNN: nearest within the city=taipei subset (the pgvector
        // post-filter recall-collapse case). lumen evaluates the filter then the
        // kNN over the survivors — no recall loss.
        "filtered_knn" => json!({"query":{"and":[
            {"knn":{"field":"embedding","vector":query_vec(),"k":KNN_K}},
            {"term":{"field":"city","value":"taipei"}}
        ]},"limit":KNN_K}),
        _ => unreachable!("unknown cell {cell}"),
    }
}

/// The fixed read matrix for the Lumen-only scale bench. These query shapes stay
/// independent from the competitive peer cells because the scale corpus has a
/// deliberately high-cardinality keyword column and the cursor row must start
/// from one precomputed mid-collection cursor.
fn scale_lumen_query(cell: &str, cursor: Option<String>) -> Value {
    let cursor = cursor.map(Value::String).unwrap_or(Value::Null);
    match cell {
        // `range` is deliberately a compound range + boolean filter cell. The
        // fixed four selector names stay stable while this one request covers
        // both required filter behaviours.
        "range" => json!({
            "query":{"and":[
                {"term":{"field":"city","value":"taipei"}},
                {"range":{"field":"age","gte":30,"lt":40}}
            ]},
            "limit":10
        }),
        // `filter_sort` is the required numeric-sort cell.
        "filter_sort" => json!({
            "query":{"range":{"field":"age","gte":0}},
            "sort":[{"field":"age","order":"asc"}],
            "limit":10,
            "track_total":false
        }),
        // `keyword_sort` is a deterministic high-cardinality keyword sort.
        // #3997 makes it sparse; `missing:last` plus an exact total reproduce
        // the formerly unbounded planner shape.
        "keyword_sort" => json!({
            "query":{"range":{"field":"age","gte":0}},
            "sort":[{"field":"sort_key","order":"asc","missing":"last"}],
            "limit":80,
            "track_total":true
        }),
        // `sorted_page_deep` is cursor pagination from a precomputed deep cursor.
        "sorted_page_deep" => json!({
            "query":{"range":{"field":"age","gte":0}},
            "sort":[{"field":"doc_key","order":"asc"}],
            "limit":SCALE_CURSOR_PAGE_SIZE,
            "cursor":cursor,
            "track_total":false
        }),
        _ => unreachable!("unknown scale cell {cell}"),
    }
}

/// pgvector literal: `[0.12,-0.04,...]`.
fn pg_vec_literal(v: &[f32]) -> String {
    let mut s = String::with_capacity(v.len() * 8 + 2);
    s.push('[');
    for (i, x) in v.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push_str(&format!("{x:.6}"));
    }
    s.push(']');
    s
}

fn pg_sql(cell: &str, table: &str) -> String {
    match cell {
        "text_bm25" => format!(
            "SELECT eid FROM {table} WHERE bio_tsv @@ websearch_to_tsquery('simple','engineer') ORDER BY ts_rank(bio_tsv, websearch_to_tsquery('simple','engineer')) DESC LIMIT 10"
        ),
        "text_and" => format!(
            "SELECT eid FROM {table} WHERE bio_tsv @@ websearch_to_tsquery('simple','engineer rust') ORDER BY ts_rank(bio_tsv, websearch_to_tsquery('simple','engineer rust')) DESC LIMIT 10"
        ),
        "filtered_search" => format!(
            "SELECT eid FROM {table} WHERE bio_tsv @@ websearch_to_tsquery('simple','engineer') AND city='taipei' AND age>=30 AND age<40 ORDER BY ts_rank(bio_tsv, websearch_to_tsquery('simple','engineer')) DESC LIMIT 10"
        ),
        "kw_term" => format!("SELECT eid FROM {table} WHERE city='taipei' LIMIT 10"),
        "range" => format!("SELECT eid FROM {table} WHERE age>=30 AND age<40 LIMIT 10"),
        "bool_filter" => {
            format!("SELECT eid FROM {table} WHERE city='taipei' AND age>=30 AND age<40 LIMIT 10")
        }
        "filter_sort" => format!(
            "SELECT eid FROM {table} WHERE city='taipei' ORDER BY age ASC, eid ASC LIMIT 10"
        ),
        "sorted_page_deep" => format!(
            "SELECT eid FROM {table} WHERE age>=0 ORDER BY age ASC, eid ASC LIMIT {} OFFSET {}",
            SORTED_PAGE_DEEP_SIZE,
            deep_page_offset(gate_n())
        ),
        "pure_sort" => format!("SELECT eid FROM {table} ORDER BY age ASC, eid ASC LIMIT 10"),
        "exists" => format!("SELECT eid FROM {table} WHERE note IS NOT NULL LIMIT 10"),
        // pg's idiomatic "find duplicates" — a GROUP BY/HAVING subquery that
        // collision-filters the rows. The B-tree on city serves the grouping.
        "duplicated" => format!(
            "SELECT eid FROM {table} WHERE city IN \
             (SELECT city FROM {table} GROUP BY city HAVING count(*) >= 2) LIMIT 10"
        ),
        "knn" => format!(
            "SELECT eid FROM {table} ORDER BY embedding <=> '{}' LIMIT {KNN_K}",
            pg_vec_literal(&query_vec())
        ),
        // pgvector + WHERE: pg's HNSW index is a post-filter (ANN top-k then drop
        // non-matching rows) → recall collapses when the filter is selective; the
        // planner may instead pre-filter and exact-sort. Either way this is the
        // cell where lumen's filter-correct kNN wins on correctness + latency.
        "filtered_knn" => format!(
            "SELECT eid FROM {table} WHERE city='taipei' ORDER BY embedding <=> '{}' LIMIT {KNN_K}",
            pg_vec_literal(&query_vec())
        ),
        _ => unreachable!("unknown cell {cell}"),
    }
}

fn os_query(cell: &str) -> Value {
    match cell {
        "text_bm25" => json!({"query":{"match":{"bio":"engineer"}},"size":10}),
        "text_and" => {
            json!({"query":{"match":{"bio":{"query":"engineer rust","operator":"and"}}},"size":10})
        }
        "kw_term" => json!({"query":{"term":{"city":"taipei"}},"size":10}),
        "range" => json!({"query":{"range":{"age":{"gte":30,"lt":40}}},"size":10}),
        "bool_filter" => {
            json!({"query":{"bool":{"filter":[{"term":{"city":"taipei"}},{"range":{"age":{"gte":30,"lt":40}}}]}},"size":10})
        }
        "filtered_search" => {
            json!({"query":{"bool":{"must":[{"match":{"bio":"engineer"}}],"filter":[{"term":{"city":"taipei"}},{"range":{"age":{"gte":30,"lt":40}}}]}},"size":10})
        }
        "filter_sort" => {
            json!({"query":{"bool":{"filter":[{"term":{"city":"taipei"}}]}},"sort":[{"age":"asc"}],"track_total_hits":false,"size":10})
        }
        "sorted_page_deep" => {
            json!({"query":{"range":{"age":{"gte":0}}},"sort":[{"age":"asc"}],"track_total_hits":false,"size":SORTED_PAGE_DEEP_SIZE,"from":deep_page_offset(gate_n())})
        }
        "pure_sort" => {
            json!({"query":{"match_all":{}},"sort":[{"age":"asc"}],"track_total_hits":false,"size":10})
        }
        "exists" => json!({"query":{"exists":{"field":"note"}},"size":10}),
        // OpenSearch has NO single query that returns the *duplicate docs* as a
        // composable filter — the idiomatic answer is a terms aggregation with
        // min_doc_count, which returns the colliding *values* (buckets), not docs,
        // and cannot be AND/OR/NOT-composed with other queries. This cell times
        // that aggregation as OS's closest equivalent; the semantic gap (values vs
        // composable doc-set) is the point, not just the latency.
        "duplicated" => json!({
            "size": 0,
            "aggs": {"dups": {"terms": {"field": "city", "min_doc_count": 2, "size": 100}}}
        }),
        // OpenSearch on this host has no k-NN plugin; the gate loop never measures
        // OS for vector cells, but the match arm must be total.
        "knn" | "filtered_knn" => json!({"query":{"match_all":{}},"size":KNN_K}),
        _ => unreachable!("unknown cell {cell}"),
    }
}

// ---------------------------------------------------------------------------
// stats
// ---------------------------------------------------------------------------
#[derive(Clone, Default)]
#[allow(dead_code)] // e2e_p50/p99 reported in the verbose path + future concurrency table
struct Stat {
    e2e_min: f64,
    e2e_p50: f64,
    e2e_p99: f64,
    engine_min: Option<f64>, // ms (lumen took_us/1000, OpenSearch took)
}

fn summarize(mut e2e: Vec<f64>, mut engine: Vec<f64>) -> Stat {
    e2e.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let p = |v: &[f64], q: f64| v[(((v.len() - 1) as f64) * q).round() as usize];
    let engine_min = if engine.is_empty() {
        None
    } else {
        engine.sort_by(|a, b| a.partial_cmp(b).unwrap());
        Some(engine[0])
    };
    Stat {
        e2e_min: e2e[0],
        e2e_p50: p(&e2e, 0.50),
        e2e_p99: p(&e2e, 0.99),
        engine_min,
    }
}

// ---------------------------------------------------------------------------
// lumen: in-process Engine behind a real axum server, driven via reqwest
// ---------------------------------------------------------------------------
async fn lumen_serve(docs: &[Doc]) -> (reqwest::Client, String) {
    let (client, base, _engine) = lumen_serve_engine(docs).await;
    (client, base)
}

/// Same as [`lumen_serve`] but ALSO returns the in-process `Arc<Engine>` handle
/// the axum server is driving. The disk gate needs the handle to call
/// `flush_to_segments` IN PLACE after indexing — turning this same live engine
/// (and therefore the same `/search` HTTP path + `took_us`) segment-backed —
/// and to probe that the in-RAM drivers were actually dropped. The server task
/// keeps an `Arc` clone, so the engine outlives this function; the returned
/// clone lets the test mutate the shared `RwLock<EngineState>` while the server
/// serves from it.
async fn lumen_serve_engine(
    docs: &[Doc],
) -> (reqwest::Client, String, Arc<lumen::storage::Engine>) {
    let engine = Arc::new(lumen::storage::Engine::new());
    let app = lumen::api::router(lumen::api::AppState::open(engine.clone()));
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    listener.set_nonblocking(true).unwrap();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .thread_name("lumen-perf-http")
            .build()
            .unwrap();
        rt.block_on(async move {
            let listener = tokio::net::TcpListener::from_std(listener).unwrap();
            let _ = axum::serve(listener, app).await;
        });
    });
    let base = format!("http://{addr}");
    let client = reqwest::Client::new();

    // wait for readiness
    for _ in 0..50 {
        if client.get(format!("{base}/healthz")).send().await.is_ok() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }

    let mut fields = json!({
        "bio":{"type":"text"},
        "city":{"type":"keyword"},
        "age":{"type":"number"},
        "note":{"type":"keyword"}
    });
    if vector_enabled() {
        fields["embedding"] = json!({"type":"vector","dim":VEC_DIM,"metric":"cosine"});
    }
    client
        .put(format!("{base}/collections/docs"))
        .json(&json!({ "fields": fields }))
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap();

    // index in batches (<= bulk item cap)
    let mut items: Vec<Value> = Vec::with_capacity(9000);
    for d in docs {
        items.push(json!({"external_id":d.eid,"field":"bio","value":d.bio}));
        items.push(json!({"external_id":d.eid,"field":"city","value":d.city}));
        items.push(json!({"external_id":d.eid,"field":"age","value":d.age}));
        // Only index `note` where present → sparse field, lumen `exists` selective.
        if let Some(note) = d.note {
            items.push(json!({"external_id":d.eid,"field":"note","value":note}));
        }
        if let Some(emb) = &d.embedding {
            items.push(json!({"external_id":d.eid,"field":"embedding","value":emb}));
        }
        if items.len() >= 9000 {
            post_index(&client, &base, &items).await;
            items.clear();
        }
    }
    if !items.is_empty() {
        post_index(&client, &base, &items).await;
    }
    (client, base, engine)
}

struct NativeEndpoint {
    addr: String,
    #[allow(dead_code)]
    dir: Option<tempfile::TempDir>,
}

async fn lumen_serve_native(engine: Arc<lumen::storage::Engine>) -> NativeEndpoint {
    #[cfg(unix)]
    {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("lumen-native.sock");
        let listener = std::os::unix::net::UnixListener::bind(&path).unwrap();
        listener.set_nonblocking(true).unwrap();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .thread_name("lumen-perf-native")
                .build()
                .unwrap();
            rt.block_on(async move {
                let listener = tokio::net::UnixListener::from_std(listener).unwrap();
                let _ = lumen::native_wire::serve_unix_search(listener, engine).await;
            });
        });
        return NativeEndpoint {
            addr: path.to_string_lossy().into_owned(),
            dir: Some(dir),
        };
    }

    #[cfg(not(unix))]
    {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        listener.set_nonblocking(true).unwrap();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .thread_name("lumen-perf-native")
                .build()
                .unwrap();
            rt.block_on(async move {
                let listener = tokio::net::TcpListener::from_std(listener).unwrap();
                let _ = lumen::native_wire::serve_search(listener, engine).await;
            });
        });
        NativeEndpoint {
            addr: addr.to_string(),
            dir: None,
        }
    }
}

async fn post_index(client: &reqwest::Client, base: &str, items: &[Value]) {
    let response = client
        .post(format!("{base}/collections/docs/index"))
        .json(&json!({"items": items}))
        .send()
        .await
        .expect("scale index request");
    let status = response.status();
    if !status.is_success() {
        let body = response
            .text()
            .await
            .unwrap_or_else(|error| format!("<failed to read response body: {error}>"));
        panic!("scale index failed with HTTP {status}: {body}");
    }
}

async fn measure_lumen(client: &reqwest::Client, base: &str, cell: &str) -> Stat {
    measure_lumen_request(client, base, lumen_query(cell)).await
}

async fn measure_lumen_request(client: &reqwest::Client, base: &str, body: Value) -> Stat {
    let url = format!("{base}/collections/docs/search");
    let mut e2e = Vec::with_capacity(REPS);
    let mut engine = Vec::with_capacity(REPS);
    for r in 0..(WARMUP + REPS) {
        let t = Instant::now();
        let j: Value = client
            .post(&url)
            .json(&body)
            .send()
            .await
            .unwrap()
            .error_for_status()
            .unwrap()
            .json()
            .await
            .unwrap();
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        if r >= WARMUP {
            e2e.push(ms);
            if let Some(us) = j.get("took_us").and_then(|v| v.as_f64()) {
                engine.push(us / 1000.0);
            }
        }
    }
    summarize(e2e, engine)
}

async fn measure_lumen_sorted_page_deep(client: &reqwest::Client, base: &str, n: usize) -> Stat {
    let url = format!("{base}/collections/docs/search");
    let target_page = (deep_page_offset(n) / SORTED_PAGE_DEEP_SIZE).max(1);
    let measure_window = REPS.min(target_page + 1);
    let measure_from = target_page + 1 - measure_window;
    let mut cursor: Option<String> = None;
    let mut e2e = Vec::with_capacity(measure_window);
    let mut engine = Vec::with_capacity(measure_window);

    for page_idx in 0..=target_page {
        let mut body = lumen_query("sorted_page_deep");
        if let Some(c) = cursor.take() {
            body["cursor"] = Value::String(c);
        }
        let t = Instant::now();
        let j: Value = client
            .post(&url)
            .json(&body)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        let hits = j
            .get("hits")
            .and_then(|v| v.as_array())
            .expect("lumen sorted_page_deep hits array");
        assert!(
            !hits.is_empty(),
            "sorted_page_deep exhausted before target page {target_page}"
        );
        if page_idx >= measure_from {
            e2e.push(ms);
            if let Some(us) = j.get("took_us").and_then(|v| v.as_f64()) {
                engine.push(us / 1000.0);
            }
        }
        cursor = j
            .get("cursor")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
    }

    summarize(e2e, engine)
}

async fn measure_lumen_native(addr: &str, cell: &str) -> Stat {
    let frame = lumen_native_frame(cell);
    let mut e2e = Vec::with_capacity(REPS);
    let mut engine = Vec::with_capacity(REPS);
    #[cfg(unix)]
    let mut stream = tokio::net::UnixStream::connect(addr).await.unwrap();
    #[cfg(not(unix))]
    let mut stream = tokio::net::TcpStream::connect(addr).await.unwrap();
    for r in 0..(WARMUP + REPS) {
        let t = Instant::now();
        let resp = lumen::native_wire::search_prepared(&mut stream, &frame)
            .await
            .unwrap();
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        if r >= WARMUP {
            e2e.push(ms);
            engine.push(resp.took_us as f64 / 1000.0);
        }
    }
    summarize(e2e, engine)
}

fn lumen_native_frame(cell: &str) -> Vec<u8> {
    match cell {
        "kw_term" => lumen::native_wire::encode_term_frame("docs", "city", "taipei", 10).unwrap(),
        "range" => {
            lumen::native_wire::encode_range_frame("docs", "age", Some(30.0), Some(40.0), 10)
                .unwrap()
        }
        "bool_filter" => lumen::native_wire::encode_term_range_frame(
            "docs",
            "city",
            "taipei",
            "age",
            Some(30.0),
            Some(40.0),
            10,
        )
        .unwrap(),
        _ => {
            let req: lumen::types::SearchRequest =
                serde_json::from_value(lumen_query(cell)).unwrap();
            lumen::native_wire::encode_search_frame("docs", &req).unwrap()
        }
    }
}

// ---------------------------------------------------------------------------
// postgres: tokio-postgres
// ---------------------------------------------------------------------------
async fn pg_setup(docs: &[Doc], table: &str) -> Option<tokio_postgres::Client> {
    let (client, connection) = match tokio_postgres::connect(&pg_dsn(), tokio_postgres::NoTls).await
    {
        Ok(c) => c,
        Err(e) => {
            if require_db_peers() {
                panic!("postgres required by LUMEN_REQUIRE_DB_PEERS=1 but unavailable: {e}");
            }
            eprintln!("  ! postgres unavailable ({e}); skipping pg");
            return None;
        }
    };
    tokio::spawn(async move {
        let _ = connection.await;
    });
    let vec_on = vector_enabled();
    client
        .batch_execute(&format!("DROP TABLE IF EXISTS {table}"))
        .await
        .unwrap();
    if vec_on {
        client
            .batch_execute("CREATE EXTENSION IF NOT EXISTS vector")
            .await
            .unwrap();
    }
    let emb_col = if vec_on {
        format!(", embedding vector({VEC_DIM})")
    } else {
        String::new()
    };
    client
        .batch_execute(&format!(
            "CREATE TABLE {table} (
                eid text PRIMARY KEY,
                bio text,
                bio_tsv tsvector GENERATED ALWAYS AS (to_tsvector('simple', bio)) STORED,
                city text,
                age int,
                note text{emb_col}
            )"
        ))
        .await
        .unwrap();
    // batched multi-row INSERT (corpus is clean [a-z ]; safe to single-quote inline)
    let cols = if vec_on {
        "(eid,bio,city,age,note,embedding)"
    } else {
        "(eid,bio,city,age,note)"
    };
    for batch in docs.chunks(2000) {
        let mut sql = format!("INSERT INTO {table} {cols} VALUES ");
        for (k, d) in batch.iter().enumerate() {
            if k > 0 {
                sql.push(',');
            }
            // note is sparse → emit a bare NULL (not the string 'NULL') when absent.
            let note = d
                .note
                .map(|s| format!("'{s}'"))
                .unwrap_or_else(|| "NULL".to_string());
            if vec_on {
                let emb = d
                    .embedding
                    .as_ref()
                    .map(|e| format!("'{}'", pg_vec_literal(e)))
                    .unwrap_or_else(|| "NULL".to_string());
                sql.push_str(&format!(
                    "('{}','{}','{}',{},{},{})",
                    d.eid, d.bio, d.city, d.age, note, emb
                ));
            } else {
                sql.push_str(&format!(
                    "('{}','{}','{}',{},{})",
                    d.eid, d.bio, d.city, d.age, note
                ));
            }
        }
        client.batch_execute(&sql).await.unwrap();
    }
    // pgvector HNSW index (mirrors bench_vs_db.py: m=16, ef_construction=64,
    // ef_search=100). SET is session-scoped and persists onto the returned client,
    // so measure_pg's prepared kNN queries run at ef_search=100.
    let vec_idx = if vec_on {
        format!(
            "CREATE INDEX {table}_vec ON {table} USING hnsw (embedding vector_cosine_ops) \
             WITH (m=16, ef_construction=64);\n             SET hnsw.ef_search = 100;\n             "
        )
    } else {
        String::new()
    };
    client
        .batch_execute(&format!(
            "CREATE INDEX {table}_bio_gin ON {table} USING gin (bio_tsv);
             CREATE INDEX {table}_city ON {table} (city);
             CREATE INDEX {table}_age ON {table} (age);
             CREATE INDEX {table}_note ON {table} (note);
             {vec_idx}ANALYZE {table}"
        ))
        .await
        .unwrap();
    Some(client)
}

async fn measure_pg(client: &tokio_postgres::Client, cell: &str, table: &str) -> Stat {
    // PREPARED statement (Bind/Execute reusing the cached plan) instead of
    // client.query(&str,..) which re-Parses+Plans every call (tokio-postgres
    // prepares an anonymous statement per call). Without this, the per-call
    // parse/plan tax dominates pg's sub-ms cheap-predicate cells and OVERSTATES
    // lumen's win — the honest prepared latency CONFIRMS those cells' exemption.
    let sql = pg_sql(cell, table);
    let stmt = client.prepare(&sql).await.unwrap();
    let mut e2e = Vec::with_capacity(REPS);
    for r in 0..(WARMUP + REPS) {
        let t = Instant::now();
        let _ = client.query(&stmt, &[]).await.unwrap();
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        if r >= WARMUP {
            e2e.push(ms);
        }
    }
    summarize(e2e, Vec::new())
}

async fn measure_pg_sorted_page_deep(client: &tokio_postgres::Client, table: &str) -> Stat {
    let sql = pg_sql("sorted_page_deep", table);
    let stmt = client.prepare(&sql).await.unwrap();
    let mut e2e = Vec::with_capacity(REPS);
    for r in 0..(WARMUP + REPS) {
        let t = Instant::now();
        let _ = client.query(&stmt, &[]).await.unwrap();
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        if r >= WARMUP {
            e2e.push(ms);
        }
    }
    summarize(e2e, Vec::new())
}

async fn pg_disk_bytes(client: &tokio_postgres::Client, table: &str) -> Option<u64> {
    let sql = format!("SELECT pg_total_relation_size('{table}'::regclass)");
    let row = client.query_one(&sql, &[]).await.ok()?;
    let bytes: i64 = row.get(0);
    u64::try_from(bytes).ok()
}

// ---------------------------------------------------------------------------
// opensearch: reqwest
// ---------------------------------------------------------------------------
async fn os_setup(docs: &[Doc], index: &str) -> Option<(reqwest::Client, String)> {
    let client = reqwest::Client::new();
    let base = os_url();
    if client.get(&base).send().await.is_err() {
        assert!(
            !require_db_peers(),
            "OpenSearch required by LUMEN_REQUIRE_DB_PEERS=1 but unavailable on {base}"
        );
        eprintln!("  ! OpenSearch unavailable on {base}; skipping os");
        return None;
    }
    let _ = client.delete(format!("{base}/{index}")).send().await;
    client
        .put(format!("{base}/{index}"))
        .json(&json!({
            "settings":{"number_of_shards":1,"number_of_replicas":0,"refresh_interval":"-1"},
            "mappings":{"properties":{"bio":{"type":"text"},"city":{"type":"keyword"},"age":{"type":"integer"},"note":{"type":"keyword"}}}
        }))
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
    // bulk ndjson, batched
    for batch in docs.chunks(5000) {
        let mut body = String::new();
        for d in batch {
            body.push_str(&format!("{{\"index\":{{\"_id\":\"{}\"}}}}\n", d.eid));
            // Omit `note` entirely when absent → OS `exists` treats a missing field
            // as "not present" (matching lumen/pg NULL semantics).
            match d.note {
                Some(note) => body.push_str(&format!(
                    "{{\"bio\":\"{}\",\"city\":\"{}\",\"age\":{},\"note\":\"{}\"}}\n",
                    d.bio, d.city, d.age, note
                )),
                None => body.push_str(&format!(
                    "{{\"bio\":\"{}\",\"city\":\"{}\",\"age\":{}}}\n",
                    d.bio, d.city, d.age
                )),
            }
        }
        client
            .post(format!("{base}/{index}/_bulk"))
            .header("content-type", "application/x-ndjson")
            .body(body)
            .send()
            .await
            .unwrap()
            .error_for_status()
            .unwrap();
    }
    client
        .post(format!("{base}/{index}/_refresh"))
        .send()
        .await
        .unwrap();
    client
        .post(format!("{base}/{index}/_forcemerge?max_num_segments=1"))
        .send()
        .await
        .unwrap();
    Some((client, base))
}

async fn measure_os(client: &reqwest::Client, base: &str, index: &str, cell: &str) -> Stat {
    let url = format!("{base}/{index}/_search?request_cache=false");
    let body = os_query(cell);
    let mut e2e = Vec::with_capacity(REPS);
    let mut engine = Vec::with_capacity(REPS);
    for r in 0..(WARMUP + REPS) {
        let t = Instant::now();
        let j: Value = client
            .post(&url)
            .json(&body)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        if r >= WARMUP {
            e2e.push(ms);
            if let Some(took) = j.get("took").and_then(|v| v.as_f64()) {
                engine.push(took); // integer-ms resolution
            }
        }
    }
    summarize(e2e, engine)
}

async fn os_disk_bytes(client: &reqwest::Client, base: &str, index: &str) -> Option<u64> {
    let j: Value = client
        .get(format!("{base}/{index}/_stats/store"))
        .send()
        .await
        .ok()?
        .json()
        .await
        .ok()?;
    j.pointer(&format!("/indices/{index}/total/store/size_in_bytes"))
        .and_then(|v| v.as_u64())
}

// ---------------------------------------------------------------------------
// qps load harness — paced fixed-rate load with bounded in-flight requests.
// ---------------------------------------------------------------------------
const QPS_LADDER: &[usize] = &[10, 100, 1000]; // qps=1 stays the serial min path above
const WARMUP_S: f64 = 2.0;
const PG_MAX_POOL: usize = 90; // under pg max_connections=100, leaving headroom
const NATIVE_MAX_POOL: usize = 16; // low-latency native wire needs fewer persistent sockets

fn parse_cells_env(var: &str) -> Vec<&'static str> {
    let Some(raw) = std::env::var(var).ok().filter(|s| !s.trim().is_empty()) else {
        return CELLS.to_vec();
    };
    let mut out = Vec::new();
    for token in raw.split(',') {
        let cell = token.trim();
        if cell.is_empty() {
            continue;
        }
        let Some(&known) = CELLS.iter().find(|&&known| known == cell) else {
            panic!(
                "unknown {var} entry {cell:?}; valid cells: {}",
                CELLS.join(",")
            );
        };
        out.push(known);
    }
    if out.is_empty() {
        panic!("{var} did not contain any valid cell names");
    }
    out
}

fn parse_scale_cells() -> Vec<&'static str> {
    let Some(raw) = std::env::var("LUMEN_SCALE_CELLS")
        .ok()
        .filter(|value| !value.trim().is_empty())
    else {
        return SCALE_READ_CELLS.to_vec();
    };

    let mut cells = Vec::new();
    for token in raw.split(',') {
        let cell = token.trim();
        if cell.is_empty() {
            continue;
        }
        let Some(&known) = SCALE_READ_CELLS.iter().find(|&&known| known == cell) else {
            panic!(
                "unknown LUMEN_SCALE_CELLS entry {cell:?}; valid scale cells: {}",
                SCALE_READ_CELLS.join(",")
            );
        };
        cells.push(known);
    }
    if cells.is_empty() {
        panic!("LUMEN_SCALE_CELLS did not contain any valid scale cells");
    }
    cells
}

fn is_vector_cell(cell: &str) -> bool {
    matches!(cell, "knn" | "filtered_knn")
}

fn is_opensearch_cell(cell: &str) -> bool {
    // OpenSearch `from` deep pagination is capped by max_result_window on stock
    // nodes. Issue #10's competitive pin is specifically the Postgres OFFSET
    // comparison, so keep OS out of this cell instead of making the gate depend
    // on cluster-level OS settings.
    !is_vector_cell(cell) && cell != "sorted_page_deep"
}

fn gate_n() -> usize {
    std::env::var("LUMEN_GATE_N")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| {
            if env_flag_enabled("LUMEN_GATE_RELEASE_SOAK") {
                RELEASE_SOAK_GATE_N
            } else {
                DEFAULT_GATE_N
            }
        })
}

fn deep_page_offset(n: usize) -> usize {
    n / 2
}

fn parse_gate_cells() -> Vec<&'static str> {
    let cells = parse_cells_env("LUMEN_GATE_CELLS");
    if vector_enabled() {
        cells
    } else {
        // Vector cells need the embedding corpus + HNSW build (LUMEN_GATE_VECTOR=1);
        // drop them from the default gate so it stays vector-free and fast.
        cells.into_iter().filter(|c| !is_vector_cell(c)).collect()
    }
}

fn parse_qps_targets_env(var: &str) -> Vec<usize> {
    let Some(raw) = std::env::var(var).ok().filter(|s| !s.trim().is_empty()) else {
        return QPS_LADDER.to_vec();
    };
    let mut out = Vec::new();
    for token in raw.split(',') {
        let token = token.trim();
        if token.is_empty() {
            continue;
        }
        let qps = token
            .parse::<usize>()
            .unwrap_or_else(|_| panic!("invalid {var} entry {token:?}; expected positive integer"));
        if qps == 0 {
            panic!("{var} entries must be > 0");
        }
        out.push(qps);
    }
    if out.is_empty() {
        panic!("{var} did not contain any valid qps targets");
    }
    out
}

fn parse_gate_qps_targets() -> Vec<usize> {
    parse_qps_targets_env("LUMEN_GATE_QPS_TARGETS")
}

fn parse_scale_qps_targets() -> Vec<usize> {
    parse_qps_targets_env("LUMEN_SCALE_QPS_TARGETS")
}

fn window_s() -> f64 {
    std::env::var("LUMEN_GATE_WINDOW_S")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(6.0)
}

fn qps_gate_enabled() -> bool {
    env_flag_enabled("LUMEN_QPS_GATE") || env_flag_enabled("LUMEN_PERF_STRICT")
}

fn env_flag_enabled(name: &str) -> bool {
    matches!(
        std::env::var(name).ok().as_deref(),
        Some("1" | "true" | "TRUE" | "yes" | "YES")
    )
}

fn qps_gate_threshold(baseline: &Value, qps: usize, peer: &str, ratchet: f64) -> Option<f64> {
    let qps_key = qps.to_string();
    let gate = &baseline["qps_cells"][qps_key.as_str()][peer];
    if gate["gate"].as_str() != Some("win") {
        return None;
    }
    let baseline_ratio = gate["baseline"].as_f64().unwrap_or(1.0);
    Some((baseline_ratio * ratchet).max(1.0))
}

/// One concurrent client's request, per engine. `body=None` ⇒ GET (for /healthz).
#[derive(Clone)]
enum Req {
    Http {
        client: reqwest::Client,
        url: String,
        body: Option<Arc<axum::body::Bytes>>,
    },
    Pg {
        clients: Arc<Vec<tokio_postgres::Client>>,
        stmts: Arc<Vec<tokio_postgres::Statement>>,
        sem: Arc<Semaphore>,
    },
    Native {
        pool: NativePool,
        frame: Arc<Vec<u8>>,
        sem: Arc<Semaphore>,
    },
}

#[derive(Clone)]
enum NativePool {
    #[cfg(unix)]
    Unix(Arc<Vec<Mutex<tokio::net::UnixStream>>>),
    #[cfg(not(unix))]
    Tcp(Arc<Vec<Mutex<tokio::net::TcpStream>>>),
}

impl NativePool {
    fn len(&self) -> usize {
        match self {
            #[cfg(unix)]
            Self::Unix(streams) => streams.len(),
            #[cfg(not(unix))]
            Self::Tcp(streams) => streams.len(),
        }
    }
}

struct Load {
    achieved_qps: f64,
    p50: f64,
    p95: f64,
    p99: f64,
    errors: usize,
    error_rate: f64,
    resources: Option<scale_procmem::ResourceDelta>,
}

async fn issue(req: &Req, worker: usize) -> bool {
    match req {
        Req::Http { client, url, body } => {
            let rb = match body {
                Some(b) => client
                    .post(url)
                    .header(reqwest::header::CONTENT_TYPE, "application/json")
                    .body((**b).clone()),
                None => client.get(url),
            };
            match rb.send().await {
                Ok(response) => match response.error_for_status() {
                    Ok(response) => response.bytes().await.is_ok(), // read full body (server serialize cost)
                    Err(_) => false,
                },
                Err(_) => false,
            }
        }
        Req::Pg {
            clients,
            stmts,
            sem,
        } => {
            let _permit = sem.acquire().await.unwrap(); // queue wait IS the pgbouncer cost
            let i = worker % clients.len();
            clients[i].query(&stmts[i], &[]).await.is_ok()
        }
        Req::Native { pool, frame, sem } => {
            let _permit = sem.acquire().await.unwrap();
            let i = worker % pool.len();
            match pool {
                #[cfg(unix)]
                NativePool::Unix(streams) => {
                    let mut stream = streams[i].lock().await;
                    lumen::native_wire::search_prepared(&mut *stream, frame.as_ref())
                        .await
                        .is_ok()
                }
                #[cfg(not(unix))]
                NativePool::Tcp(streams) => {
                    let mut stream = streams[i].lock().await;
                    lumen::native_wire::search_prepared(&mut *stream, frame.as_ref())
                        .await
                        .is_ok()
                }
            }
        }
    }
}

fn http_json_body(value: Value) -> Arc<axum::body::Bytes> {
    Arc::new(axum::body::Bytes::from(
        serde_json::to_vec(&value).expect("serialize qps request body"),
    ))
}

/// Issue requests at `target_qps` for `WARMUP_S + window_s()`, counting only
/// in-window completions. In-flight work is bounded so an overloaded peer reports
/// lower achieved_qps instead of building an unbounded queue. This makes qps=1000
/// mean 1000 requests/second, not 1000 concurrent tight-loop workers.
async fn run_load(req: Req, target_qps: usize) -> Load {
    let res0 = scale_procmem::resource_sample();
    let start = Instant::now();
    let warmup_end = start + Duration::from_secs_f64(WARMUP_S);
    let window = Duration::from_secs_f64(window_s());
    let window_end = warmup_end + window;
    let interval = Duration::from_secs_f64(1.0 / target_qps.max(1) as f64);
    let max_in_flight = (target_qps.max(1) * 2).clamp(32, 4096);
    let in_flight = Arc::new(Semaphore::new(max_in_flight));
    let mut set: JoinSet<Result<Option<f64>, ()>> = JoinSet::new();
    let mut issued = 0usize;
    loop {
        let due = start + interval.mul_f64(issued as f64);
        if due >= window_end {
            break;
        }
        if let Some(wait) = due.checked_duration_since(Instant::now()) {
            tokio::time::sleep(wait).await;
        }
        let Ok(permit) = in_flight.clone().try_acquire_owned() else {
            issued += 1;
            continue;
        };
        let req = req.clone();
        let worker = issued;
        set.spawn(async move {
            let _permit = permit;
            let t = Instant::now();
            let ok = issue(&req, worker).await;
            if !ok {
                Err(())
            } else if t >= warmup_end && t < window_end {
                Ok(Some(t.elapsed().as_secs_f64() * 1000.0))
            } else {
                Ok(None)
            }
        });
        issued += 1;
    }
    let mut all: Vec<f64> = Vec::new();
    let mut errors = 0usize;
    while let Some(r) = set.join_next().await {
        match r {
            Ok(Ok(Some(s))) => all.push(s),
            Ok(Err(())) | Err(_) => errors += 1,
            Ok(Ok(None)) => {}
        }
    }
    all.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let pct = |q: f64| {
        if all.is_empty() {
            0.0
        } else {
            all[(((all.len() - 1) as f64) * q).round() as usize]
        }
    };
    let total = all.len() + errors;
    let resources = match (res0, scale_procmem::resource_sample()) {
        (Some(before), Some(after)) => Some(after.saturating_delta(before)),
        _ => None,
    };
    Load {
        achieved_qps: all.len() as f64 / window.as_secs_f64(),
        p50: pct(0.50),
        p95: pct(0.95),
        p99: pct(0.99),
        errors,
        error_rate: if total == 0 {
            0.0
        } else {
            errors as f64 / total as f64
        },
        resources,
    }
}

fn resource_cols(load: &Load) -> (String, String, String, String, String, String) {
    match load.resources {
        Some(res) => (
            format!("{:.1}", res.cpu_ms()),
            res.rss_bytes
                .map(|b| format!("{:.1}", scale_procmem::mib(b)))
                .unwrap_or_else(|| "-".into()),
            res.minflt.to_string(),
            res.majflt.to_string(),
            res.inblock.to_string(),
            res.oublock.to_string(),
        ),
        None => (
            "-".into(),
            "-".into(),
            "-".into(),
            "-".into(),
            "-".into(),
            "-".into(),
        ),
    }
}

/// Bounded pg pool of `n` connections, each with the cell's statement PREPARED,
/// plus a Semaphore — the realistic pgbouncer model at qps > conns.
async fn pg_pool(cell: &str, table: &str, n: usize) -> Option<Req> {
    let mut clients = Vec::with_capacity(n);
    let mut stmts = Vec::with_capacity(n);
    for _ in 0..n {
        let (c, conn) = tokio_postgres::connect(&pg_dsn(), tokio_postgres::NoTls)
            .await
            .ok()?;
        tokio::spawn(async move {
            let _ = conn.await;
        });
        let sql = pg_sql(cell, table);
        let s = c.prepare(&sql).await.ok()?;
        clients.push(c);
        stmts.push(s);
    }
    Some(Req::Pg {
        clients: Arc::new(clients),
        stmts: Arc::new(stmts),
        sem: Arc::new(Semaphore::new(n)),
    })
}

#[cfg(unix)]
async fn connect_native_unix(addr: &str) -> Option<tokio::net::UnixStream> {
    for _ in 0..50 {
        match tokio::net::UnixStream::connect(addr).await {
            Ok(stream) => return Some(stream),
            Err(_) => tokio::time::sleep(Duration::from_millis(2)).await,
        }
    }
    None
}

#[cfg(not(unix))]
async fn connect_native_tcp(addr: &str) -> Option<tokio::net::TcpStream> {
    for _ in 0..50 {
        match tokio::net::TcpStream::connect(addr).await {
            Ok(stream) => return Some(stream),
            Err(_) => tokio::time::sleep(Duration::from_millis(2)).await,
        }
    }
    None
}

async fn native_pool(addr: &str, cell: &str, requested: usize) -> Option<Req> {
    let n = requested.max(1).min(NATIVE_MAX_POOL);
    let frame = Arc::new(lumen_native_frame(cell));
    #[cfg(unix)]
    {
        let mut streams = Vec::with_capacity(n);
        for _ in 0..n {
            streams.push(Mutex::new(connect_native_unix(addr).await?));
        }
        return Some(Req::Native {
            pool: NativePool::Unix(Arc::new(streams)),
            frame,
            sem: Arc::new(Semaphore::new(n)),
        });
    }

    #[cfg(not(unix))]
    {
        let mut streams = Vec::with_capacity(n);
        for _ in 0..n {
            streams.push(Mutex::new(connect_native_tcp(addr).await?));
        }
        Some(Req::Native {
            pool: NativePool::Tcp(Arc::new(streams)),
            frame,
            sem: Arc::new(Semaphore::new(n)),
        })
    }
}

/// Pure client+loopback+runtime ceiling at this qps (GET /healthz, ~no server
/// work) — used to flag a cell HARNESS-BOUND (achieved near this) vs SERVER-BOUND.
async fn healthz_ceiling_load(client: &reqwest::Client, base: &str, qps: usize) -> Load {
    run_load(
        Req::Http {
            client: client.clone(),
            url: format!("{base}/healthz"),
            body: None,
        },
        qps,
    )
    .await
}

// ---------------------------------------------------------------------------
// gate
// ---------------------------------------------------------------------------
#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
#[ignore = "competitive perf gate — release-mode Lumen regression; peer compare is explicit"]
async fn competitive_perf_gate() {
    // Default 10k, Lumen-only: peer data is calibrated separately and retained in
    // perf-baseline.json. Re-running pg/OpenSearch is explicit because it is slow
    // and only needed when a benchmark cell or peer configuration changes.
    let n = gate_n();
    let compare_peers = compare_peers_enabled();

    let baseline: Value = serde_json::from_str(include_str!("perf-baseline.json"))
        .expect("perf-baseline.json parses");
    let ratchet = baseline["ratchet"].as_f64().unwrap_or(0.8);
    let pg_table = "docs_mem";
    let os_index = "docs-mem";
    let cells = parse_gate_cells();
    let gate_qps_targets = parse_gate_qps_targets();
    let pg_cheap_cells: Vec<&'static str> = PG_CHEAP_CELLS
        .iter()
        .copied()
        .filter(|cell| cells.contains(cell))
        .collect();
    if std::env::var("LUMEN_GATE_CELLS").is_ok() {
        println!("# LUMEN_GATE_CELLS={cells:?}");
    }
    if std::env::var("LUMEN_GATE_QPS_TARGETS").is_ok() {
        println!("# LUMEN_GATE_QPS_TARGETS={gate_qps_targets:?}");
    }
    println!(
        "# peer comparison: {}",
        if compare_peers {
            "on (refreshing pg/OpenSearch ratios)"
        } else {
            "off (using retained peer-calibrated baselines)"
        }
    );

    println!("\ngenerating corpus N={n} (no-vector, search gate) ...");
    let docs = gen_corpus(n);

    println!("loading lumen ...");
    let (lc, lbase, lengine) = lumen_serve_engine(&docs).await;
    let lnative = if compare_peers {
        Some(lumen_serve_native(lengine.clone()).await)
    } else {
        None
    };
    let pg = if compare_peers {
        println!("loading postgres ...");
        pg_setup(&docs, pg_table).await
    } else {
        None
    };
    let os = if compare_peers {
        println!("loading opensearch ...");
        os_setup(&docs, os_index).await
    } else {
        None
    };
    if compare_peers && require_db_peers() {
        assert!(pg.is_some(), "postgres peer is required for this EC gate");
        assert!(os.is_some(), "OpenSearch peer is required for this EC gate");
    }

    // measure
    let mut lumen_s = std::collections::BTreeMap::new();
    let mut lumen_native_s = std::collections::BTreeMap::new();
    let mut pg_s = std::collections::BTreeMap::new();
    let mut os_s = std::collections::BTreeMap::new();
    for &cell in &cells {
        let lumen_stat = if cell == "sorted_page_deep" {
            measure_lumen_sorted_page_deep(&lc, &lbase, n).await
        } else {
            measure_lumen(&lc, &lbase, cell).await
        };
        lumen_s.insert(cell, lumen_stat);
        if compare_peers && PG_CHEAP_CELLS.contains(&cell) {
            if let Some(lnative) = &lnative {
                lumen_native_s.insert(cell, measure_lumen_native(&lnative.addr, cell).await);
            }
        }
        if let Some(c) = &pg {
            let pg_stat = if cell == "sorted_page_deep" {
                measure_pg_sorted_page_deep(c, pg_table).await
            } else {
                measure_pg(c, cell, pg_table).await
            };
            pg_s.insert(cell, pg_stat);
        }
        if let Some((c, b)) = &os {
            // OS has no k-NN plugin on this host → skip vector cells (peer reported as "-").
            if is_opensearch_cell(cell) {
                os_s.insert(cell, measure_os(c, b, os_index, cell).await);
            }
        }
    }

    // assert + report
    println!("\n=== competitive perf gate (N={n}) — ratio = peer/lumen, >1 = lumen faster ===");
    println!(
        "{:<16} {:>9} {:>9}   {:>13} {:>11}   {:>13} {:>11}",
        "cell", "lum_e2e", "lum_eng", "pg_e2e ratio", "verdict", "os_e2e ratio", "verdict"
    );
    let mut regressions: Vec<String> = Vec::new();
    let mut reds: Vec<String> = Vec::new();

    for &cell in &cells {
        let l = &lumen_s[cell];
        let lum_eng = l.engine_min.unwrap_or(f64::NAN);
        for regression in check_lumen_regression(cell, l, &baseline) {
            regressions.push(regression);
        }

        // vs pg on end-to-end
        let (pg_txt, pg_verdict) = match pg.as_ref().and(pg_s.get(cell)) {
            Some(p) => {
                let ratio = p.e2e_min / l.e2e_min;
                let g = &baseline["cells"][cell]["pg"];
                let (v, fail, red) = judge(g, ratio, ratchet);
                if fail {
                    regressions.push(format!(
                        "{cell} vs pg (e2e): ratio {ratio:.2} below WIN threshold"
                    ));
                }
                if red {
                    reds.push(format!("{cell} vs pg (e2e): ratio {ratio:.2}"));
                }
                (format!("{ratio:.2}x"), v)
            }
            None => ("-".into(), "skip".into()),
        };

        // vs OpenSearch on END-TO-END: lumen and OpenSearch share the HTTP/JSON
        // protocol class, so the transport tax cancels and e2e IS the fair engine
        // comparison — and it is float-precise. OpenSearch's `took` is integer-ms,
        // useless (and wildly flappy) for the ~1ms text/filter cells, so we do NOT
        // gate on it (it is still collected for reference).
        let (os_txt, os_verdict) = match os.as_ref().and(os_s.get(cell)) {
            Some(o) => {
                let ratio = o.e2e_min / l.e2e_min;
                let g = &baseline["cells"][cell]["os"];
                let (v, fail, red) = judge(g, ratio, ratchet);
                if fail {
                    regressions.push(format!(
                        "{cell} vs os (e2e): ratio {ratio:.2} below WIN threshold"
                    ));
                }
                if red {
                    reds.push(format!("{cell} vs os (e2e): ratio {ratio:.2}"));
                }
                (format!("{ratio:.2}x"), v)
            }
            None => ("-".into(), "skip".into()),
        };

        println!(
            "{:<16} {:>9.3} {:>9.3}   {:>13} {:>11}   {:>13} {:>11}",
            cell, l.e2e_min, lum_eng, pg_txt, pg_verdict, os_txt, os_verdict
        );
    }

    if compare_peers {
        println!(
            "\n=== native binary search path (prepared compact frame over Unix socket/TCP fallback) — pg cheap predicate gate ==="
        );
        println!(
            "{:<16} {:>11} {:>11} {:>11} {:>11} {:>13} {:>8}",
            "cell", "native_e2e", "native_eng", "http_e2e", "pg_e2e", "pg/native", "v"
        );
        for &cell in &pg_cheap_cells {
            match (lumen_native_s.get(cell), lumen_s.get(cell), pg_s.get(cell)) {
                (Some(n), Some(h), Some(p)) => {
                    let ratio = p.e2e_min / n.e2e_min;
                    let g = &baseline["cells"][cell]["pg_native"];
                    let (verdict, fail, red) = judge(g, ratio, ratchet);
                    if fail {
                        regressions.push(format!(
                            "{cell} vs pg native (e2e): ratio {ratio:.2} below WIN threshold"
                        ));
                    }
                    if red {
                        reds.push(format!("{cell} vs pg native (e2e): ratio {ratio:.2}"));
                    }
                    println!(
                        "{:<16} {:>11.3} {:>11.3} {:>11.3} {:>11.3} {:>13.2}x {:>8}",
                        cell,
                        n.e2e_min,
                        n.engine_min.unwrap_or(f64::NAN),
                        h.e2e_min,
                        p.e2e_min,
                        ratio,
                        verdict
                    );
                }
                _ => println!(
                    "{:<16} {:>11} {:>11} {:>11} {:>11} {:>13} {:>8}",
                    cell, "-", "-", "-", "-", "-", "skip"
                ),
            }
        }
    }

    // ---- qps axis: concurrent throughput (p50 under load). Default report-only
    // on co-located boxes; `LUMEN_PERF_STRICT=1` (or `LUMEN_QPS_GATE=1` for this
    // test only) turns qps rows recorded in perf-baseline.json into an opt-in
    // strict gate. `LUMEN_GATE_QPS_TARGETS=10` focuses the low-QPS diagnostic
    // row. Co-located HTTP rows pinned at the client/runtime ceiling are retried
    // once if they still beat the peer but miss the ratcheted margin; true
    // losses still fail, and remaining below-floor wins stay TARGETs. Use
    // `--exact` when running this test by name so the disk gate does not run
    // concurrently and distort co-located qps rows. ----
    let qps_peer_strict_requested = qps_gate_enabled();
    let qps_strict = qps_peer_strict_requested && compare_peers;
    if qps_peer_strict_requested && !compare_peers {
        println!(
            "# qps peer strict requested, but peer comparison is off; enforcing Lumen-only qps health"
        );
    }
    let qps_cells: Vec<&'static str> = cells
        .iter()
        .copied()
        .filter(|cell| *cell != "sorted_page_deep")
        .collect();
    println!(
        "\n=== qps axis (window {}s, strict={}) — ratio = peer_p50/lumen_p50 ===",
        window_s(),
        if qps_strict { "on" } else { "off" }
    );
    println!(
        "    resource columns are RUSAGE_SELF samples for this test process; lumen HTTP/native servers run in-process"
    );
    println!(
        "{:<16} {:>5} {:>9} {:>9} {:>9} {:>9} {:>9} {:>7} {:>6} {:>8} {:>8} {:>7} {:>7} {:>7} {:>7}   {:>10} {:>6}   {:>10} {:>6}  {:>5}",
        "cell",
        "qps",
        "lum_qps",
        "lum_p50",
        "lum_p95",
        "lum_p99",
        "h_p50",
        "err%",
        "errors",
        "cpu_ms",
        "rss_mib",
        "minflt",
        "majflt",
        "blk_in",
        "blk_out",
        "pg ratio",
        "v",
        "os ratio",
        "v",
        "sat"
    );
    for &qps in &gate_qps_targets {
        let ceiling = healthz_ceiling_load(&lc, &lbase, qps).await;
        for &cell in &qps_cells {
            let lumen_req = Req::Http {
                client: lc.clone(),
                url: format!("{lbase}/collections/docs/search"),
                body: Some(http_json_body(lumen_query(cell))),
            };
            let mut ll = run_load(lumen_req.clone(), qps).await;
            let pl = if pg.is_some() {
                match pg_pool(cell, pg_table, qps.min(PG_MAX_POOL)).await {
                    Some(p) => Some(run_load(p, qps).await),
                    None => None,
                }
            } else {
                None
            };
            let os_req = os.as_ref().map(|(oc, ob)| Req::Http {
                client: oc.clone(),
                url: format!("{ob}/{os_index}/_search?request_cache=false"),
                body: Some(http_json_body(os_query(cell))),
            });
            let mut ol = if let Some(req) = os_req.clone() {
                Some(run_load(req, qps).await)
            } else {
                None
            };

            if qps_strict {
                if let (Some(threshold), Some(os_load), Some(os_req)) = (
                    qps_gate_threshold(&baseline, qps, "os", ratchet),
                    ol.as_ref(),
                    os_req.clone(),
                ) {
                    let healthy0 = ll.achieved_qps >= 0.9 * qps as f64;
                    let harness_bound0 =
                        ceiling.achieved_qps > 0.0 && ll.achieved_qps >= 0.7 * ceiling.achieved_qps;
                    let ratio0 = if ll.p50 > 0.0 {
                        os_load.p50 / ll.p50
                    } else {
                        0.0
                    };
                    if healthy0
                        && harness_bound0
                        && ratio0 + 1e-9 >= 1.0
                        && ratio0 + 1e-9 < threshold
                    {
                        tokio::time::sleep(Duration::from_millis(500)).await;
                        let retry_ll = run_load(lumen_req.clone(), qps).await;
                        let retry_ol = run_load(os_req, qps).await;
                        let retry_ratio = if retry_ll.p50 > 0.0 {
                            retry_ol.p50 / retry_ll.p50
                        } else {
                            0.0
                        };
                        if retry_ratio > ratio0 {
                            println!(
                                "    qps retry improved {cell} qps{qps} vs os: {ratio0:.2}x -> {retry_ratio:.2}x"
                            );
                            ll = retry_ll;
                            ol = Some(retry_ol);
                        } else {
                            println!(
                                "    qps retry did not improve {cell} qps{qps} vs os: {ratio0:.2}x -> {retry_ratio:.2}x"
                            );
                        }
                    }
                }
            }

            // Only gate if lumen actually drove ~the target AND is not pinned at
            // the client/runtime ceiling (else the harness, not the server, is the limit).
            let healthy = ll.achieved_qps >= 0.9 * qps as f64;
            let harness_bound =
                ceiling.achieved_qps > 0.0 && ll.achieved_qps >= 0.7 * ceiling.achieved_qps;
            let sat = if harness_bound {
                "HARN"
            } else if healthy {
                "ok"
            } else {
                "SVR"
            };
            if !compare_peers {
                if !healthy {
                    regressions.push(format!(
                        "{cell} qps{qps}: lumen achieved {:.0} qps below 90% target",
                        ll.achieved_qps
                    ));
                }
                if ll.errors > 0 {
                    regressions.push(format!(
                        "{cell} qps{qps}: lumen returned {} errors ({:.2}%)",
                        ll.errors,
                        ll.error_rate * 100.0
                    ));
                }
            }

            // Default report-only: under-load p50 on a single co-located box
            // (lumen's server shares CPU with the load client + the 90 pg
            // backends + the OS JVM) is contention-prone. `LUMEN_PERF_STRICT=1`
            // or `LUMEN_QPS_GATE=1` enables a strict gate for rows explicitly present in the
            // `qps_cells` baseline, intended for local/isolation-host proof.
            let verdict = |peer: &str, pl: &Option<Load>| -> (String, &'static str, Option<f64>) {
                match pl {
                    Some(load) if ll.p50 > 0.0 => {
                        let ratio = load.p50 / ll.p50;
                        let is_win = baseline["cells"][cell][peer]["gate"].as_str() == Some("win");
                        let v = if !is_win {
                            "exmpt"
                        } else if ratio + 1e-9 >= 1.0 {
                            "win"
                        } else {
                            "LOSE"
                        };
                        (format!("{ratio:.2}x"), v, Some(ratio))
                    }
                    _ => ("-".into(), "skip", None),
                }
            };
            let (pg_txt, pg_v, _) = verdict("pg", &pl);
            let (os_txt, os_v, os_ratio) = verdict("os", &ol);
            if qps_strict {
                if let Some(threshold) = qps_gate_threshold(&baseline, qps, "os", ratchet) {
                    match os_ratio {
                        Some(_) if !healthy => regressions.push(format!(
                            "{cell} qps{qps} vs os: lumen achieved {:.0} qps below 90% target",
                            ll.achieved_qps
                        )),
                        Some(ratio) if ratio + 1e-9 < 1.0 => regressions.push(format!(
                            "{cell} qps{qps} vs os: ratio {ratio:.2} is a loss"
                        )),
                        Some(ratio) if harness_bound && ratio + 1e-9 < threshold => {
                            reds.push(format!(
                                "{cell} qps{qps} vs os (harness-bound): ratio {ratio:.2} below qps WIN threshold {threshold:.2}"
                            ));
                        }
                        Some(ratio) if ratio + 1e-9 < threshold => regressions.push(format!(
                            "{cell} qps{qps} vs os: ratio {ratio:.2} below qps WIN threshold {threshold:.2}"
                        )),
                        Some(_) => {}
                        None => regressions.push(format!(
                            "{cell} qps{qps} vs os: missing peer row for qps gate"
                        )),
                    }
                }
            }
            let (cpu_ms, rss_mib, minflt, majflt, blk_in, blk_out) = resource_cols(&ll);
            println!(
                "{:<16} {:>5} {:>9.0} {:>9.3} {:>9.3} {:>9.3} {:>9.3} {:>7.2} {:>6} {:>8} {:>8} {:>7} {:>7} {:>7} {:>7}   {:>10} {:>6}   {:>10} {:>6}  {:>5}",
                cell,
                qps,
                ll.achieved_qps,
                ll.p50,
                ll.p95,
                ll.p99,
                ceiling.p50,
                ll.error_rate * 100.0,
                ll.errors,
                cpu_ms,
                rss_mib,
                minflt,
                majflt,
                blk_in,
                blk_out,
                pg_txt,
                pg_v,
                os_txt,
                os_v,
                sat
            );
        }
    }

    if compare_peers {
        println!(
            "\n=== native qps axis (window {}s, strict={}) — ratio = pg_p50/lumen_native_p50 ===",
            window_s(),
            if qps_strict { "on" } else { "off" }
        );
        println!(
            "{:<16} {:>5} {:>9} {:>9} {:>9} {:>9} {:>7} {:>6} {:>8} {:>8} {:>7} {:>7} {:>7} {:>7}   {:>13} {:>8}  {:>5}",
            "cell",
            "qps",
            "nat_qps",
            "nat_p50",
            "nat_p95",
            "nat_p99",
            "err%",
            "errors",
            "cpu_ms",
            "rss_mib",
            "minflt",
            "majflt",
            "blk_in",
            "blk_out",
            "pg/native",
            "v",
            "sat"
        );
        let lnative = lnative
            .as_ref()
            .expect("compare_peers=true creates a native endpoint");
        for &qps in &gate_qps_targets {
            for &cell in &pg_cheap_cells {
                let nl = native_pool(&lnative.addr, cell, qps.min(PG_MAX_POOL))
                    .await
                    .map(|req| async move { run_load(req, qps).await });
                let nl = match nl {
                    Some(fut) => Some(fut.await),
                    None => None,
                };
                let pl = if pg.is_some() {
                    match pg_pool(cell, pg_table, qps.min(PG_MAX_POOL)).await {
                        Some(p) => Some(run_load(p, qps).await),
                        None => None,
                    }
                } else {
                    None
                };

                let (
                    nat_qps,
                    nat_p50,
                    nat_p95,
                    nat_p99,
                    nat_err_pct,
                    nat_errors,
                    ratio,
                    verdict,
                    sat,
                ) = match (&nl, &pl) {
                    (Some(n), Some(p)) if n.p50 > 0.0 => {
                        let ratio = p.p50 / n.p50;
                        let healthy = n.achieved_qps >= 0.9 * qps as f64;
                        let sat = if healthy { "ok" } else { "SVR" };
                        let verdict = if ratio + 1e-9 >= 1.0 { "win" } else { "LOSE" };
                        (
                            n.achieved_qps,
                            n.p50,
                            n.p95,
                            n.p99,
                            n.error_rate * 100.0,
                            n.errors,
                            Some(ratio),
                            verdict,
                            sat,
                        )
                    }
                    (Some(n), _) => (
                        n.achieved_qps,
                        n.p50,
                        n.p95,
                        n.p99,
                        n.error_rate * 100.0,
                        n.errors,
                        None,
                        "skip",
                        "ok",
                    ),
                    _ => (0.0, 0.0, 0.0, 0.0, 0.0, 0, None, "skip", "SVR"),
                };

                if qps_strict {
                    if let Some(threshold) =
                        qps_gate_threshold(&baseline, qps, "pg_native", ratchet)
                    {
                        match (&nl, ratio) {
                        (Some(n), Some(_)) if n.achieved_qps < 0.9 * qps as f64 => {
                            regressions.push(format!(
                                "{cell} native qps{qps} vs pg: lumen native achieved {:.0} qps below 90% target",
                                n.achieved_qps
                            ));
                        }
                        (_, Some(r)) if r + 1e-9 < threshold => regressions.push(format!(
                            "{cell} native qps{qps} vs pg: ratio {r:.2} below qps WIN threshold {threshold:.2}"
                        )),
                        (Some(_), Some(_)) => {}
                        _ => regressions.push(format!(
                            "{cell} native qps{qps} vs pg: missing native or pg row for qps gate"
                        )),
                    }
                    }
                }

                let ratio_txt = ratio
                    .map(|r| format!("{r:.2}x"))
                    .unwrap_or_else(|| "-".into());
                let (cpu_ms, rss_mib, minflt, majflt, blk_in, blk_out) =
                    nl.as_ref().map(resource_cols).unwrap_or_else(|| {
                        (
                            "-".into(),
                            "-".into(),
                            "-".into(),
                            "-".into(),
                            "-".into(),
                            "-".into(),
                        )
                    });
                println!(
                    "{:<16} {:>5} {:>9.0} {:>9.3} {:>9.3} {:>9.3} {:>7.2} {:>6} {:>8} {:>8} {:>7} {:>7} {:>7} {:>7}   {:>13} {:>8}  {:>5}",
                    cell,
                    qps,
                    nat_qps,
                    nat_p50,
                    nat_p95,
                    nat_p99,
                    nat_err_pct,
                    nat_errors,
                    cpu_ms,
                    rss_mib,
                    minflt,
                    majflt,
                    blk_in,
                    blk_out,
                    ratio_txt,
                    verdict,
                    sat
                );
            }
        }
    }

    let lumen_disk_bytes = {
        let dir = tempfile::tempdir().expect("create lumen footprint tempdir");
        lengine
            .flush_to_segments(dir.path(), 1)
            .expect("flush lumen footprint segments");
        scale_segment_bytes(dir.path()).0
    };
    let pg_disk_bytes = if compare_peers {
        match &pg {
            Some(c) => pg_disk_bytes(c, pg_table).await,
            None => None,
        }
    } else {
        None
    };
    let os_disk_bytes = if compare_peers {
        match &os {
            Some((c, b)) => os_disk_bytes(c, b, os_index).await,
            None => None,
        }
    } else {
        None
    };
    println!("\n=== storage footprint (same N={n} corpus; on-disk bytes) ===");
    println!("{:<12} {:>14} {:>12}", "engine", "disk_mib", "bytes/doc");
    let print_footprint = |engine: &str, bytes: Option<u64>| {
        if let Some(bytes) = bytes {
            println!(
                "{:<12} {:>14.2} {:>12.1}",
                engine,
                scale_procmem::mib(bytes),
                bytes as f64 / n as f64
            );
        } else {
            println!("{:<12} {:>14} {:>12}", engine, "-", "-");
        }
    };
    print_footprint("lumen", Some(lumen_disk_bytes));
    print_footprint("postgres", pg_disk_bytes);
    print_footprint("opensearch", os_disk_bytes);

    if !reds.is_empty() {
        println!("\nTARGET (expected red — drives the work, NOT a gate failure):");
        for r in &reds {
            println!("  · {r}");
        }
    }
    println!(
        "\nNOTE: default mode is Lumen-only. pg/OpenSearch ratios in perf-baseline.json \
         are retained calibration evidence and are not remeasured unless \
         LUMEN_GATE_COMPARE_PEERS=1 is set. In peer-compare mode, vs-OS and vs-pg are \
         gated on end-to-end; lumen + OpenSearch share HTTP/JSON while pg cheap \
         predicate HTTP cells stay EXEMPT and are covered by the native binary table. \
         LUMEN_GATE_QPS_TARGETS can focus one qps tier."
    );

    if !regressions.is_empty() {
        println!("\nGATE FAILED — WIN-cell regressions:");
        for r in &regressions {
            println!("  x {r}");
        }
        panic!(
            "competitive perf gate: {} WIN-cell regression(s)",
            regressions.len()
        );
    }
    println!("\nGATE PASSED (no WIN-cell regressions).");
}

/// Returns (verdict label, is_win_regression, is_target_red).
fn judge(g: &Value, ratio: f64, ratchet: f64) -> (String, bool, bool) {
    match g["gate"].as_str().unwrap_or("exempt") {
        "win" => {
            let base = g["baseline"].as_f64().unwrap_or(1.0);
            let req = (ratchet * base).max(1.0);
            if ratio + 1e-9 >= req {
                (format!("WIN ok>={req:.1}"), false, false)
            } else {
                (format!("WIN<{req:.1}"), true, false)
            }
        }
        "target" => {
            let floor = g["floor"].as_f64().unwrap_or(1.0);
            if ratio + 1e-9 >= floor {
                ("TARGET ok".into(), false, false)
            } else {
                ("TARGET red".into(), false, true)
            }
        }
        _ => ("exempt".into(), false, false),
    }
}

fn check_lumen_regression(cell: &str, stat: &Stat, baseline: &Value) -> Vec<String> {
    let gate = &baseline["lumen_cells"][cell];
    let mut regressions = Vec::new();
    let mut check = |name: &str, actual: f64, key: &str| {
        if let Some(max) = gate[key].as_f64() {
            if actual.is_finite() && actual > max {
                regressions.push(format!(
                    "{cell} lumen {name}: {actual:.3}ms above {max:.3}ms regression floor"
                ));
            }
        }
    };
    check("e2e min", stat.e2e_min, "max_e2e_min_ms");
    check("e2e p50", stat.e2e_p50, "max_e2e_p50_ms");
    if let Some(engine_min) = stat.engine_min {
        check("engine min", engine_min, "max_engine_min_ms");
    }
    regressions
}

// ===========================================================================
// DISK-TIER GATE (Stage 2): does lumen's SEGMENT-backed query path still beat
// pg + OpenSearch? The in-memory gate above proves the RAM engine wins; this
// proves the claim survives the disk tier — the engine reads its forward/
// inverted payload off the mmap'd `.lseg` segments (demand-paged from the warm
// page cache), the same query path a `flush_to_segments`'d production node runs.
//
// Test-only, additive, `#[ignore]` (needs pg + OpenSearch up). Run with:
//   cargo test --release -p lumen \
//       --test perf_gate_vs_db -- --ignored --nocapture
// ===========================================================================

/// DISK-backed lumen serve: index the corpus exactly like [`lumen_serve`], then
/// `flush_to_segments` the SAME live engine IN PLACE — sealing every collection,
/// DROPPING the in-RAM forward/inverted drivers, and attaching the segment
/// readers. Subsequent `/search` calls on the same server now read the mmap'd
/// columns (warm page cache), so `measure_lumen`'s `took_us` reflects the disk
/// path. This is the headline "lumen running with its disk tier" scenario: same
/// process, warm cache, drivers genuinely gone from RAM.
///
/// Returns the client/base for `measure_lumen`, the engine handle (to probe),
/// and the `TempDir` — which MUST be kept alive for the duration of the test so
/// the segment mmaps stay mapped (dropping it unlinks the `.lseg` files).
async fn lumen_serve_disk(
    docs: &[Doc],
) -> (
    reqwest::Client,
    String,
    Arc<lumen::storage::Engine>,
    tempfile::TempDir,
) {
    let (client, base, engine) = lumen_serve_engine(docs).await;
    let dir = tempfile::tempdir().unwrap();
    // Seal in place: drivers dropped, segment readers attached on THIS engine.
    engine
        .flush_to_segments(dir.path(), 1)
        .expect("flush_to_segments (disk tier)");
    (client, base, engine, dir)
}

/// Assert the disk path is GENUINELY segment-driven (not silently still in RAM):
/// after `flush_to_segments`, every representative field's in-RAM driver must be
/// EMPTY (forward/tokens len 0 — dropped to disk) AND a segment must be attached.
/// If this fails, the "disk" numbers would be a lie (RAM reads), so it is a HARD
/// assert, not a report. Uses `Engine::segment_field_probe`.
fn assert_segment_backed(engine: &lumen::storage::Engine) {
    for field in ["bio", "city", "age", "doc_key", "sort_key"] {
        let (driver_len, has_segment) = engine
            .segment_field_probe("docs", field)
            .unwrap_or_else(|e| panic!("probe field `{field}`: {e}"));
        assert!(
            has_segment,
            "field `{field}`: no segment attached after flush_to_segments — \
             the disk path is NOT segment-backed"
        );
        assert_eq!(
            driver_len, 0,
            "field `{field}`: in-RAM driver still holds {driver_len} entries after \
             flush — queries would read RAM, not the segment; disk numbers invalid"
        );
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
#[ignore = "DISK-tier explicit peer calibration gate — requires Postgres + OpenSearch"]
async fn competitive_perf_gate_disk() {
    // Explicit peer disk calibration keeps the retained N=1M proof available
    // without making routine AW/release runs pay this cost.
    let n = gate_n();

    let baseline: Value = serde_json::from_str(include_str!("perf-baseline.json"))
        .expect("perf-baseline.json parses");
    let ratchet = baseline["ratchet"].as_f64().unwrap_or(0.8);
    let pg_table = "docs_disk";
    let os_index = "docs-disk";

    println!("\ngenerating corpus N={n} (no-vector, search gate) ...");
    let docs = gen_corpus(n);

    println!("loading lumen IN-MEM (overhead reference column) ...");
    let (lc_mem, lbase_mem) = lumen_serve(&docs).await;

    println!("loading lumen DISK (flush_to_segments in place — drivers dropped) ...");
    let (lc_disk, lbase_disk, disk_engine, _disk_dir) = lumen_serve_disk(&docs).await;

    // GENUINELY segment-driven? Probe before measuring so a silent RAM fallback
    // cannot masquerade as a disk number.
    assert_segment_backed(&disk_engine);
    println!(
        "  ✓ disk path confirmed segment-backed (bio/city/age drivers dropped, segments attached)"
    );

    println!("loading postgres ...");
    let pg = pg_setup(&docs, pg_table).await;
    println!("loading opensearch ...");
    let os = os_setup(&docs, os_index).await;
    let cells = parse_gate_cells();

    // measure every engine on every cell
    let mut disk_s = std::collections::BTreeMap::new();
    let mut mem_s = std::collections::BTreeMap::new();
    let mut pg_s = std::collections::BTreeMap::new();
    let mut os_s = std::collections::BTreeMap::new();
    for &cell in &cells {
        let disk_stat = if cell == "sorted_page_deep" {
            measure_lumen_sorted_page_deep(&lc_disk, &lbase_disk, n).await
        } else {
            measure_lumen(&lc_disk, &lbase_disk, cell).await
        };
        let mem_stat = if cell == "sorted_page_deep" {
            measure_lumen_sorted_page_deep(&lc_mem, &lbase_mem, n).await
        } else {
            measure_lumen(&lc_mem, &lbase_mem, cell).await
        };
        disk_s.insert(cell, disk_stat);
        mem_s.insert(cell, mem_stat);
        if let Some(c) = &pg {
            let pg_stat = if cell == "sorted_page_deep" {
                measure_pg_sorted_page_deep(c, pg_table).await
            } else {
                measure_pg(c, cell, pg_table).await
            };
            pg_s.insert(cell, pg_stat);
        }
        if let Some((c, b)) = &os {
            // OS has no k-NN plugin on this host, and sorted_page_deep is pg-only.
            if is_opensearch_cell(cell) {
                os_s.insert(cell, measure_os(c, b, os_index, cell).await);
            }
        }
    }

    // ---- report + assert. e2e_min is the float-precise comparable (same HTTP
    // path for disk + in-mem + OpenSearch). ratio = peer/lumen_disk, >1 = lumen
    // faster. ovh = disk_e2e / inmem_e2e (warm-cache disk overhead vs RAM). ----
    println!(
        "\n=== DISK-tier competitive perf gate (N={n}) — ratio = peer/lumen_DISK, >1 = lumen faster ==="
    );
    println!(
        "{:<16} {:>9} {:>9} {:>6}   {:>9} {:>9}   {:>11} {:>11}   {:>11} {:>11}",
        "cell",
        "disk_e2e",
        "mem_e2e",
        "ovh",
        "pg_e2e",
        "os_e2e",
        "pg ratio",
        "verdict",
        "os ratio",
        "verdict"
    );
    let mut regressions: Vec<String> = Vec::new();
    let mut reds: Vec<String> = Vec::new();

    for &cell in &cells {
        let d = &disk_s[cell];
        let m = &mem_s[cell];
        let ovh = if m.e2e_min > 0.0 {
            d.e2e_min / m.e2e_min
        } else {
            f64::NAN
        };

        // vs pg on end-to-end (using the DISK lumen as the denominator)
        let (pg_e2e, pg_txt, pg_verdict) = match pg.as_ref().and(pg_s.get(cell)) {
            Some(p) => {
                let ratio = p.e2e_min / d.e2e_min;
                let g = &baseline["cells"][cell]["pg"];
                let (v, fail, red) = judge(g, ratio, ratchet);
                if fail {
                    regressions.push(format!(
                        "{cell} vs pg (disk e2e): ratio {ratio:.2} below WIN threshold"
                    ));
                }
                if red {
                    reds.push(format!("{cell} vs pg (disk e2e): ratio {ratio:.2}"));
                }
                (format!("{:.3}", p.e2e_min), format!("{ratio:.2}x"), v)
            }
            None => ("-".into(), "-".into(), "skip".into()),
        };

        // vs OpenSearch on end-to-end (shared HTTP/JSON class; float-precise)
        let (os_e2e, os_txt, os_verdict) = match os.as_ref().and(os_s.get(cell)) {
            Some(o) => {
                let ratio = o.e2e_min / d.e2e_min;
                let g = &baseline["cells"][cell]["os"];
                let (v, fail, red) = judge(g, ratio, ratchet);
                if fail {
                    regressions.push(format!(
                        "{cell} vs os (disk e2e): ratio {ratio:.2} below WIN threshold"
                    ));
                }
                if red {
                    reds.push(format!("{cell} vs os (disk e2e): ratio {ratio:.2}"));
                }
                (format!("{:.3}", o.e2e_min), format!("{ratio:.2}x"), v)
            }
            None => ("-".into(), "-".into(), "skip".into()),
        };

        println!(
            "{:<16} {:>9.3} {:>9.3} {:>5.2}x   {:>9} {:>9}   {:>11} {:>11}   {:>11} {:>11}",
            cell, d.e2e_min, m.e2e_min, ovh, pg_e2e, os_e2e, pg_txt, pg_verdict, os_txt, os_verdict
        );
    }

    // engine-only took_us column (reference): disk vs in-mem internal cost.
    println!("\n=== engine-only took (lum_eng, ms) — disk vs in-mem internal cost ===");
    println!(
        "{:<16} {:>10} {:>10} {:>8}",
        "cell", "disk_eng", "mem_eng", "ovh"
    );
    for &cell in &cells {
        let de = disk_s[cell].engine_min.unwrap_or(f64::NAN);
        let me = mem_s[cell].engine_min.unwrap_or(f64::NAN);
        let ovh = if me > 0.0 { de / me } else { f64::NAN };
        println!("{:<16} {:>10.4} {:>10.4} {:>7.2}x", cell, de, me, ovh);
    }

    if !reds.is_empty() {
        println!("\nTARGET (expected red — drives the work, NOT a gate failure):");
        for r in &reds {
            println!("  · {r}");
        }
    }
    println!(
        "\nNOTE: DISK tier = lumen serving from mmap'd `.lseg` segments after \
         flush_to_segments dropped the in-RAM drivers (probe-confirmed above). Warm \
         page cache (same process, just sealed), so this is the fair 'lumen with its \
         disk tier' latency. ratio = peer/lumen_DISK on end-to-end (the float-precise \
         comparable; OpenSearch's integer-ms `took` is too coarse to gate). ovh = \
         disk_e2e / inmem_e2e. pg's binary protocol wins cheap-predicate cells on \
         loopback (EXEMPT). Same ratchet/WIN logic as the in-mem gate — WIN cells must \
         still beat the gated opponent ON DISK or the build fails."
    );

    if !regressions.is_empty() {
        // HARD GATE (Phase 2m): the BOUNDED decoded-posting cache + sort-via-
        // sorted-index landed, so the segment query path now serves resident
        // `Arc<RoaringBitmap>`s on a warm hit (the inverted-index hot-zone that
        // parallels the OS page cache for the forward payload) and walks the
        // on-disk sorted-value index for sort instead of rebuilding the whole-field
        // BTreeMap per query. The low-cardinality number/range/sort/keyword cells
        // that regressed 5x-525x are recovered, so the WIN cells must beat pg+OS ON
        // DISK or the build fails — same ratchet/WIN logic as the in-mem gate.
        println!("\nDISK GATE FAILED — WIN-cell regressions on the segment path:");
        for r in &regressions {
            println!("  x {r}");
        }
        panic!(
            "DISK competitive perf gate: {} WIN-cell regression(s) on the segment path",
            regressions.len()
        );
    }
    println!("\nDISK GATE PASSED (no WIN-cell regressions on the segment path).");
}

// ===========================================================================
// LUMEN-ONLY SCALE BENCH (Stage 2): measure the DISK engine across the
// project-standard local row-count ladder with NO Postgres and NO OpenSearch.
// The standard cap is 100k docs; 1M+ rows are explicit release-soak/research runs
// so local benchmark cost does not become the development bottleneck.
// For each N it stream-generates docs, indexes directly via the Engine API (NOT
// over HTTP), `flush_to_segments` so queries are segment-backed (the disk path),
// wraps the SAME engine in the axum server so `measure_lumen`/`run_load` hit it
// over HTTP, and reports per-cell latency + optional qps ladder PLUS per-N
// storage facts: on-disk segment MiB, bytes/doc (+ per-field breakdown), peak
// process RSS, and the RSS/on-disk ratio. It is a MEASUREMENT/REPORT — no WIN
// assertions; it only sanity-asserts results are non-empty and index bytes > 0.
//
// Reuses the corpus, HTTP, and paced-load mechanics from this same file. The
// fixed SCALE_READ_CELLS selectors and scale_lumen_query stay independent from
// the competitive peer matrix.
//
//   cargo test --release -p lumen --test perf_gate_vs_db -- \
//       --ignored --nocapture lumen_scale_bench
//   LUMEN_SCALE_ALLOW_ABOVE_STANDARD=1 LUMEN_GATE_WINDOW_S=0.2 LUMEN_SCALE_CHUNK_ROWS=100000 \
//       LUMEN_SCALE_ROWS=1000000 \
//       cargo test --release -p lumen --test perf_gate_vs_db -- \
//       --ignored --nocapture lumen_scale_bench   # reopened-shard HTTP qps smoke
// ===========================================================================

// Test-only RSS helper — a tiny copy of disk_scale_proof.rs `procmem` (a Rust
// integration test can't import another test binary's private mod, and
// `memory-stats` is already a dev-dep). Reads cross-platform PHYSICAL resident
// set size (macOS `task_info`, Linux `/proc/self/statm`), normalized to BYTES.
mod scale_procmem {
    #[derive(Clone, Copy, Debug)]
    pub struct ResourceSample {
        pub user_us: u64,
        pub sys_us: u64,
        pub minflt: u64,
        pub majflt: u64,
        pub inblock: u64,
        pub oublock: u64,
        pub rss_bytes: Option<u64>,
    }

    #[derive(Clone, Copy, Debug)]
    pub struct ResourceDelta {
        pub user_ms: f64,
        pub sys_ms: f64,
        pub minflt: u64,
        pub majflt: u64,
        pub inblock: u64,
        pub oublock: u64,
        pub rss_bytes: Option<u64>,
    }

    impl ResourceDelta {
        pub fn cpu_ms(self) -> f64 {
            self.user_ms + self.sys_ms
        }
    }

    impl ResourceSample {
        pub fn saturating_delta(self, before: Self) -> ResourceDelta {
            ResourceDelta {
                user_ms: self.user_us.saturating_sub(before.user_us) as f64 / 1000.0,
                sys_ms: self.sys_us.saturating_sub(before.sys_us) as f64 / 1000.0,
                minflt: self.minflt.saturating_sub(before.minflt),
                majflt: self.majflt.saturating_sub(before.majflt),
                inblock: self.inblock.saturating_sub(before.inblock),
                oublock: self.oublock.saturating_sub(before.oublock),
                rss_bytes: self.rss_bytes,
            }
        }
    }

    /// Current physical resident set size in BYTES, or `None` if unavailable.
    pub fn rss_bytes() -> Option<u64> {
        memory_stats::memory_stats().map(|m| m.physical_mem as u64)
    }

    #[cfg(unix)]
    fn timeval_us(tv: libc::timeval) -> u64 {
        let sec = u64::try_from(tv.tv_sec).unwrap_or(0);
        let usec = u64::try_from(tv.tv_usec).unwrap_or(0);
        sec.saturating_mul(1_000_000).saturating_add(usec)
    }

    #[cfg(unix)]
    fn signed_to_u64(v: libc::c_long) -> u64 {
        u64::try_from(v).unwrap_or(0)
    }

    pub fn resource_sample() -> Option<ResourceSample> {
        #[cfg(unix)]
        unsafe {
            let mut ru: libc::rusage = std::mem::zeroed();
            if libc::getrusage(libc::RUSAGE_SELF, &mut ru) != 0 {
                return None;
            }
            Some(ResourceSample {
                user_us: timeval_us(ru.ru_utime),
                sys_us: timeval_us(ru.ru_stime),
                minflt: signed_to_u64(ru.ru_minflt),
                majflt: signed_to_u64(ru.ru_majflt),
                inblock: signed_to_u64(ru.ru_inblock),
                oublock: signed_to_u64(ru.ru_oublock),
                rss_bytes: rss_bytes(),
            })
        }
        #[cfg(not(unix))]
        {
            None
        }
    }

    /// Format a byte count as MiB.
    pub fn mib(bytes: u64) -> f64 {
        bytes as f64 / (1024.0 * 1024.0)
    }
}

/// Sum every `*.lseg` byte under the checkpoint dir (recursively over the
/// per-collection `<hexcollection>/` subdirs) AND break the total down by FIELD.
/// Segment file names encode the field, so we attribute each `.lseg` to its
/// field by stripping the trailing `__<part>.lseg` / `.lseg` suffix and keying on
/// the leading field token. Returns `(total_bytes, per_field_bytes)`.
fn scale_segment_bytes(dir: &std::path::Path) -> (u64, std::collections::BTreeMap<String, u64>) {
    fn walk(
        p: &std::path::Path,
        total: &mut u64,
        by_field: &mut std::collections::BTreeMap<String, u64>,
    ) {
        let Ok(rd) = std::fs::read_dir(p) else { return };
        for ent in rd.flatten() {
            let path = ent.path();
            if path.is_dir() {
                walk(&path, total, by_field);
            } else if path.extension().and_then(|e| e.to_str()) == Some("lseg") {
                if let Ok(md) = std::fs::metadata(&path) {
                    let len = md.len();
                    *total += len;
                    // Field attribution: segment files are named `<field>.lseg`,
                    // with sidecars `<field>.eids.lseg` and the EID/meta column
                    // `_collection.lmeta.lseg`. The leading dot-separated token IS
                    // the field (e.g. `bio.lseg`→`bio`, `bio.eids.lseg`→`bio`,
                    // `_collection.lmeta.lseg`→`_collection`).
                    let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("?");
                    let field = name.split('.').next().unwrap_or(name).to_string();
                    *by_field.entry(field).or_insert(0) += len;
                }
            }
        }
    }
    let mut total = 0u64;
    let mut by_field = std::collections::BTreeMap::new();
    walk(dir, &mut total, &mut by_field);
    (total, by_field)
}

fn scale_collection_fields() -> std::collections::BTreeMap<String, lumen::types::FieldSpec> {
    use lumen::types::{FieldSpec, FieldType};
    let spec = |ft: FieldType| FieldSpec {
        field_type: ft,
        analyzer: None,
        multi: None,
        dim: None,
        metric: None,
        backend: None,
        quantize: None,
    };
    let mut fields = std::collections::BTreeMap::new();
    fields.insert("bio".to_string(), spec(FieldType::Text));
    fields.insert("city".to_string(), spec(FieldType::Keyword));
    fields.insert("age".to_string(), spec(FieldType::Number));
    fields.insert("doc_key".to_string(), spec(FieldType::Keyword));
    fields.insert("sort_key".to_string(), spec(FieldType::Keyword));
    fields
}

fn create_scale_collection(engine: &lumen::storage::Engine, collection_id: &str) {
    use lumen::types::CreateCollectionRequest;
    engine
        .create_collection(
            collection_id,
            CreateCollectionRequest {
                fields: scale_collection_fields(),
            },
        )
        .expect("create_collection docs");
}

fn scale_doc_key(document: usize) -> String {
    format!("scale-key-{document:010}")
}

/// Present for nine out of every ten rows. The value is unique and sorted by
/// document ordinal, while the omitted tenth exercises `missing:last`.
fn scale_sort_key(document: usize) -> Option<String> {
    (document % 10 != 0).then(|| format!("scale-sort-{document:010}"))
}

/// Stream-index docs DIRECTLY through the `Engine::index` API (NOT over HTTP)
/// in batches under the bulk cap. For explicit above-1M research runs, the
/// in-process call path keeps benchmark overhead below HTTP/JSON write overhead.
/// Up to five items per doc (bio/city/age/doc_key/sort_key) mirror the HTTP
/// indexer exactly. The
/// corpus is generated on the fly, so large research runs do not first allocate a
/// `Vec<Doc>`.
fn scale_index_direct_range(
    engine: &lumen::storage::Engine,
    collection_id: &str,
    start: usize,
    count: usize,
    total: usize,
    completed_before: usize,
    rng: &mut Lcg,
) {
    use lumen::types::{FieldValue, IndexItem, IndexRequest};
    const BATCH_DOCS: usize = 2_000;
    const MAX_ITEMS: usize = lumen::types::MAX_INDEX_BATCH_SIZE;
    let progress_every = std::env::var("LUMEN_SCALE_PROGRESS_EVERY")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(10_000_000)
        .max(1);
    let mut items: Vec<IndexItem> = Vec::with_capacity(BATCH_DOCS * 5);
    let flush = |items: &mut Vec<IndexItem>| {
        if items.is_empty() {
            return;
        }
        engine
            .index(
                collection_id,
                IndexRequest {
                    items: std::mem::take(items),
                    request_id: None,
                },
            )
            .expect("engine.index (direct)");
    };
    for local_i in 0..count {
        // A sparse row has four or five fields. Flush before adding the next
        // maximum-size row so a near-full batch can never cross the public
        // 10,000-item index cap.
        if items.len().saturating_add(5) > MAX_ITEMS {
            flush(&mut items);
        }
        let global_i = start + local_i;
        let d = gen_doc(global_i, rng);
        items.push(IndexItem {
            external_id: d.eid.clone(),
            field: "bio".into(),
            value: FieldValue::String(d.bio),
            version: None,
        });
        items.push(IndexItem {
            external_id: d.eid.clone(),
            field: "city".into(),
            value: FieldValue::String(d.city.to_string()),
            version: None,
        });
        items.push(IndexItem {
            external_id: d.eid.clone(),
            field: "age".into(),
            value: FieldValue::Number(d.age as f64),
            version: None,
        });
        items.push(IndexItem {
            external_id: d.eid,
            field: "doc_key".into(),
            value: FieldValue::String(scale_doc_key(global_i)),
            version: None,
        });
        if let Some(sort_key) = scale_sort_key(global_i) {
            items.push(IndexItem {
                external_id: format!("d{global_i}"),
                field: "sort_key".into(),
                value: FieldValue::String(sort_key),
                version: None,
            });
        }
        let done = completed_before + local_i + 1;
        if total >= progress_every && done % progress_every == 0 {
            println!(
                "    indexed {done}/{total} docs ({:.1}%)",
                (done as f64 / total as f64) * 100.0
            );
        }
    }
    flush(&mut items);
}

fn scale_index_direct(engine: &lumen::storage::Engine, n: usize) {
    let mut rng = Lcg::new(SEED);
    scale_index_direct_range(engine, "docs", 0, n, n, 0, &mut rng);
}

fn scale_document_items(document: usize) -> Vec<Value> {
    let mut rng = Lcg::for_doc(document);
    let doc = gen_doc(document, &mut rng);
    let mut items = vec![
        json!({"external_id":doc.eid,"field":"bio","value":doc.bio}),
        json!({"external_id":format!("d{document}"),"field":"city","value":doc.city}),
        json!({"external_id":format!("d{document}"),"field":"age","value":doc.age}),
        json!({"external_id":format!("d{document}"),"field":"doc_key","value":scale_doc_key(document)}),
    ];
    if let Some(sort_key) = scale_sort_key(document) {
        items.push(
            json!({"external_id":format!("d{document}"),"field":"sort_key","value":sort_key}),
        );
    }
    items
}

fn scale_sentinel_items() -> Vec<Value> {
    vec![
        json!({"external_id":"scale-unindex-survivor","field":"bio","value":"scale sentinel"}),
        json!({"external_id":"scale-unindex-survivor","field":"city","value":"sentinel"}),
        json!({"external_id":"scale-unindex-survivor","field":"age","value":-1}),
        json!({"external_id":"scale-unindex-survivor","field":"doc_key","value":"scale-sentinel"}),
        json!({"external_id":"scale-unindex-survivor","field":"sort_key","value":"scale-sentinel"}),
    ]
}

async fn scale_reindex_documents(
    client: &reqwest::Client,
    base: &str,
    documents: impl IntoIterator<Item = usize>,
) {
    // Sparse fixture rows have four or five fields. Check before appending a
    // whole row so a 996-item batch cannot cross the public item cap with a
    // five-item row.
    const MAX_ITEMS: usize = lumen::types::MAX_INDEX_BATCH_SIZE;
    let mut items = Vec::with_capacity(MAX_ITEMS);
    for document in documents {
        let document_items = scale_document_items(document);
        if items.len().saturating_add(document_items.len()) > MAX_ITEMS {
            post_index(client, base, &items).await;
            items.clear();
        }
        items.extend(document_items);
    }
    if !items.is_empty() {
        post_index(client, base, &items).await;
    }
}

async fn scale_search_json(client: &reqwest::Client, base: &str, body: Value) -> Value {
    client
        .post(format!("{base}/collections/docs/search"))
        .json(&body)
        .send()
        .await
        .expect("scale search request")
        .error_for_status()
        .expect("scale search success status")
        .json()
        .await
        .expect("scale search JSON")
}

/// First-use timings for the #3997 query shape. Each limit changes the exact
/// request-cache key, so this reports a cold response-cache request without
/// adding a write (which would measure mutation work rather than sort work).
/// These are evidence rows only. They intentionally have no latency floor.
async fn scale_measure_keyword_sort_cold(client: &reqwest::Client, base: &str, documents: usize) {
    for limit in [1u32, 20, 80, 500, 2_000] {
        let mut query = scale_lumen_query("keyword_sort", None);
        query["limit"] = json!(limit);
        let started = Instant::now();
        let response = scale_search_json(client, base, query).await;
        let elapsed = started.elapsed();
        let expected_len = documents.min(limit as usize);
        assert_eq!(
            scale_response_ids(&response, "keyword_sort cold").len(),
            expected_len,
            "cold keyword_sort limit={limit} returned an unexpected page size"
        );
        assert_eq!(
            response["total"].as_u64(),
            Some(documents as u64),
            "cold keyword_sort limit={limit} lost missing rows from the exact total"
        );
        println!(
            "  keyword_sort cold_response_cache: N={documents} limit={limit} sparse=90% missing=last track_total=true e2e={:.3}ms took_us={}",
            elapsed.as_secs_f64() * 1000.0,
            response["took_us"].as_u64().unwrap_or(0),
        );
    }
}

async fn scale_documents_indexed(client: &reqwest::Client, base: &str) -> u64 {
    client
        .get(format!("{base}/collections/docs/stats"))
        .send()
        .await
        .expect("scale stats request")
        .error_for_status()
        .expect("scale stats success status")
        .json::<Value>()
        .await
        .expect("scale stats JSON")["documents_indexed"]
        .as_u64()
        .expect("scale stats documents_indexed")
}

fn assert_scale_page(response: &Value, start: usize, expected_len: usize) {
    let hits = response["hits"]
        .as_array()
        .expect("scale cursor hits array");
    assert_eq!(
        hits.len(),
        expected_len,
        "scale cursor page has an unexpected number of hits"
    );
    let expected: Vec<_> = (start..start + expected_len)
        .map(|document| format!("d{document}"))
        .collect();
    let actual: Vec<_> = hits
        .iter()
        .map(|hit| {
            hit["external_id"]
                .as_str()
                .expect("scale cursor external_id")
                .to_string()
        })
        .collect();
    assert_eq!(actual, expected, "scale cursor order changed");
}

fn scale_response_ids(response: &Value, cell: &str) -> Vec<String> {
    response["hits"]
        .as_array()
        .unwrap_or_else(|| panic!("scale {cell} preflight hits array"))
        .iter()
        .map(|hit| {
            hit["external_id"]
                .as_str()
                .unwrap_or_else(|| panic!("scale {cell} preflight external_id"))
                .to_string()
        })
        .collect()
}

async fn scale_preflight_fixture(client: &reqwest::Client, base: &str, documents: usize) {
    assert_eq!(
        scale_documents_indexed(client, base).await,
        documents as u64,
        "scale preflight fixture count differs from N={documents} before measurements"
    );

    let range = scale_search_json(client, base, scale_lumen_query("range", None)).await;
    let expected_range: Vec<String> = (0..documents)
        .filter_map(|document| {
            let mut rng = Lcg::for_doc(document);
            let doc = gen_doc(document, &mut rng);
            (doc.city == "taipei" && (30..40).contains(&doc.age)).then_some(doc.eid)
        })
        .take(10)
        .collect();
    // Current 0.4 constant-score filters use posting/internal-docid order. This
    // fixture indexes d0..dN sequentially, so document ordinal is the stable
    // hard oracle. External-id ordering is a separate Search v2 target contract.
    assert_eq!(
        scale_response_ids(&range, "range"),
        expected_range,
        "scale preflight range filter result set/order changed from deterministic fixture document order"
    );

    let filter_sort = scale_search_json(client, base, scale_lumen_query("filter_sort", None)).await;
    let mut expected_filter_sort: Vec<(i32, usize)> = (0..documents)
        .map(|document| (doc_age(&format!("d{document}")), document))
        .collect();
    expected_filter_sort.sort_by(|(left_age, left_document), (right_age, right_document)| {
        left_age
            .cmp(right_age)
            .then_with(|| left_document.cmp(right_document))
    });
    let expected_filter_sort: Vec<String> = expected_filter_sort
        .into_iter()
        .take(10)
        .map(|(_, document)| format!("d{document}"))
        .collect();
    assert_eq!(
        scale_response_ids(&filter_sort, "filter_sort"),
        expected_filter_sort,
        "scale preflight numeric sort must be age ascending with deterministic fixture document-order tie-break"
    );

    let keyword_sort =
        scale_search_json(client, base, scale_lumen_query("keyword_sort", None)).await;
    let expected_keyword_sort: Vec<String> = (0..documents)
        .filter(|document| scale_sort_key(*document).is_some())
        .take(80)
        .map(|document| format!("d{document}"))
        .collect();
    assert_eq!(
        scale_response_ids(&keyword_sort, "keyword_sort"),
        expected_keyword_sort,
        "scale preflight sparse high-cardinality missing:last keyword sort changed"
    );
    assert_eq!(
        keyword_sort["total"].as_u64(),
        Some(documents as u64),
        "scale preflight keyword_sort must keep an exact total including missing rows"
    );

    let sorted_page_deep =
        scale_search_json(client, base, scale_lumen_query("sorted_page_deep", None)).await;
    assert_scale_page(
        &sorted_page_deep,
        0,
        usize::try_from(SCALE_CURSOR_PAGE_SIZE).expect("cursor page size fits usize"),
    );
    println!(
        "  preflight: N={documents} fixture count and range/filter-sort/keyword-sort/sorted-page order verified"
    );
}

async fn scale_precompute_mid_collection_cursor(
    client: &reqwest::Client,
    base: &str,
    documents: usize,
) -> String {
    let page_size = SCALE_CURSOR_PAGE_SIZE as usize;
    let mid_start = (documents / 2 / page_size) * page_size;
    assert!(
        mid_start >= page_size && mid_start + 2 * page_size <= documents,
        "scale cursor preflight needs two full mid-collection pages for N={documents}"
    );

    let mut cursor = None;
    for page_start in (0..mid_start).step_by(page_size) {
        let page = scale_search_json(
            client,
            base,
            scale_lumen_query("sorted_page_deep", cursor.take()),
        )
        .await;
        assert_scale_page(&page, page_start, page_size);
        cursor = page["cursor"].as_str().map(str::to_owned);
        assert!(
            cursor.is_some(),
            "scale cursor preflight exhausted before mid-collection page {mid_start}"
        );
    }
    let mid_cursor = cursor.expect("scale precomputed mid-collection cursor");

    let first = scale_search_json(
        client,
        base,
        scale_lumen_query("sorted_page_deep", Some(mid_cursor.clone())),
    )
    .await;
    assert_scale_page(&first, mid_start, page_size);
    let next_cursor = first["cursor"]
        .as_str()
        .expect("first mid-collection cursor page has continuation")
        .to_string();
    let second = scale_search_json(
        client,
        base,
        scale_lumen_query("sorted_page_deep", Some(next_cursor)),
    )
    .await;
    assert_scale_page(&second, mid_start + page_size, page_size);
    let first_ids: std::collections::BTreeSet<_> = first["hits"]
        .as_array()
        .expect("first scale cursor hits")
        .iter()
        .map(|hit| {
            hit["external_id"]
                .as_str()
                .expect("first cursor external_id")
        })
        .collect();
    assert!(
        second["hits"]
            .as_array()
            .expect("second scale cursor hits")
            .iter()
            .all(|hit| !first_ids.contains(
                hit["external_id"]
                    .as_str()
                    .expect("second cursor external_id")
            )),
        "scale cursor preflight pages overlap"
    );
    mid_cursor
}

async fn scale_assert_ids(
    client: &reqwest::Client,
    base: &str,
    ids: &[String],
    expected: &[String],
) {
    let response = scale_search_json(
        client,
        base,
        json!({"query":{"ids":{"values":ids}},"limit":ids.len()}),
    )
    .await;
    let actual: std::collections::BTreeSet<_> = response["hits"]
        .as_array()
        .expect("ids query hits")
        .iter()
        .map(|hit| {
            hit["external_id"]
                .as_str()
                .expect("ids query external_id")
                .to_string()
        })
        .collect();
    let expected: std::collections::BTreeSet<_> = expected.iter().cloned().collect();
    assert_eq!(actual, expected, "ids query visibility changed");
}

struct ScaleMutationMeasurement {
    batch_unindex: Duration,
    post_reindex_keyword_sort: Duration,
    post_reindex_keyword_sort_took_us: u64,
    truncate: Duration,
    batch_ids: usize,
    readyz_status: reqwest::StatusCode,
    reclaimer_baseline: lumen::storage::CollectionReclaimerSnapshot,
    reclaimer_after_truncate: lumen::storage::CollectionReclaimerSnapshot,
    reclaimer_drained: lumen::storage::CollectionReclaimerSnapshot,
    reclaimer_drain: Duration,
}

async fn wait_for_scale_reclaimer_drain(
    baseline: lumen::storage::CollectionReclaimerSnapshot,
    truncate_started: Instant,
) -> lumen::storage::CollectionReclaimerSnapshot {
    let submitted_for_this_run = baseline.submitted_generations + 1;
    let completed_for_this_run = baseline.completed_generations + 1;
    let deadline = Instant::now() + SCALE_RECLAIMER_DRAIN_TIMEOUT;

    loop {
        let snapshot = lumen::storage::collection_reclaimer_snapshot();
        if snapshot.submitted_generations >= submitted_for_this_run
            && snapshot.completed_generations >= completed_for_this_run
            && snapshot.pending_generations == baseline.pending_generations
        {
            return snapshot;
        }
        assert!(
            Instant::now() < deadline,
            "scale truncate reclaimer did not drain this run's generation within {:?}: baseline={baseline:?}; latest={snapshot:?}; elapsed={:?}",
            SCALE_RECLAIMER_DRAIN_TIMEOUT,
            truncate_started.elapsed(),
        );
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}

async fn measure_scale_mutations(
    client: &reqwest::Client,
    base: &str,
    documents: usize,
) -> ScaleMutationMeasurement {
    let batch_ids: Vec<String> = (0..documents.min(1_000))
        .map(|document| format!("d{document}"))
        .collect();
    let survivor = "scale-unindex-survivor".to_string();

    // At N=1,000 the required batch removes every measured document. Add one
    // temporary survivor after the read matrix so the mutation probe can still
    // prove an unaffected document without changing the recorded scale N.
    let sentinel_items = scale_sentinel_items();
    post_index(client, base, &sentinel_items).await;
    assert_eq!(
        scale_documents_indexed(client, base).await,
        documents as u64 + 1,
        "scale mutation probe must add only its temporary survivor"
    );

    let started = Instant::now();
    let response = client
        .post(format!("{base}/collections/docs/docs:unindex"))
        .json(&json!({"external_ids":&batch_ids}))
        .send()
        .await
        .expect("batch unindex request")
        .error_for_status()
        .expect("batch unindex success status");
    assert_eq!(
        response.status(),
        reqwest::StatusCode::NO_CONTENT,
        "batch unindex must return its atomic logical acknowledgement"
    );
    let batch_unindex = started.elapsed();

    scale_assert_ids(client, base, &batch_ids, &[]).await;
    scale_assert_ids(
        client,
        base,
        std::slice::from_ref(&survivor),
        std::slice::from_ref(&survivor),
    )
    .await;
    assert_eq!(
        scale_documents_indexed(client, base).await,
        documents as u64 + 1 - batch_ids.len() as u64,
        "batch unindex must make every selected id invisible together"
    );

    scale_reindex_documents(client, base, batch_ids.iter().map(|id| doc_ordinal(id))).await;
    assert_eq!(
        scale_documents_indexed(client, base).await,
        documents as u64 + 1,
        "reindex must restore every batch-unindexed document"
    );

    client
        .post(format!("{base}/collections/docs/docs:unindex"))
        .json(&json!({"external_ids":[survivor]}))
        .send()
        .await
        .expect("temporary survivor cleanup request")
        .error_for_status()
        .expect("temporary survivor cleanup status");
    assert_eq!(
        scale_documents_indexed(client, base).await,
        documents as u64,
        "temporary survivor cleanup must restore the exact scale document count"
    );

    // Both reindex and survivor cleanup invalidate the response cache. This
    // first post-write request is intentionally measured without a threshold:
    // it proves the sealed dictionary plus live-tail merge keeps the sparse
    // keyword page and exact total correct after the write path.
    let started = Instant::now();
    let post_reindex_keyword_response =
        scale_search_json(client, base, scale_lumen_query("keyword_sort", None)).await;
    let post_reindex_keyword_sort = started.elapsed();
    let expected_keyword_sort: Vec<String> = (0..documents)
        .filter(|document| scale_sort_key(*document).is_some())
        .take(80)
        .map(|document| format!("d{document}"))
        .collect();
    assert_eq!(
        scale_response_ids(&post_reindex_keyword_response, "keyword_sort post-reindex"),
        expected_keyword_sort,
        "post-reindex cold keyword_sort changed sparse missing:last order"
    );
    assert_eq!(
        post_reindex_keyword_response["total"].as_u64(),
        Some(documents as u64),
        "post-reindex cold keyword_sort lost its exact total"
    );
    let post_reindex_keyword_sort_took_us = post_reindex_keyword_response["took_us"]
        .as_u64()
        .unwrap_or(0);

    let reclaimer_baseline = lumen::storage::collection_reclaimer_snapshot();
    let started = Instant::now();
    let response = client
        .post(format!("{base}/collections/docs/docs:truncate"))
        .send()
        .await
        .expect("truncate request")
        .error_for_status()
        .expect("truncate success status");
    assert_eq!(
        response.status(),
        reqwest::StatusCode::NO_CONTENT,
        "truncate must return its logical acknowledgement"
    );
    let truncate = started.elapsed();
    let reclaimer_after_truncate = lumen::storage::collection_reclaimer_snapshot();
    assert_eq!(
        scale_documents_indexed(client, base).await,
        0,
        "truncate must make its fresh collection visible before the reclaimer drains"
    );
    let readyz_status = client
        .get(format!("{base}/readyz"))
        .send()
        .await
        .expect("readyz after truncate request")
        .status();
    assert_eq!(
        readyz_status,
        reqwest::StatusCode::OK,
        "truncate must not move the scale engine into drain"
    );
    let reclaimer_drained = wait_for_scale_reclaimer_drain(reclaimer_baseline, started).await;

    ScaleMutationMeasurement {
        batch_unindex,
        post_reindex_keyword_sort,
        post_reindex_keyword_sort_took_us,
        truncate,
        batch_ids: batch_ids.len(),
        readyz_status,
        reclaimer_baseline,
        reclaimer_after_truncate,
        reclaimer_drained,
        reclaimer_drain: started.elapsed(),
    }
}

/// Build a DISK-backed lumen for the scale bench: a fresh Engine, the `docs`
/// schema, the corpus indexed DIRECTLY (fast path), then `flush_to_segments`
/// IN PLACE (drivers dropped → segment-backed), then the SAME engine wrapped in
/// the axum server (the exact `lumen::api::router` builder `lumen_serve_engine`
/// uses) so `measure_lumen`/`run_load` can drive it over HTTP. Returns the
/// client/base, the engine handle, the segment `TempDir` (kept alive so the
/// mmaps stay mapped), and the flush sequence used.
async fn scale_serve_disk(
    n: usize,
    seq: u64,
) -> (
    reqwest::Client,
    String,
    Arc<lumen::storage::Engine>,
    tempfile::TempDir,
) {
    let engine = Arc::new(lumen::storage::Engine::new());
    create_scale_collection(&engine, "docs");

    // Fast in-process indexing, then seal to disk (drivers dropped).
    scale_index_direct(&engine, n);
    let dir = tempfile::tempdir().unwrap();
    engine
        .flush_to_segments(dir.path(), seq)
        .expect("flush_to_segments (scale disk tier)");

    // Same engine, same router builder lumen_serve_engine uses, over HTTP.
    let app = lumen::api::router(lumen::api::AppState::open(engine.clone()));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    let base = format!("http://{addr}");
    let client = reqwest::Client::new();
    for _ in 0..50 {
        if client.get(format!("{base}/healthz")).send().await.is_ok() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    (client, base, engine, dir)
}

async fn scale_serve_inram(n: usize) -> (reqwest::Client, String, Arc<lumen::storage::Engine>) {
    let engine = Arc::new(lumen::storage::Engine::new());
    create_scale_collection(&engine, "docs");
    scale_index_direct(&engine, n);

    let app = lumen::api::router(lumen::api::AppState::open(engine.clone()));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    let base = format!("http://{addr}");
    let client = reqwest::Client::new();
    for _ in 0..50 {
        if client.get(format!("{base}/healthz")).send().await.is_ok() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    (client, base, engine)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn scale_mutation_probe_waits_for_reclaimer_drain() {
    // This is a focused harness gate, not a scale-matrix row. A small corpus
    // exercises the same HTTP batch-unindex and truncate path without running
    // the DEV or full read/QPS matrix.
    let (client, base, _engine, _dir) = scale_serve_disk(1_000, 91).await;
    let measurement = measure_scale_mutations(&client, &base, 1_000).await;

    assert_eq!(measurement.batch_ids, 1_000);
    assert!(
        measurement.reclaimer_after_truncate.submitted_generations
            >= measurement.reclaimer_baseline.submitted_generations + 1
    );
    assert!(
        measurement.reclaimer_drained.completed_generations
            >= measurement.reclaimer_baseline.completed_generations + 1
    );
    assert_eq!(
        measurement.reclaimer_drained.pending_generations,
        measurement.reclaimer_baseline.pending_generations
    );
    assert!(
        measurement.reclaimer_drain <= SCALE_RECLAIMER_DRAIN_TIMEOUT,
        "focused scale mutation probe exceeded its reclaimer drain timeout"
    );
}

struct ScaleShard {
    engine: Arc<lumen::storage::Engine>,
    dir: std::path::PathBuf,
}

struct ShardedScale {
    shards: Vec<ScaleShard>,
    _roots: Vec<tempfile::TempDir>,
    peak_rss: u64,
}

struct StorageOnlyChunks {
    root: tempfile::TempDir,
    chunks: usize,
    on_disk_bytes: u64,
    by_field: std::collections::BTreeMap<String, u64>,
    peak_rss: u64,
}

impl ShardedScale {
    fn segment_bytes(&self) -> (u64, std::collections::BTreeMap<String, u64>) {
        let mut total = 0u64;
        let mut by_field = std::collections::BTreeMap::new();
        for shard in &self.shards {
            let (bytes, fields) = scale_segment_bytes(&shard.dir);
            total += bytes;
            for (field, field_bytes) in fields {
                *by_field.entry(field).or_insert(0) += field_bytes;
            }
        }
        (total, by_field)
    }

    fn search(&self, req: lumen::types::SearchRequest) -> lumen::types::SearchResponse {
        lumen::routing::search_shards_parallel(
            "docs",
            req,
            &self.shards,
            |shard, collection_id, req| Ok(shard.engine.search(collection_id, req)?),
            |hit, field| match field {
                "age" => Some(doc_age(&hit.external_id) as f64),
                // `doc_key` is a zero-padded rendering of the same document
                // ordinal, so the numeric surrogate preserves its deterministic
                // high-cardinality keyword order during shard merge.
                "doc_key" => Some(doc_ordinal(&hit.external_id) as f64),
                _ => None,
            },
        )
        .expect("sharded scale search")
    }
}

fn doc_ordinal(eid: &str) -> usize {
    eid.strip_prefix('d')
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(usize::MAX)
}

fn doc_age(eid: &str) -> i32 {
    let i = doc_ordinal(eid);
    18 + ((i as i64 * 7919) % 63) as i32
}

fn scale_serve_disk_sharded(n: usize, seq: u64, chunk_rows: usize) -> ShardedScale {
    assert!(chunk_rows > 0, "LUMEN_SCALE_CHUNK_ROWS must be > 0");
    let mut rng = Lcg::new(SEED);
    let mut shards = Vec::new();
    let mut roots = Vec::new();
    let mut peak_rss = scale_procmem::rss_bytes().unwrap_or(0);
    let mut start = 0usize;
    let mut shard_id = 0usize;
    while start < n {
        let take = (n - start).min(chunk_rows);
        println!(
            "    chunk {shard_id}: indexing docs [{start}, {}) then flush_to_segments",
            start + take
        );
        let engine = Arc::new(lumen::storage::Engine::new());
        create_scale_collection(&engine, "docs");
        scale_index_direct_range(&engine, "docs", start, take, n, start, &mut rng);
        let dir = tempfile::tempdir().unwrap();
        engine
            .flush_to_segments(dir.path(), seq + shard_id as u64)
            .expect("flush_to_segments (scale chunk)");
        assert_segment_backed(&engine);
        peak_rss = peak_rss.max(scale_procmem::rss_bytes().unwrap_or(0));
        let shard_dir = dir.path().to_path_buf();
        shards.push(ScaleShard {
            engine,
            dir: shard_dir,
        });
        roots.push(dir);
        start += take;
        shard_id += 1;
    }
    ShardedScale {
        shards,
        _roots: roots,
        peak_rss,
    }
}

fn scale_serve_disk_sharded_reopened(n: usize, seq: u64, chunk_rows: usize) -> ShardedScale {
    assert!(chunk_rows > 0, "LUMEN_SCALE_CHUNK_ROWS must be > 0");
    let mut rng = Lcg::new(SEED);
    let mut shards = Vec::new();
    let mut roots = Vec::new();
    let mut peak_rss = scale_procmem::rss_bytes().unwrap_or(0);
    let mut start = 0usize;
    let mut shard_id = 0usize;
    while start < n {
        let take = (n - start).min(chunk_rows);
        println!(
            "    chunk {shard_id}: indexing docs [{start}, {}) then flush_to_segments + reopen shard",
            start + take
        );
        let build_engine = Arc::new(lumen::storage::Engine::new());
        create_scale_collection(&build_engine, "docs");
        scale_index_direct_range(&build_engine, "docs", start, take, n, start, &mut rng);
        let dir = tempfile::tempdir().unwrap();
        build_engine
            .flush_to_segments(dir.path(), seq + shard_id as u64)
            .expect("flush_to_segments (scale reopened chunk)");
        assert_segment_backed(&build_engine);
        peak_rss = peak_rss.max(scale_procmem::rss_bytes().unwrap_or(0));
        drop(build_engine);

        let engine = Arc::new(lumen::storage::Engine::new());
        let reopened_seq = engine
            .reopen_from_segment_dir(dir.path())
            .expect("reopen scale chunk segment dir");
        assert_eq!(reopened_seq, seq + shard_id as u64);
        assert_segment_backed(&engine);
        peak_rss = peak_rss.max(scale_procmem::rss_bytes().unwrap_or(0));
        let shard_dir = dir.path().to_path_buf();
        shards.push(ScaleShard {
            engine,
            dir: shard_dir,
        });
        roots.push(dir);
        start += take;
        shard_id += 1;
    }
    ShardedScale {
        shards,
        _roots: roots,
        peak_rss,
    }
}

async fn sharded_scale_healthz() -> &'static str {
    "ok"
}

async fn sharded_scale_search(
    axum::extract::State(scale): axum::extract::State<Arc<ShardedScale>>,
    axum::extract::Path(collection_id): axum::extract::Path<String>,
    axum::Json(req): axum::Json<lumen::types::SearchRequest>,
) -> Result<axum::Json<lumen::types::SearchResponse>, (axum::http::StatusCode, String)> {
    if collection_id != "docs" {
        return Err((
            axum::http::StatusCode::NOT_FOUND,
            format!("unknown collection {collection_id}"),
        ));
    }
    Ok(axum::Json(scale.search(req)))
}

async fn scale_serve_disk_sharded_http(
    n: usize,
    seq: u64,
    chunk_rows: usize,
) -> (reqwest::Client, String, Arc<ShardedScale>) {
    let scale = Arc::new(scale_serve_disk_sharded(n, seq, chunk_rows));
    let app = axum::Router::new()
        .route("/healthz", axum::routing::get(sharded_scale_healthz))
        .route(
            "/collections/{collection_id}/search",
            axum::routing::post(sharded_scale_search),
        )
        .with_state(scale.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    let base = format!("http://{addr}");
    let client = reqwest::Client::new();
    for _ in 0..50 {
        if client.get(format!("{base}/healthz")).send().await.is_ok() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    (client, base, scale)
}

async fn scale_serve_disk_sharded_reopened_http(
    n: usize,
    seq: u64,
    chunk_rows: usize,
) -> (reqwest::Client, String, Arc<ShardedScale>) {
    let scale = Arc::new(scale_serve_disk_sharded_reopened(n, seq, chunk_rows));
    let app = axum::Router::new()
        .route("/healthz", axum::routing::get(sharded_scale_healthz))
        .route(
            "/collections/{collection_id}/search",
            axum::routing::post(sharded_scale_search),
        )
        .with_state(scale.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    let base = format!("http://{addr}");
    let client = reqwest::Client::new();
    for _ in 0..50 {
        if client.get(format!("{base}/healthz")).send().await.is_ok() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    (client, base, scale)
}

async fn scale_serve_disk_sharded_reopened_parallel_http(
    n: usize,
    seq: u64,
    chunk_rows: usize,
    workers: usize,
) -> (reqwest::Client, String, Arc<ShardedScale>) {
    let scale = Arc::new(scale_serve_disk_sharded_reopened_parallel(
        n, seq, chunk_rows, workers,
    ));
    let app = axum::Router::new()
        .route("/healthz", axum::routing::get(sharded_scale_healthz))
        .route(
            "/collections/{collection_id}/search",
            axum::routing::post(sharded_scale_search),
        )
        .with_state(scale.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    let base = format!("http://{addr}");
    let client = reqwest::Client::new();
    for _ in 0..50 {
        if client.get(format!("{base}/healthz")).send().await.is_ok() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    (client, base, scale)
}

fn scale_build_disk_chunks_storage_only(
    n: usize,
    seq: u64,
    chunk_rows: usize,
    workers: usize,
) -> StorageOnlyChunks {
    assert!(chunk_rows > 0, "LUMEN_SCALE_CHUNK_ROWS must be > 0");
    let root = tempfile::tempdir().unwrap();
    let total_chunks = n.div_ceil(chunk_rows);
    let workers = workers.max(1).min(total_chunks.max(1));
    let mut on_disk_bytes = 0u64;
    let mut by_field = std::collections::BTreeMap::new();
    let mut peak_rss = scale_procmem::rss_bytes().unwrap_or(0);
    let next_chunk = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let root_path = root.path().to_path_buf();

    std::thread::scope(|scope| {
        let mut handles = Vec::with_capacity(workers);
        for worker_id in 0..workers {
            let next_chunk = next_chunk.clone();
            let root_path = root_path.clone();
            handles.push(scope.spawn(move || {
                let mut worker_bytes = 0u64;
                let mut worker_fields = std::collections::BTreeMap::new();
                let mut worker_peak = scale_procmem::rss_bytes().unwrap_or(0);
                let mut worker_chunks = 0usize;
                loop {
                    let shard_id = next_chunk.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    if shard_id >= total_chunks {
                        break;
                    }
                    let start = shard_id * chunk_rows;
                    let take = (n - start).min(chunk_rows);
                    println!(
                        "    worker {worker_id} chunk {shard_id}: indexing docs [{start}, {}) then flush_to_segments + drop Engine",
                        start + take
                    );
                    let engine = Arc::new(lumen::storage::Engine::new());
                    create_scale_collection(&engine, "docs");
                    let mut rng = Lcg::for_doc(start);
                    scale_index_direct_range(&engine, "docs", start, take, n, start, &mut rng);
                    worker_peak = worker_peak.max(scale_procmem::rss_bytes().unwrap_or(0));

                    let chunk_dir = root_path.join(format!("chunk-{shard_id:06}"));
                    std::fs::create_dir_all(&chunk_dir).expect("create chunk segment dir");
                    engine
                        .flush_to_segments(&chunk_dir, seq + shard_id as u64)
                        .expect("flush_to_segments (storage-only chunk)");
                    assert_segment_backed(&engine);

                    let (bytes, fields) = scale_segment_bytes(&chunk_dir);
                    assert!(bytes > 0, "chunk {shard_id}: no segment bytes written");
                    worker_bytes += bytes;
                    for (field, field_bytes) in fields {
                        *worker_fields.entry(field).or_insert(0) += field_bytes;
                    }

                    drop(engine);
                    worker_peak = worker_peak.max(scale_procmem::rss_bytes().unwrap_or(0));
                    worker_chunks += 1;
                }
                (worker_bytes, worker_fields, worker_peak, worker_chunks)
            }));
        }

        for handle in handles {
            let (bytes, fields, worker_peak, worker_chunks) =
                handle.join().expect("storage-only chunk worker panicked");
            on_disk_bytes += bytes;
            for (field, field_bytes) in fields {
                *by_field.entry(field).or_insert(0) += field_bytes;
            }
            peak_rss = peak_rss.max(worker_peak);
            debug_assert!(worker_chunks <= total_chunks);
        }
    });

    StorageOnlyChunks {
        root,
        chunks: total_chunks,
        on_disk_bytes,
        by_field,
        peak_rss,
    }
}

fn scale_open_storage_only_chunks(
    chunks: StorageOnlyChunks,
    seq: u64,
    workers: usize,
) -> ShardedScale {
    let total_chunks = chunks.chunks;
    let workers = workers.max(1).min(total_chunks.max(1));
    let root_path = chunks.root.path().to_path_buf();
    let mut peak_rss = chunks.peak_rss.max(scale_procmem::rss_bytes().unwrap_or(0));
    let next_chunk = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let mut shard_slots: Vec<Option<ScaleShard>> = Vec::with_capacity(total_chunks);
    shard_slots.resize_with(total_chunks, || None);

    println!("    reopening {total_chunks} sealed chunks with {workers} workers");
    std::thread::scope(|scope| {
        let mut handles = Vec::with_capacity(workers);
        for worker_id in 0..workers {
            let next_chunk = next_chunk.clone();
            let root_path = root_path.clone();
            handles.push(scope.spawn(move || {
                let mut worker_shards = Vec::new();
                let mut worker_peak = scale_procmem::rss_bytes().unwrap_or(0);
                loop {
                    let shard_id = next_chunk.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    if shard_id >= total_chunks {
                        break;
                    }
                    let started = Instant::now();
                    println!(
                        "    worker {worker_id} reopen chunk {shard_id}/{total_chunks}: start"
                    );
                    let dir = root_path.join(format!("chunk-{shard_id:06}"));
                    let engine = Arc::new(lumen::storage::Engine::new());
                    let reopened_seq = engine
                        .reopen_from_segment_dir(&dir)
                        .expect("reopen parallel-built scale chunk segment dir");
                    assert_eq!(reopened_seq, seq + shard_id as u64);
                    assert_segment_backed(&engine);
                    worker_peak = worker_peak.max(scale_procmem::rss_bytes().unwrap_or(0));
                    println!(
                        "    worker {worker_id} reopen chunk {shard_id}/{total_chunks}: done {:.2}s",
                        started.elapsed().as_secs_f64()
                    );
                    worker_shards.push((shard_id, ScaleShard { engine, dir }));
                }
                (worker_shards, worker_peak)
            }));
        }

        for handle in handles {
            let (worker_shards, worker_peak) = handle.join().expect("reopen chunk worker panicked");
            peak_rss = peak_rss.max(worker_peak);
            for (shard_id, shard) in worker_shards {
                shard_slots[shard_id] = Some(shard);
            }
        }
    });

    let shards = shard_slots
        .into_iter()
        .enumerate()
        .map(|(shard_id, shard)| {
            shard.unwrap_or_else(|| panic!("missing reopened shard {shard_id}"))
        })
        .collect();
    ShardedScale {
        shards,
        _roots: vec![chunks.root],
        peak_rss,
    }
}

fn scale_serve_disk_sharded_reopened_parallel(
    n: usize,
    seq: u64,
    chunk_rows: usize,
    workers: usize,
) -> ShardedScale {
    let chunks = scale_build_disk_chunks_storage_only(n, seq, chunk_rows, workers);
    scale_open_storage_only_chunks(chunks, seq, workers)
}

fn measure_scale_lumen_sharded(scale: &ShardedScale, cell: &str) -> Stat {
    let req: lumen::types::SearchRequest =
        serde_json::from_value(scale_lumen_query(cell, None)).expect("parse scale query");
    let mut e2e = Vec::with_capacity(REPS);
    let mut engine = Vec::with_capacity(REPS);
    for i in 0..(WARMUP + REPS) {
        let t0 = Instant::now();
        let resp = scale.search(req.clone());
        let elapsed = t0.elapsed().as_secs_f64() * 1000.0;
        if i >= WARMUP {
            e2e.push(elapsed);
            engine.push(resp.took_us as f64 / 1000.0);
        }
    }
    summarize(e2e, engine)
}

/// Per-N storage facts captured once for the whole collection.
struct ScaleStorage {
    on_disk_bytes: u64,
    by_field: std::collections::BTreeMap<String, u64>,
    peak_rss: u64,
    rss_available: bool,
}

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
#[ignore = "LUMEN-ONLY disk scale bench (no pg/OS) — report, run --ignored --nocapture lumen_scale_bench"]
async fn lumen_scale_bench() {
    // Row ladder: env LUMEN_SCALE_ROWS (comma-separated). DEFAULT stays within
    // the project-standard 100k local benchmark scope.
    let rows: Vec<usize> = std::env::var("LUMEN_SCALE_ROWS")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .map(|s| {
            s.split(',')
                .filter_map(|t| t.trim().parse::<usize>().ok())
                .collect()
        })
        .unwrap_or_else(|| DEFAULT_SCALE_ROWS.to_vec());
    let standard_max_rows = std::env::var("LUMEN_SCALE_MAX_ROWS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(DEFAULT_SCALE_MAX_ROWS);
    let allow_above_standard = env_flag_enabled("LUMEN_SCALE_ALLOW_ABOVE_STANDARD")
        || env_flag_enabled("LUMEN_SCALE_ALLOW_ABOVE_1M");
    if !allow_above_standard {
        if let Some(&too_large) = rows.iter().find(|&&n| n > standard_max_rows) {
            panic!(
                "LUMEN_SCALE_ROWS contains {too_large}, above the standard local benchmark cap {standard_max_rows}. \
                 The lumen readiness default stops at 100k docs; run larger rows only as an explicit \
                 release-soak/research experiment with LUMEN_SCALE_ALLOW_ABOVE_STANDARD=1."
            );
        }
    }
    let max_inmem_rows = std::env::var("LUMEN_SCALE_MAX_INMEM_ROWS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(10_000_000);
    let allow_high_rss = std::env::var("LUMEN_SCALE_ALLOW_HIGH_RSS")
        .ok()
        .map(|s| s == "1" || s.eq_ignore_ascii_case("true") || s.eq_ignore_ascii_case("yes"))
        .unwrap_or(false);
    // LUMEN_SCALE_DISK=1 (default on) forces the segment/disk path. The bench is
    // disk-only by construction; the flag is honored for parity with the spec and
    // can be set to 0 to bypass the flush (in-RAM driver path) for A/B.
    let disk = std::env::var("LUMEN_SCALE_DISK")
        .ok()
        .map(|s| s != "0" && !s.eq_ignore_ascii_case("false"))
        .unwrap_or(true);
    let run_qps = std::env::var("LUMEN_SCALE_QPS")
        .ok()
        .map(|s| s != "0" && !s.eq_ignore_ascii_case("false"))
        .unwrap_or(true);
    let chunk_rows = std::env::var("LUMEN_SCALE_CHUNK_ROWS")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .and_then(|s| s.parse::<usize>().ok());
    let storage_only = std::env::var("LUMEN_SCALE_STORAGE_ONLY")
        .ok()
        .map(|s| s == "1" || s.eq_ignore_ascii_case("true") || s.eq_ignore_ascii_case("yes"))
        .unwrap_or(false);
    let chunk_workers = std::env::var("LUMEN_SCALE_CHUNK_WORKERS")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(1)
        .max(1);
    let reopen_shards = std::env::var("LUMEN_SCALE_REOPEN_SHARDS")
        .ok()
        .map(|s| s != "0" && !s.eq_ignore_ascii_case("false") && !s.eq_ignore_ascii_case("no"))
        .unwrap_or(true);
    let selected_cells = parse_scale_cells();
    let qps_targets = parse_scale_qps_targets();
    if let Some(0) = chunk_rows {
        panic!("LUMEN_SCALE_CHUNK_ROWS must be > 0");
    }
    if chunk_rows.is_some() && !disk {
        panic!("LUMEN_SCALE_CHUNK_ROWS requires LUMEN_SCALE_DISK=1");
    }
    if storage_only && (!disk || chunk_rows.is_none() || run_qps) {
        panic!(
            "LUMEN_SCALE_STORAGE_ONLY=1 requires LUMEN_SCALE_DISK=1, LUMEN_SCALE_CHUNK_ROWS=<n>, and LUMEN_SCALE_QPS=0"
        );
    }
    if chunk_workers > 1 && !(storage_only || (chunk_rows.is_some() && run_qps && reopen_shards)) {
        panic!(
            "LUMEN_SCALE_CHUNK_WORKERS>1 currently requires LUMEN_SCALE_STORAGE_ONLY=1 or chunked reopened qps"
        );
    }
    if !allow_high_rss {
        if let Some(chunk) = chunk_rows {
            if chunk > max_inmem_rows {
                panic!(
                    "LUMEN_SCALE_CHUNK_ROWS={chunk} is above the in-memory-build guard {max_inmem_rows}. \
                     Use a smaller chunk, or explicitly opt into a research run with enough RAM/swap."
                );
            }
        }
        if let Some(&too_large) = rows.iter().find(|&&n| n > max_inmem_rows) {
            if !(disk && chunk_rows.is_some()) {
                panic!(
                    "LUMEN_SCALE_ROWS contains {too_large}, above the default in-memory-build guard {max_inmem_rows}. \
                     The scale bench stream-generates docs, but the single-segment path still holds the mutable index \
                     until flush_to_segments. Keep the standard benchmark at 100k, or set LUMEN_SCALE_CHUNK_ROWS=<rows_per_chunk> \
                     only for an explicit release-soak/research run."
                );
            }
        }
    }
    let sharded_qps_max_rows = std::env::var("LUMEN_SCALE_SHARDED_QPS_MAX_ROWS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(20_000_000);
    if chunk_rows.is_some() && run_qps && !storage_only && !reopen_shards && !allow_high_rss {
        if let Some(&too_large) = rows.iter().find(|&&n| n > sharded_qps_max_rows) {
            panic!(
                "LUMEN_SCALE_ROWS contains {too_large}, above the default sharded qps guard {sharded_qps_max_rows}. \
                 LUMEN_SCALE_REOPEN_SHARDS=0 keeps all chunk Engines live behind the test HTTP router. \
                 Leave LUMEN_SCALE_REOPEN_SHARDS at its default or lower the row count."
            );
        }
    }

    println!("\n########################################################################");
    println!("# LUMEN-ONLY DISK SCALE BENCH  (NO Postgres, NO OpenSearch)");
    println!("#   row ladder : {rows:?}");
    println!(
        "#   disk path  : {}",
        if disk {
            "ON (flush_to_segments → segment-backed)"
        } else {
            "OFF (in-RAM drivers)"
        }
    );
    println!(
        "#   chunking   : {}",
        chunk_rows
            .map(|n| {
                if storage_only {
                    format!(
                        "ON ({n} rows/chunk, storage-only; Engines dropped; workers={chunk_workers})"
                    )
                } else if run_qps {
                    if reopen_shards {
                        format!("ON ({n} rows/chunk, reopened sharded HTTP qps + direct merge latency; workers={chunk_workers})")
                    } else {
                        format!("ON ({n} rows/chunk, live-engine sharded HTTP qps + direct merge latency)")
                    }
                } else {
                    format!("ON ({n} rows/chunk, sharded direct merge)")
                }
            })
            .unwrap_or_else(|| "OFF (single segment build)".to_string())
    );
    println!(
        "#   per cell   : fixed read matrix (range, boolean, number sort, high-cardinality keyword sort, cursor pagination){}",
        if run_qps {
            " + paced read-only qps ladder"
        } else {
            " (qps ladder skipped)"
        }
    );
    println!("#   cells      : {:?}", selected_cells);
    println!(
        "#   qps targets: {}",
        if run_qps {
            format!("{qps_targets:?}")
        } else {
            "skipped".to_string()
        }
    );
    println!("#   per N      : on-disk index MiB, bytes/doc (+per-field), peak RSS, RSS/disk");
    println!("########################################################################");

    for &n in &rows {
        let row_started = Instant::now();
        println!("\n========================================================================");
        println!("N = {n} rows");
        println!("========================================================================");

        // RSS baseline before we build/serve this N (so peak isolates THIS N's work).
        let rss_available = scale_procmem::rss_bytes().is_some();
        let mut peak_rss = scale_procmem::rss_bytes().unwrap_or(0);

        println!(
            "  streaming + indexing {n} docs DIRECTLY (in-process), then {} ...",
            if disk {
                "flush_to_segments (disk path)"
            } else {
                "serving in-RAM"
            }
        );

        // Build + index + (optionally) seal. Chunked mode keeps the build RSS
        // bounded by sealing one shard at a time and dropping each shard's mutable
        // drivers before indexing the next shard.
        let mut client: Option<reqwest::Client> = None;
        let mut base: Option<String> = None;
        let mut engine: Option<Arc<lumen::storage::Engine>> = None;
        let mut dir: Option<tempfile::TempDir> = None;
        let mut sharded: Option<Arc<ShardedScale>> = None;
        let mut storage_only_chunks: Option<StorageOnlyChunks> = None;
        if storage_only {
            let chunk = chunk_rows.expect("storage-only requires chunk rows");
            storage_only_chunks = Some(scale_build_disk_chunks_storage_only(
                n,
                1,
                chunk,
                chunk_workers,
            ));
        } else if let Some(chunk) = chunk_rows.filter(|_| disk) {
            if run_qps {
                let (c, b, scale) = if reopen_shards {
                    if chunk_workers > 1 {
                        scale_serve_disk_sharded_reopened_parallel_http(n, 1, chunk, chunk_workers)
                            .await
                    } else {
                        scale_serve_disk_sharded_reopened_http(n, 1, chunk).await
                    }
                } else {
                    scale_serve_disk_sharded_http(n, 1, chunk).await
                };
                client = Some(c);
                base = Some(b);
                sharded = Some(scale);
            } else {
                sharded = Some(Arc::new(scale_serve_disk_sharded(n, 1, chunk)));
            }
        } else if disk {
            let (c, b, e, d) = scale_serve_disk(n, 1).await;
            client = Some(c);
            base = Some(b);
            engine = Some(e);
            dir = Some(d);
        } else {
            // In-RAM A/B fallback: same streaming direct index path, no flush.
            let (c, b, e) = scale_serve_inram(n).await;
            client = Some(c);
            base = Some(b);
            engine = Some(e);
            // No flush — keep a throwaway empty tempdir so cleanup shape matches.
            dir = Some(tempfile::tempdir().unwrap());
        }
        peak_rss = peak_rss.max(scale_procmem::rss_bytes().unwrap_or(0));
        if let Some(scale) = &sharded {
            peak_rss = peak_rss.max(scale.peak_rss);
        }
        if let Some(chunks) = &storage_only_chunks {
            peak_rss = peak_rss.max(chunks.peak_rss);
        }

        // On-disk segment accounting (only meaningful on the disk path).
        let (on_disk_bytes, by_field) = if let Some(chunks) = &storage_only_chunks {
            (chunks.on_disk_bytes, chunks.by_field.clone())
        } else if let Some(scale) = &sharded {
            scale.segment_bytes()
        } else if disk {
            scale_segment_bytes(dir.as_ref().unwrap().path())
        } else {
            (0, std::collections::BTreeMap::new())
        };
        if disk {
            assert!(
                on_disk_bytes > 0,
                "N={n}: no on-disk segment bytes were written"
            );
        }

        // Fail before any latency/QPS measurement when the normal single-engine
        // fixture or deterministic result ordering has changed. Chunked research
        // routing has its own merge proof and is intentionally not treated as a
        // normal single-engine matrix row.
        if !storage_only && sharded.is_none() {
            if selected_cells.contains(&"keyword_sort") {
                scale_measure_keyword_sort_cold(
                    client
                        .as_ref()
                        .expect("cold keyword timing requires HTTP client"),
                    base.as_ref()
                        .expect("cold keyword timing requires HTTP base"),
                    n,
                )
                .await;
            }
            scale_preflight_fixture(
                client
                    .as_ref()
                    .expect("single-engine preflight requires HTTP client"),
                base.as_ref()
                    .expect("single-engine preflight requires HTTP base"),
                n,
            )
            .await;
        }

        // ---- per-cell latency (HTTP e2e_min + engine took_us) ------------------
        if !storage_only {
            for &cell in &selected_cells {
                println!(
                    "  latency start: N={n} cell={cell} elapsed={:.1}s",
                    row_started.elapsed().as_secs_f64()
                );
                let s = if let Some(scale) = &sharded {
                    measure_scale_lumen_sharded(scale.as_ref(), cell)
                } else {
                    measure_lumen_request(
                        client.as_ref().unwrap(),
                        base.as_ref().unwrap(),
                        scale_lumen_query(cell, None),
                    )
                    .await
                };
                peak_rss = peak_rss.max(scale_procmem::rss_bytes().unwrap_or(0));
                println!(
                    "  latency done : N={n} cell={cell} e2e_min={:.3}ms eng_min={:.4}ms elapsed={:.1}s",
                    s.e2e_min,
                    s.engine_min.unwrap_or(f64::NAN),
                    row_started.elapsed().as_secs_f64()
                );
            }
        }

        // Sanity: at least one substantive cell returned hits (results non-empty).
        if !storage_only && n >= 40 {
            let sanity_cell = selected_cells[0];
            let hits = if let Some(scale) = &sharded {
                let req: lumen::types::SearchRequest =
                    serde_json::from_value(scale_lumen_query(sanity_cell, None)).unwrap();
                scale.search(req).hits.len()
            } else {
                let url = format!("{}/collections/docs/search", base.as_ref().unwrap());
                let j: Value = client
                    .as_ref()
                    .unwrap()
                    .post(&url)
                    .json(&scale_lumen_query(sanity_cell, None))
                    .send()
                    .await
                    .unwrap()
                    .error_for_status()
                    .unwrap()
                    .json()
                    .await
                    .unwrap();
                j.get("hits")
                    .and_then(|h| h.as_array())
                    .map(|a| a.len())
                    .unwrap_or(0)
            };
            assert!(
                hits > 0,
                "N={n}: {sanity_cell} returned no hits — corpus/index broken"
            );
        }

        // ---- paced read-only qps ladder --------------------------------------
        // Each row retains its complete measurement so the final report remains
        // one row per (documents, read cell, target qps), including qps=10.
        let mut load_rows: Vec<(&str, usize, Load, &'static str)> = Vec::new();
        if run_qps {
            let client = client.as_ref().expect("qps path requires HTTP client");
            let base = base.as_ref().expect("qps path requires HTTP base");
            let mid_collection_cursor =
                scale_precompute_mid_collection_cursor(client, base, n).await;
            println!(
                "  cursor preflight: N={n} precomputed mid-collection cursor; ordered non-overlapping pages verified"
            );
            for &qps in &qps_targets {
                println!(
                    "  qps ladder start: N={n} target={qps} elapsed={:.1}s",
                    row_started.elapsed().as_secs_f64()
                );
                let ceiling_load = healthz_ceiling_load(client, base, qps).await;
                assert_eq!(
                    ceiling_load.errors, 0,
                    "N={n}: /healthz qps probe had {} request errors",
                    ceiling_load.errors
                );
                let ceiling = ceiling_load.achieved_qps;
                for &cell in &selected_cells {
                    println!(
                        "  qps start: N={n} target={qps} cell={cell} ceiling={ceiling:.0} elapsed={:.1}s",
                        row_started.elapsed().as_secs_f64()
                    );
                    let l = run_load(
                        Req::Http {
                            client: client.clone(),
                            url: format!("{base}/collections/docs/search"),
                            body: Some(http_json_body(scale_lumen_query(
                                cell,
                                (cell == "sorted_page_deep").then(|| mid_collection_cursor.clone()),
                            ))),
                        },
                        qps,
                    )
                    .await;
                    peak_rss = peak_rss.max(scale_procmem::rss_bytes().unwrap_or(0));
                    // This is a classification, not a threshold or gate: `ok`
                    // reached the requested rate, `HARN` could not drive even
                    // /healthz to that rate, and `SVR` leaves the remaining
                    // shortfall to the server/query path.
                    let state = if l.achieved_qps >= qps as f64 {
                        "ok"
                    } else if ceiling < qps as f64 {
                        "HARN"
                    } else {
                        "SVR"
                    };
                    println!(
                        "  qps done : N={n} target={qps} cell={cell} achieved={:.0} p50={:.3}ms p95={:.3}ms p99={:.3}ms errors={} state={state} elapsed={:.1}s",
                        l.achieved_qps,
                        l.p50,
                        l.p95,
                        l.p99,
                        l.errors,
                        row_started.elapsed().as_secs_f64()
                    );
                    load_rows.push((cell, qps, l, state));
                }
            }
        }

        let storage = ScaleStorage {
            on_disk_bytes,
            by_field,
            peak_rss,
            rss_available,
        };

        // -------------------- MATRIX REPORT for this N -------------------------
        if storage_only {
            println!("\n  --- N={n} latency/qps skipped: storage-only chunked footprint proof ---");
        } else {
            println!("\n  --- N={n} read qps matrix (lumen-only, disk-backed HTTP) ---");
            println!(
                "  {:>8} {:<30} {:>8} {:>12} {:>10} {:>10} {:>10} {:>8} {:>10} {:>5}",
                "documents",
                "cell",
                "target",
                "achieved_qps",
                "p50_ms",
                "p95_ms",
                "p99_ms",
                "errors",
                "error_rate",
                "state",
            );
            for (cell, qps, load, state) in &load_rows {
                println!(
                    "  {n:>8} {cell:<30} {qps:>8} {:>12.3} {:>10.3} {:>10.3} {:>10.3} {:>8} {:>10.4} {:>5}",
                    load.achieved_qps,
                    load.p50,
                    load.p95,
                    load.p99,
                    load.errors,
                    load.error_rate,
                    state,
                );
            }
            let request_errors: usize = load_rows.iter().map(|(_, _, load, _)| load.errors).sum();
            assert_eq!(
                request_errors, 0,
                "N={n}: read qps matrix had {request_errors} request errors"
            );
        }

        if storage_only {
            println!(
                "  mutations: skipped (storage-only chunked proof has no HTTP mutation surface)"
            );
        } else if sharded.is_some() {
            println!(
                "  mutations: unavailable for chunked research mode (the read-only shard harness has no single atomic cross-shard docs:unindex/docs:truncate surface)"
            );
        } else {
            let mutation = measure_scale_mutations(
                client.as_ref().expect("mutation path requires HTTP client"),
                base.as_ref().expect("mutation path requires HTTP base"),
                n,
            )
            .await;
            println!(
                "  mutations: N={n} batch_unindex_ids={} logical={:?}; keyword_sort_post_reindex_cold_e2e={:?} took_us={}; truncate logical={:?}; readyz_after_truncate={}; reclaimer_pending_baseline={}; reclaimer_submitted_baseline={}; reclaimer_completed_baseline={}; reclaimer_after_truncate_pending={}; reclaimer_after_truncate_queued={}; reclaimer_after_truncate_active={}; reclaimer_queue_high_water={}; reclaimer_drained_pending={}; reclaimer_drained_submitted={}; reclaimer_drained_completed={}; reclaimer_drain_from_truncate_start={:?}",
                mutation.batch_ids,
                mutation.batch_unindex,
                mutation.post_reindex_keyword_sort,
                mutation.post_reindex_keyword_sort_took_us,
                mutation.truncate,
                mutation.readyz_status,
                mutation.reclaimer_baseline.pending_generations,
                mutation.reclaimer_baseline.submitted_generations,
                mutation.reclaimer_baseline.completed_generations,
                mutation.reclaimer_after_truncate.pending_generations,
                mutation.reclaimer_after_truncate.queued_tasks,
                mutation.reclaimer_after_truncate.active_tasks,
                mutation.reclaimer_drained.queue_high_water,
                mutation.reclaimer_drained.pending_generations,
                mutation.reclaimer_drained.submitted_generations,
                mutation.reclaimer_drained.completed_generations,
                mutation.reclaimer_drain,
            );
        }

        // -------------------- per-N storage summary line -----------------------
        let bytes_per_doc = if n > 0 {
            storage.on_disk_bytes as f64 / n as f64
        } else {
            0.0
        };
        let rss_str = if storage.rss_available {
            format!("{:.1} MiB", scale_procmem::mib(storage.peak_rss))
        } else {
            "unavailable".to_string()
        };
        let rss_disk_ratio = if storage.on_disk_bytes > 0 && storage.rss_available {
            format!(
                "{:.2}x",
                storage.peak_rss as f64 / storage.on_disk_bytes as f64
            )
        } else {
            "-".to_string()
        };
        println!("\n  --- N={n} storage summary (lumen disk) ---");
        println!(
            "  rows={n}  on-disk index={:.2} MiB  bytes/doc={bytes_per_doc:.1}  peak RSS={rss_str}  RSS/on-disk={rss_disk_ratio}",
            scale_procmem::mib(storage.on_disk_bytes),
        );
        if let Some(chunks) = &storage_only_chunks {
            println!(
                "  storage-only chunks={}  root={}",
                chunks.chunks,
                chunks.root.path().display()
            );
        }
        if !storage.by_field.is_empty() {
            print!("  per-field on-disk:");
            for (f, b) in &storage.by_field {
                print!("  {f}={:.2}MiB", scale_procmem::mib(*b));
            }
            println!();
        }

        // Keep the engine + tempdir alive across all measurement above; drop now.
        drop(client);
        drop(engine);
        drop(dir);
        drop(sharded);
        drop(storage_only_chunks);
    }

    println!("\n########################################################################");
    println!("# LUMEN-ONLY SCALE BENCH COMPLETE — numbers above are lumen's disk tier.");
    println!(
        "#   (no pg / no OpenSearch; standard local cap is 100k docs; larger rows require explicit release-soak/research opt-in)"
    );
    println!("########################################################################");
}
// CODEGEN-END
