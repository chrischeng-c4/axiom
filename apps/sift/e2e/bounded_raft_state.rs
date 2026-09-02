use std::{collections::BTreeMap, sync::Arc};

use raft_runtime::RaftStateMachine;
use sift::{
    durability::{
        decode_raft_batch_for_diagnostics, encode_default_raft_batch_for_diagnostics,
        encode_raft_batch_for_diagnostics,
    },
    DurableJournal, EventEnvelope, SignalKind,
};

fn log(id: impl Into<String>) -> EventEnvelope {
    let mut event = EventEnvelope::for_project(
        "bounded-raft",
        "test",
        id,
        SignalKind::Log,
        serde_json::json!({"message":"the repeated body makes compression measurable"}),
    );
    event.resource = BTreeMap::from([("service.name".into(), "bounded-raft".into())]);
    event
}

#[test]
fn follower_apply_outcomes_age_out_of_a_small_fixed_window() {
    let data = tempfile::tempdir().unwrap();
    let journal = Arc::new(DurableJournal::open(data.path()).unwrap());
    let state_machine = sift::durability::SiftStateMachine::new(journal);
    for index in 1..=100_u64 {
        let command = serde_json::to_vec(&serde_json::json!({
            "kind":"append_events",
            "events":[log(format!("outcome-{index}"))]
        }))
        .unwrap();
        state_machine.apply(index, &command).unwrap();
    }

    assert!(
        state_machine.take_append_outcomes(1).is_none(),
        "an outcome that no local request claimed must age out"
    );
    assert!(state_machine.take_append_outcomes(100).is_some());
}

#[test]
fn repetitive_thousand_item_batches_use_a_bounded_compressed_wire_codec() {
    let events = (0..1_000)
        .map(|index| log(format!("compressed-{index:04}")))
        .collect::<Vec<_>>();
    let legacy = serde_json::to_vec(&serde_json::json!({
        "kind":"append_events",
        "events":events
    }))
    .unwrap();
    let encoded = encode_raft_batch_for_diagnostics(events.clone()).unwrap();
    assert!(encoded.starts_with(b"SIFTCMD1"));
    assert!(
        encoded.len() * 8 < legacy.len(),
        "the repetitive MVP load batch must compress by at least 8x"
    );
    assert_eq!(decode_raft_batch_for_diagnostics(&encoded).unwrap(), events);
    assert_eq!(decode_raft_batch_for_diagnostics(&legacy).unwrap(), events);
    let production = encode_default_raft_batch_for_diagnostics(events.clone()).unwrap();
    assert_eq!(
        decode_raft_batch_for_diagnostics(&production).unwrap(),
        events
    );
    assert_ne!(production, legacy);
    let production: serde_json::Value = serde_json::from_slice(&production).unwrap();
    assert!(production["acknowledged_at"].as_str().is_some());
}
