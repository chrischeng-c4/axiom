// <HANDWRITE gap="standardize:claim-code" tracker="projects-lumen-benches-bench-duplicates-rs" reason="Existing code claimed during Score standardization until deterministic generator coverage lands.">
//! Duplicate-detection bench.
//!
//! Builds a 100k-doc keyword field with ~10% duplicate rate
//! (≈ 90k distinct keys, 10k of which are repeated 2–3×), then measures
//! one `duplicates` call. README §9 perf-regression suite.

use std::collections::BTreeMap;

use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use lumen::storage::{Engine, MAX_INDEX_ITEMS};
use lumen::types::{
    CreateCollectionRequest, DuplicatesRequest, FieldSpec, FieldType, FieldValue, IndexItem,
    IndexRequest,
};

const N: usize = 100_000;
/// Target ~10% of docs share a value with at least one other doc.
/// With this many "hot" keys repeated 2-3× and the rest unique, the
/// dup detector has to walk a sparse `terms` map.
const HOT_KEYS: usize = 5_000;

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

fn build_corpus() -> Engine {
    let engine = Engine::new();
    let mut fields = BTreeMap::new();
    fields.insert(
        "email".into(),
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

    let mut rng = Lcg::new(0xF6F6);
    let batch = MAX_INDEX_ITEMS;
    let mut i = 0usize;
    while i < N {
        let upto = (i + batch).min(N);
        let mut items = Vec::with_capacity(upto - i);
        for j in i..upto {
            // 10% of writes pick a "hot" key (collides with ≥1 other),
            // 90% pick a globally unique key.
            let value = if rng.next_u32() % 10 == 0 {
                format!("hot{}@example.com", rng.next_u32() % HOT_KEYS as u32)
            } else {
                format!("unique{j}@example.com")
            };
            items.push(IndexItem {
                external_id: format!("u{j}"),
                field: "email".into(),
                value: FieldValue::String(value),
            });
        }
        engine
            .index(
                "c",
                IndexRequest {
                    items,
                    request_id: None,
                },
            )
            .expect("index");
        i = upto;
    }
    engine
}

fn bench_duplicates(c: &mut Criterion) {
    let engine = build_corpus();

    let mut group = c.benchmark_group("duplicates");
    group.sample_size(20);

    group.bench_function("keyword/100k", |b| {
        b.iter_batched(
            || DuplicatesRequest {
                field: "email".into(),
                min_group_size: 2,
                limit: 100,
                offset: 0,
            },
            |req| {
                engine.duplicates("c", req).expect("duplicates");
            },
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

criterion_group!(benches, bench_duplicates);
criterion_main!(benches);

// </HANDWRITE>
