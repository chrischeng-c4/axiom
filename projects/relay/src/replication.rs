// SPEC-MANAGED: projects/relay/tech-design/logic/ha-via-leader-follower-log-replication-async-primary-backup.md#logic
// HANDWRITE-BEGIN gap="missing-generator:logic:867abe72" tracker="pending-tracker" reason="spawn_follower(local, leader_url, subjects) -> FollowerHandle: one tokio task per subject that tails the leader's subscribe stream over h2c, decodes length-prefixed CBOR LogEntry frames, and re-applies each via local.publish (idempotent on message_id); reconnects with backoff on stream end/error. FollowerHandle aborts the tasks on stop/drop."
//! HA via leader/follower log replication (async primary-backup).
//!
//! A follower tails the leader's broadcast `subscribe` stream for each subject
//! and re-applies every entry to its own durable log via [`crate::Relay::publish`].
//! Because publish is idempotent on `message_id` and routing is deterministic,
//! the follower mirrors the leader's content (and shard/seq), reconnects are
//! safe (the dedupe window absorbs replay), and a caught-up follower can be
//! promoted by pointing producers/consumers at it.
//!
//! This is **asynchronous** primary-backup: followers are eventually
//! consistent. Synchronous quorum writes and automatic leader election (full
//! Raft) are out of scope.

use std::sync::Arc;
use std::time::Duration;

use futures::StreamExt;

use crate::engine::Relay;
use crate::types::LogEntry;
use crate::wire::decode_frames;

/// Handle to a running follower; drop or [`stop`](FollowerHandle::stop) to end
/// replication.
///
/// @spec projects/relay/tech-design/logic/ha-via-leader-follower-log-replication-async-primary-backup.md#logic
pub struct FollowerHandle {
    tasks: Vec<tokio::task::JoinHandle<()>>,
}

impl FollowerHandle {
    /// Stop replicating (aborts the per-subject tasks).
    pub fn stop(self) {
        for t in &self.tasks {
            t.abort();
        }
    }
}

impl Drop for FollowerHandle {
    fn drop(&mut self) {
        for t in &self.tasks {
            t.abort();
        }
    }
}

/// Spawn a follower that replicates `subjects` from `leader_url` into `local`.
///
/// One task per subject tails the leader's `subscribe` stream over h2c and
/// applies each entry via `local.publish` (idempotent on `message_id`). On
/// stream end / error it backs off and reconnects.
///
/// @spec projects/relay/tech-design/logic/ha-via-leader-follower-log-replication-async-primary-backup.md#logic
pub fn spawn_follower(
    local: Arc<Relay>,
    leader_url: impl Into<String>,
    subjects: Vec<String>,
) -> FollowerHandle {
    let leader = leader_url.into();
    let mut tasks = Vec::new();
    for subject in subjects {
        let local = Arc::clone(&local);
        let leader = leader.clone();
        tasks.push(tokio::spawn(async move {
            let client = match reqwest::Client::builder().http2_prior_knowledge().build() {
                Ok(c) => c,
                Err(_) => return,
            };
            let url =
                format!("{leader}/v1/{subject}/subscribe?from_seq=0&subscriber_id=relay-follower");
            loop {
                if let Ok(resp) = client.get(&url).send().await {
                    let mut stream = resp.bytes_stream();
                    let mut buf: Vec<u8> = Vec::new();
                    while let Some(chunk) = stream.next().await {
                        let Ok(chunk) = chunk else { break };
                        buf.extend_from_slice(&chunk);
                        let (frames, used): (Vec<LogEntry>, usize) = decode_frames(&buf);
                        if used > 0 {
                            buf.drain(0..used);
                        }
                        for e in frames {
                            // Idempotent on message_id; reconnect/replay is safe.
                            let _ = local.publish(
                                &subject,
                                &e.message_id,
                                e.payload,
                                e.headers,
                                e.appended_at,
                            );
                        }
                    }
                }
                // Stream ended or errored: back off, then reconnect from seq 0.
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
        }));
    }
    FollowerHandle { tasks }
}
// HANDWRITE-END
