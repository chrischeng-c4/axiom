//! Search-latency benches on a 100k-document corpus.
//!
//! Three scenarios from the README §9 perf-regression suite:
//!   * `search/match_single`  — 1-token `match` on a `text` field
//!   * `search/match_and_3`   — 3-token AND `match` on the same
//!   * `search/term_keyword`  — `term` on a `keyword` field
//!
//! The corpus is built once outside the timed region. Each bench
//! iteration only runs the query and discards the response.

use std::collections::BTreeMap;

use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use lumen::storage::{Engine, MAX_INDEX_ITEMS};
use lumen::types::{
    Analyzer, CreateCollectionRequest, FieldSpec, FieldType, FieldValue, IndexItem, IndexRequest,
    MatchOp, MatchQuery, QueryNode, SearchRequest, TermQuery,
};

const N: usize = 100_000;

// --- Deterministic data generation ---------------------------------------

struct Lcg(u64);

impl Lcg {
    fn new(seed: u64) -> Self {
        Self(seed)
    }
    fn next_u32(&mut self) -> u32 {
        self.0 = self.0.wrapping_mul(1664525).wrapping_add(1013904223);
        (self.0 >> 16) as u32
    }
}

/// 64-word vocabulary. With 5–9 tokens per doc and 100k docs the
/// posting list per token lands around 7-15k entries — large enough to
/// stress the intersect in `MatchOp::And` without exploding into the
/// "every doc matches" pathological case.
const VOCAB: &[&str] = &[
    "rust",
    "engineer",
    "senior",
    "junior",
    "tokyo",
    "taipei",
    "hsinchu",
    "designer",
    "backend",
    "frontend",
    "database",
    "kubernetes",
    "search",
    "index",
    "vector",
    "neural",
    "machine",
    "learning",
    "system",
    "service",
    "infrastructure",
    "platform",
    "team",
    "lead",
    "manager",
    "developer",
    "python",
    "javascript",
    "typescript",
    "golang",
    "tokio",
    "axum",
    "tower",
    "cargo",
    "borrow",
    "lifetime",
    "trait",
    "macro",
    "async",
    "future",
    "stream",
    "channel",
    "mutex",
    "rwlock",
    "arc",
    "raft",
    "consensus",
    "leader",
    "follower",
    "term",
    "log",
    "snapshot",
    "compaction",
    "memtable",
    "wal",
    "lsm",
    "btree",
    "inverted",
    "bm25",
    "tfidf",
    "tokeniser",
    "analyzer",
    "shard",
    "partition",
];

const KEYWORDS: &[&str] = &[
    "alpha", "beta", "gamma", "delta", "epsilon", "zeta", "eta", "theta", "iota", "kappa",
];

fn sentence(rng: &mut Lcg) -> String {
    let n = 5 + (rng.next_u32() % 5) as usize;
    let mut s = String::with_capacity(n * 8);
    for i in 0..n {
        if i > 0 {
            s.push(' ');
        }
        let w = VOCAB[(rng.next_u32() as usize) % VOCAB.len()];
        s.push_str(w);
    }
    s
}

// --- Corpus construction --------------------------------------------------

/// Build a fresh `Engine` populated with 100k docs covering one `text`
/// field and one `keyword` field. Run once per bench, *outside* the
/// timed region.
fn build_corpus() -> Engine {
    let engine = Engine::new();
    let mut fields = BTreeMap::new();
    fields.insert(
        "bio".into(),
        FieldSpec {
            field_type: FieldType::Text,
            analyzer: Some(Analyzer::WhitespaceLower),
            multi: None,
            dim: None,
            metric: None,
            backend: None,
            quantize: None,
        },
    );
    fields.insert(
        "tier".into(),
        FieldSpec {
            field_type: FieldType::Keyword,
            analyzer: None,
            multi: None,
            dim: None,
            metric: None,
            backend: None,
            quantize: None,
        },
    );
    engine
        .create_collection("c", CreateCollectionRequest { fields })
        .expect("create_collection");

    let mut text_rng = Lcg::new(0xD4D4);
    let mut kw_rng = Lcg::new(0xE5E5);

    // Engine caps `index` requests at MAX_INDEX_ITEMS. Build one field at
    // a time in batches sized at the cap.
    let batch = MAX_INDEX_ITEMS;
    let mut i = 0usize;
    while i < N {
        let upto = (i + batch).min(N);
        let mut bio = Vec::with_capacity(upto - i);
        let mut tier = Vec::with_capacity(upto - i);
        for j in i..upto {
            bio.push(IndexItem {
                external_id: format!("u{j}"),
                field: "bio".into(),
                value: FieldValue::String(sentence(&mut text_rng)),
            });
            tier.push(IndexItem {
                external_id: format!("u{j}"),
                field: "tier".into(),
                value: FieldValue::String(
                    KEYWORDS[(kw_rng.next_u32() as usize) % KEYWORDS.len()].to_string(),
                ),
            });
        }
        engine
            .index(
                "c",
                IndexRequest {
                    items: bio,
                    request_id: None,
                },
            )
            .expect("index bio");
        engine
            .index(
                "c",
                IndexRequest {
                    items: tier,
                    request_id: None,
                },
            )
            .expect("index tier");
        i = upto;
    }
    engine
}

// --- Benches --------------------------------------------------------------

fn bench_search(c: &mut Criterion) {
    // One shared corpus, lazily built on first access.
    let engine = build_corpus();

    let mut group = c.benchmark_group("search");
    group.sample_size(20);

    group.bench_function("match_single/100k", |b| {
        b.iter_batched(
            || SearchRequest {
                query: QueryNode::Match(MatchQuery {
                    field: "bio".into(),
                    text: "rust".into(),
                    op: MatchOp::And,
                }),
                limit: 20,
                cursor: None,
            },
            |req| {
                engine.search("c", req).expect("search");
            },
            BatchSize::SmallInput,
        );
    });

    group.bench_function("match_and_3/100k", |b| {
        b.iter_batched(
            || SearchRequest {
                query: QueryNode::Match(MatchQuery {
                    field: "bio".into(),
                    text: "rust engineer system".into(),
                    op: MatchOp::And,
                }),
                limit: 20,
                cursor: None,
            },
            |req| {
                engine.search("c", req).expect("search");
            },
            BatchSize::SmallInput,
        );
    });

    group.bench_function("term_keyword/100k", |b| {
        b.iter_batched(
            || SearchRequest {
                query: QueryNode::Term(TermQuery {
                    field: "tier".into(),
                    value: FieldValue::String("alpha".into()),
                }),
                limit: 20,
                cursor: None,
            },
            |req| {
                engine.search("c", req).expect("search");
            },
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

criterion_group!(benches, bench_search);
criterion_main!(benches);
