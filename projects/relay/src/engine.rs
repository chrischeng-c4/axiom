// SPEC-MANAGED: projects/relay/tech-design/logic/core-durable-log-single-multi-broadcast-delivery-model.md#logic
// HANDWRITE-BEGIN gap="missing-generator:logic:464901bc" tracker="pending-tracker" reason="Relay core engine tying publish -> classify -> broadcast / work-queue delivery over one durable log."
//! The in-process relay core: one durable ordered log per subject, with both
//! broadcast fan-out and work-queue competing-consumer delivery reading from
//! that same log.
//!
//! Sequencing is per `(subject, shard)`; this core implements a single shard
//! (`0`) per subject — multi-shard fan-out belongs to the transport / sharding
//! issue (#115). Synchronization across threads is the server's concern; the
//! core exposes plain `&mut self` methods.

use std::collections::{BTreeMap, HashMap};
use std::io;

use chrono::{DateTime, Utc};

use crate::broadcast::BroadcastDelivery;
use crate::config::RelayCoreConfig;
use crate::log::Log;
use crate::types::{
    AppendOutcome, CommittedOffset, DeliveryModel, Lease, LogEntry, Payload, Seq, ShardId,
};
use crate::workqueue::WorkQueue;

/// The single shard this core implements per subject.
const SHARD: ShardId = 0;

struct SubjectState {
    log: Log,
    broadcast: BroadcastDelivery,
    workqueue: WorkQueue,
    model: DeliveryModel,
}

/// In-process broker core.
///
/// @spec projects/relay/tech-design/logic/core-durable-log-single-multi-broadcast-delivery-model.md#logic
pub struct Relay {
    config: RelayCoreConfig,
    subjects: HashMap<String, SubjectState>,
}

impl Relay {
    /// Build a core over `config`. Subjects are opened lazily on first use.
    pub fn new(config: RelayCoreConfig) -> Self {
        Relay {
            config,
            subjects: HashMap::new(),
        }
    }

    fn subject_state(&mut self, subject: &str) -> io::Result<&mut SubjectState> {
        if !self.subjects.contains_key(subject) {
            let log = Log::open(&self.config, subject, SHARD)?;
            let workqueue = WorkQueue::new(
                subject,
                SHARD,
                self.config.work_queue.lease_ttl_ms,
                self.config.work_queue.max_attempts,
            );
            self.subjects.insert(
                subject.to_string(),
                SubjectState {
                    log,
                    broadcast: BroadcastDelivery::new(),
                    workqueue,
                    model: DeliveryModel::Broadcast,
                },
            );
        }
        Ok(self.subjects.get_mut(subject).expect("just inserted"))
    }

    /// Publish a message to `subject`. Idempotent on `message_id`: a repeated
    /// id returns the existing seq with `deduped = true`.
    ///
    /// @spec projects/relay/tech-design/logic/core-durable-log-single-multi-broadcast-delivery-model.md#logic
    pub fn publish(
        &mut self,
        subject: &str,
        message_id: &str,
        payload: Payload,
        headers: BTreeMap<String, String>,
        now: DateTime<Utc>,
    ) -> io::Result<AppendOutcome> {
        let ss = self.subject_state(subject)?;
        ss.log.append(message_id, payload, headers, now)
    }

    /// Set the descriptive delivery model for a subject. Both broadcast and
    /// work-queue delivery operate regardless; this records routing intent.
    pub fn set_delivery_model(&mut self, subject: &str, model: DeliveryModel) -> io::Result<()> {
        self.subject_state(subject)?.model = model;
        Ok(())
    }

    /// The recorded delivery model for a subject, if it exists.
    pub fn delivery_model(&self, subject: &str) -> Option<DeliveryModel> {
        self.subjects.get(subject).map(|s| s.model)
    }

    /// Number of entries appended to `subject`'s log.
    pub fn log_len(&mut self, subject: &str) -> io::Result<Seq> {
        Ok(self.subject_state(subject)?.log.len())
    }

    /// Register (or re-position) a broadcast subscriber to start at `from_seq`.
    ///
    /// @spec projects/relay/tech-design/logic/core-durable-log-single-multi-broadcast-delivery-model.md#logic
    pub fn subscribe(
        &mut self,
        subject: &str,
        subscriber_id: &str,
        from_seq: Seq,
    ) -> io::Result<()> {
        self.subject_state(subject)?
            .broadcast
            .subscribe(subscriber_id, from_seq);
        Ok(())
    }

    /// Deliver all not-yet-delivered entries to a broadcast subscriber, in
    /// order, advancing its cursor.
    ///
    /// @spec projects/relay/tech-design/logic/core-durable-log-single-multi-broadcast-delivery-model.md#logic
    pub fn poll(&mut self, subject: &str, subscriber_id: &str) -> io::Result<Vec<LogEntry>> {
        let ss = self.subject_state(subject)?;
        Ok(ss.broadcast.poll(subscriber_id, &ss.log))
    }

    /// Lease the next eligible entry of `subject` to a competing consumer.
    ///
    /// @spec projects/relay/tech-design/logic/core-durable-log-single-multi-broadcast-delivery-model.md#logic
    pub fn lease(
        &mut self,
        subject: &str,
        consumer_id: &str,
        now: DateTime<Utc>,
    ) -> io::Result<Option<Lease>> {
        let ss = self.subject_state(subject)?;
        let len = ss.log.len();
        Ok(ss.workqueue.lease(consumer_id, len, now))
    }

    /// Acknowledge a work-queue lease.
    ///
    /// @spec projects/relay/tech-design/logic/core-durable-log-single-multi-broadcast-delivery-model.md#logic
    pub fn ack(&mut self, subject: &str, lease_id: &str) -> io::Result<bool> {
        Ok(self.subject_state(subject)?.workqueue.ack(lease_id))
    }

    /// Reclaim expired leases on `subject`, making their entries redelivery-
    /// eligible. Returns the reclaimed seqs.
    ///
    /// @spec projects/relay/tech-design/logic/core-durable-log-single-multi-broadcast-delivery-model.md#logic
    pub fn reclaim_expired(&mut self, subject: &str, now: DateTime<Utc>) -> io::Result<Vec<Seq>> {
        Ok(self.subject_state(subject)?.workqueue.reclaim_expired(now))
    }

    /// The committed work-queue offset for `subject`.
    ///
    /// @spec projects/relay/tech-design/logic/core-durable-log-single-multi-broadcast-delivery-model.md#logic
    pub fn committed_offset(&mut self, subject: &str) -> io::Result<Option<CommittedOffset>> {
        Ok(self.subject_state(subject)?.workqueue.committed_offset())
    }
}
// HANDWRITE-END
