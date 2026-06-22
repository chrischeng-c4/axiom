// SPEC-MANAGED: projects/lumen/tech-design/semantic/lumen-tests.md#unit-test
// CODEGEN-BEGIN
//! Disk-format measurement: old `serde_json` path vs the new CBOR + lz4 path,
//! on the *same* RDB snapshot of a vector-heavy corpus. Run with:
//!
//! ```sh
//! cargo test -p lumen --test disk_format_bench -- --ignored --nocapture
//! ```
//!
//! This is a measurement harness, not an assertion test — it prints size and
//! decode-time ratios so the Stage-1 "kill JSON" win is quantified, not claimed.

use std::collections::BTreeMap;
use std::time::Instant;

use lumen::rdb::RdbSnapshot;
use lumen::storage::Engine;
use lumen::types::{
    CreateCollectionRequest, FieldSpec, FieldType, FieldValue, IndexItem, IndexRequest,
    VectorBackend, VectorMetric,
};

/// Deterministic LCG so the corpus is host-independent.
struct Lcg(u64);
/// @spec projects/lumen/tech-design/semantic/lumen-tests.md#unit-test
impl Lcg {
    fn new(seed: u64) -> Self {
        Self(seed.wrapping_mul(6_364_136_223_846_793_005) ^ 0x9E37_79B9_7F4A_7C15)
    }
    fn next_u64(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.0
    }
    fn next_f32(&mut self) -> f32 {
        let bits = (self.next_u64() >> 32) as u32;
        ((bits as f64) / (u32::MAX as f64) * 2.0 - 1.0) as f32
    }
}

fn vec_field(dim: u32, metric: VectorMetric) -> FieldSpec {
    FieldSpec {
        field_type: FieldType::Vector,
        analyzer: None,
        multi: None,
        dim: Some(dim),
        metric: Some(metric),
        // FlatCpu stores vectors without building an HNSW graph, so corpus
        // setup is fast. The snapshot content (raw vectors) is backend-agnostic.
        backend: Some(VectorBackend::FlatCpu),
        quantize: None,
    }
}

fn scalar_field(ty: FieldType) -> FieldSpec {
    FieldSpec {
        field_type: ty,
        analyzer: None,
        multi: None,
        dim: None,
        metric: None,
        backend: None,
        quantize: None,
    }
}

#[test]
#[ignore = "measurement harness — run explicitly with --ignored --nocapture"]
fn disk_format_size_and_decode_speed() {
    const N: usize = 5_000;
    const DIM: u32 = 128;

    let engine = Engine::new();
    let mut fields = BTreeMap::new();
    fields.insert(
        "embedding".to_string(),
        vec_field(DIM, VectorMetric::Cosine),
    );
    fields.insert("title".to_string(), scalar_field(FieldType::Text));
    fields.insert("category".to_string(), scalar_field(FieldType::Keyword));
    engine
        .create_collection("docs", CreateCollectionRequest { fields })
        .unwrap();

    let mut rng = Lcg::new(0xC0DE_CAFE_u64);
    let mut items = Vec::with_capacity(N * 3);
    for i in 0..N {
        let eid = format!("doc-{i}");
        let v: Vec<f32> = (0..DIM).map(|_| rng.next_f32()).collect();
        items.push(IndexItem {
            external_id: eid.clone(),
            field: "embedding".into(),
            value: FieldValue::Vector(v),
        });
        items.push(IndexItem {
            external_id: eid.clone(),
            field: "title".into(),
            value: FieldValue::String(format!(
                "document {i} lorem ipsum dolor sit amet consectetur token{}",
                i % 97
            )),
        });
        items.push(IndexItem {
            external_id: eid,
            field: "category".into(),
            value: FieldValue::String(format!("cat-{}", i % 50)),
        });
    }
    // Bulk index caps at 10k items/request; chunk the writes.
    for chunk in items.chunks(9_000) {
        engine
            .index(
                "docs",
                IndexRequest {
                    items: chunk.to_vec(),
                    request_id: None,
                },
            )
            .unwrap();
    }

    let rdb = RdbSnapshot::capture(&engine, N as u64).unwrap();

    // --- sizes ---
    let json = serde_json::to_vec(&rdb).unwrap();
    let json_lz4 = lz4_flex::compress_prepend_size(&json);
    let cbor_lz4 = rdb.encode().unwrap(); // the production path (CBOR + lz4)
    let mut cbor_raw = Vec::new();
    ciborium::into_writer(&rdb, &mut cbor_raw).unwrap();

    let kb = |b: usize| b as f64 / 1024.0;
    println!("\n=== RDB on-disk size (N={N} docs, dim={DIM} + text + keyword) ===");
    println!("serde_json (old):       {:>9.1} KiB", kb(json.len()));
    println!("serde_json + lz4:       {:>9.1} KiB", kb(json_lz4.len()));
    println!("CBOR (raw):             {:>9.1} KiB", kb(cbor_raw.len()));
    println!(
        "CBOR + lz4 (NEW):       {:>9.1} KiB   <-- production",
        kb(cbor_lz4.len())
    );
    println!(
        "size win vs JSON:           {:.2}x smaller (uncompressed), {:.2}x vs JSON+lz4",
        json.len() as f64 / cbor_lz4.len() as f64,
        json_lz4.len() as f64 / cbor_lz4.len() as f64
    );

    // --- decode speed (cold-start page-in cost) ---
    const ITERS: u32 = 20;
    let t0 = Instant::now();
    for _ in 0..ITERS {
        let v: RdbSnapshot = serde_json::from_slice(&json).unwrap();
        std::hint::black_box(&v);
    }
    let json_dec = t0.elapsed() / ITERS;

    let t1 = Instant::now();
    for _ in 0..ITERS {
        let v = RdbSnapshot::decode(&cbor_lz4).unwrap();
        std::hint::black_box(&v);
    }
    let cbor_dec = t1.elapsed() / ITERS;

    println!("\n=== decode time (per snapshot, avg of {ITERS}) ===");
    println!("serde_json::from_slice: {:>9.2?}", json_dec);
    println!("CBOR+lz4 decode (NEW):  {:>9.2?}", cbor_dec);
    println!(
        "decode speedup:             {:.2}x faster\n",
        json_dec.as_secs_f64() / cbor_dec.as_secs_f64()
    );
}
// CODEGEN-END
