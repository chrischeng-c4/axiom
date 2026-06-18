// SPEC-MANAGED: projects/relay/tech-design/logic/core-durable-log-single-multi-broadcast-delivery-model.md#logic
// HANDWRITE-BEGIN gap="missing-generator:logic:fb5859a5" tracker="pending-tracker" reason="Broadcast / multicast fan-out and replay-from-seq over the log via per-subscriber cursors."
//! Broadcast / multicast delivery: every subscriber receives every message in
//! seq order, replayable from any seq, via an independent per-subscriber cursor.

use std::collections::HashMap;

use crate::log::Log;
use crate::types::{LogEntry, Seq, ShardId, SubscriberCursor};

struct Cursor {
    from_seq: Seq,
    /// Next seq to deliver to this subscriber.
    next: Seq,
}

/// Fan-out delivery over a single log. Each subscriber advances its own cursor;
/// none consume from another, so all see the full ordered stream.
///
/// @spec projects/relay/tech-design/logic/core-durable-log-single-multi-broadcast-delivery-model.md#logic
#[derive(Default)]
pub struct BroadcastDelivery {
    subscribers: HashMap<String, Cursor>,
}

impl BroadcastDelivery {
    pub fn new() -> Self {
        BroadcastDelivery::default()
    }

    /// Register (or re-position) a subscriber to start delivery at `from_seq`.
    ///
    /// Replaying is just subscribing from an earlier seq.
    ///
    /// @spec projects/relay/tech-design/logic/core-durable-log-single-multi-broadcast-delivery-model.md#logic
    pub fn subscribe(&mut self, subscriber_id: &str, from_seq: Seq) {
        self.subscribers.insert(
            subscriber_id.to_string(),
            Cursor {
                from_seq,
                next: from_seq,
            },
        );
    }

    /// Remove a subscriber.
    pub fn unsubscribe(&mut self, subscriber_id: &str) -> bool {
        self.subscribers.remove(subscriber_id).is_some()
    }

    /// Deliver every not-yet-delivered entry to `subscriber_id` and advance its
    /// cursor. Returns the entries in seq order; empty if the subscriber is
    /// caught up or unknown.
    ///
    /// @spec projects/relay/tech-design/logic/core-durable-log-single-multi-broadcast-delivery-model.md#logic
    pub fn poll(&mut self, subscriber_id: &str, log: &Log) -> Vec<LogEntry> {
        let Some(cursor) = self.subscribers.get_mut(subscriber_id) else {
            return Vec::new();
        };
        let out: Vec<LogEntry> = log.range(cursor.next).to_vec();
        cursor.next = log.len();
        out
    }

    /// Snapshot of a subscriber's cursor.
    pub fn cursor(
        &self,
        subscriber_id: &str,
        subject: &str,
        shard: ShardId,
    ) -> Option<SubscriberCursor> {
        self.subscribers
            .get(subscriber_id)
            .map(|c| SubscriberCursor {
                subscriber_id: subscriber_id.to_string(),
                subject: subject.to_string(),
                shard,
                from_seq: c.from_seq,
                position: c.next,
            })
    }
}
// HANDWRITE-END
