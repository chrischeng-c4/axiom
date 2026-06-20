// SPEC-MANAGED: projects/relay/tech-design/logic/competitor-perf-gate-vs-nats-rabbitmq-redpanda-arena-ratchet.md#unit-test
// HANDWRITE-BEGIN gap="missing-generator:unit-test:9056e8a0" tracker="pending-tracker" reason="criterion benchmarks for the three gate cells: append throughput, broadcast fan-out, work-queue lease+ack cycle (the relay-side measurement)."
//! relay-side measurement for the competitor perf-gate (#125): the three gate
//! cells benched against the in-process core. arena drives the same logical
//! workloads against relay's HTTP/2 service and the competitors; these
//! criterion benches are the local, competitor-free baseline.

use std::collections::BTreeMap;

use chrono::Utc;
use criterion::{criterion_group, criterion_main, BatchSize, Criterion};

use relay::{Relay, RelayCoreConfig};

const N: usize = 1_000;

fn fresh() -> Relay {
    Relay::new(RelayCoreConfig::in_memory())
}

fn seeded() -> Relay {
    let mut r = fresh();
    let now = Utc::now();
    for i in 0..N {
        r.publish(
            "bench",
            &format!("m{i}"),
            serde_json::json!({ "i": i }),
            BTreeMap::new(),
            now,
        )
        .unwrap();
    }
    r
}

// durable log: append throughput.
fn bench_append(c: &mut Criterion) {
    c.bench_function("append_1k", |b| {
        b.iter_batched(
            fresh,
            |mut r| {
                let now = Utc::now();
                for i in 0..N {
                    r.publish(
                        "bench",
                        &format!("m{i}"),
                        serde_json::json!({ "i": i }),
                        BTreeMap::new(),
                        now,
                    )
                    .unwrap();
                }
            },
            BatchSize::SmallInput,
        )
    });
}

// broadcast: fan-out delivery of a full log to a subscriber.
fn bench_broadcast_fanout(c: &mut Criterion) {
    c.bench_function("broadcast_fanout_1k", |b| {
        b.iter_batched(
            || {
                let mut r = seeded();
                r.subscribe("bench", "sub", 0).unwrap();
                r
            },
            |mut r| {
                let got = r.poll("bench", "sub").unwrap();
                assert_eq!(got.len(), N);
            },
            BatchSize::SmallInput,
        )
    });
}

// work-queue: lease + ack cycle over a full log.
fn bench_work_queue_cycle(c: &mut Criterion) {
    c.bench_function("work_queue_lease_ack_1k", |b| {
        b.iter_batched(
            seeded,
            |mut r| {
                let now = Utc::now();
                let mut acked = 0;
                while let Some(l) = r.lease("bench", "c", now).unwrap() {
                    if r.ack("bench", &l.lease_id, Some(l.epoch)).unwrap() {
                        acked += 1;
                    }
                }
                assert_eq!(acked, N);
            },
            BatchSize::SmallInput,
        )
    });
}

criterion_group!(
    benches,
    bench_append,
    bench_broadcast_fanout,
    bench_work_queue_cycle
);
criterion_main!(benches);
// HANDWRITE-END
