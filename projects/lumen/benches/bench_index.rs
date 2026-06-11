// <HANDWRITE gap="standardize:claim-code" tracker="projects-lumen-benches-bench-index-rs" reason="Existing code claimed during Score standardization until deterministic generator coverage lands.">
//! Index-throughput benches.
//!
//! Drives the `Engine` directly (not the HTTP layer) since the README
//! §9 perf gate is engine-level — going through axum + reqwest would
//! conflate transport, serialisation, and storage costs.
//!
//! Three scenarios, each writing 10k items in one `index` call:
//!   * `index/keyword/10k`  — `BTreeMap<String, BTreeSet<String>>` path
//!   * `index/text/10k`     — `whitespace_lower` analyzer + tokenisation
//!   * `index/number/10k`   — `SortableF64` keyed `BTreeMap`
//!
//! Setup (collection creation + payload generation) is built in the
//! batch closure of `iter_batched` so the measured work is pure indexing.

use std::collections::BTreeMap;

use criterion::{criterion_group, criterion_main, BatchSize, Criterion, Throughput};
use lumen::storage::Engine;
use lumen::types::{
    Analyzer, CreateCollectionRequest, FieldSpec, FieldType, FieldValue, IndexItem, IndexRequest,
};

const N: usize = 10_000;

// --- Deterministic, allocation-light fixture data -------------------------

/// Seeded LCG (Numerical Recipes constants). Avoids pulling in `rand`.
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

const TEXT_WORDS: &[&str] = &[
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
];

/// Build a `whitespace_lower`-friendly sentence of 5–9 tokens, drawn
/// from a 30-word vocabulary so token frequency varies but is bounded.
fn sentence(rng: &mut Lcg) -> String {
    let n = 5 + (rng.next_u32() % 5) as usize;
    let mut s = String::with_capacity(n * 8);
    for i in 0..n {
        if i > 0 {
            s.push(' ');
        }
        let w = TEXT_WORDS[(rng.next_u32() as usize) % TEXT_WORDS.len()];
        s.push_str(w);
    }
    s
}

fn keyword_items() -> Vec<IndexItem> {
    let mut rng = Lcg::new(0xA1A1);
    (0..N)
        .map(|i| IndexItem {
            external_id: format!("u{i}"),
            field: "email".into(),
            value: FieldValue::String(format!("user{}@example.com", rng.next_u32() % 5_000)),
        })
        .collect()
}

fn text_items() -> Vec<IndexItem> {
    let mut rng = Lcg::new(0xB2B2);
    (0..N)
        .map(|i| IndexItem {
            external_id: format!("u{i}"),
            field: "bio".into(),
            value: FieldValue::String(sentence(&mut rng)),
        })
        .collect()
}

fn number_items() -> Vec<IndexItem> {
    let mut rng = Lcg::new(0xC3C3);
    (0..N)
        .map(|i| IndexItem {
            external_id: format!("u{i}"),
            field: "age".into(),
            value: FieldValue::Number((rng.next_u32() % 100) as f64),
        })
        .collect()
}

// --- Fresh-engine factory --------------------------------------------------

fn fresh_engine(field: &str, ftype: FieldType, analyzer: Option<Analyzer>) -> Engine {
    let engine = Engine::new();
    let mut fields = BTreeMap::new();
    fields.insert(
        field.to_string(),
        FieldSpec {
            field_type: ftype,
            analyzer,
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
    engine
}

// --- Benches --------------------------------------------------------------

fn bench_index(c: &mut Criterion) {
    let mut group = c.benchmark_group("index");
    group.throughput(Throughput::Elements(N as u64));
    group.sample_size(10);

    group.bench_function("keyword/10k", |b| {
        b.iter_batched(
            || {
                (
                    fresh_engine("email", FieldType::Keyword, None),
                    keyword_items(),
                )
            },
            |(engine, items)| {
                engine
                    .index(
                        "c",
                        IndexRequest {
                            items,
                            request_id: None,
                        },
                    )
                    .expect("index");
            },
            BatchSize::SmallInput,
        );
    });

    group.bench_function("text/10k", |b| {
        b.iter_batched(
            || {
                (
                    fresh_engine("bio", FieldType::Text, Some(Analyzer::WhitespaceLower)),
                    text_items(),
                )
            },
            |(engine, items)| {
                engine
                    .index(
                        "c",
                        IndexRequest {
                            items,
                            request_id: None,
                        },
                    )
                    .expect("index");
            },
            BatchSize::SmallInput,
        );
    });

    group.bench_function("number/10k", |b| {
        b.iter_batched(
            || (fresh_engine("age", FieldType::Number, None), number_items()),
            |(engine, items)| {
                engine
                    .index(
                        "c",
                        IndexRequest {
                            items,
                            request_id: None,
                        },
                    )
                    .expect("index");
            },
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

criterion_group!(benches, bench_index);
criterion_main!(benches);

// </HANDWRITE>
